// src/kernel/scheduler.rs

use crate::arch::aarch64::context::context_switch;
use crate::arch::aarch64::exception::{
    disable_interrupts, restore_interrupts, save_and_disable_interrupts, ExceptionContext,
};
use crate::kernel::sync::IrqSafeSpinLock;
use crate::kernel::task::{Task, TaskEntry, TaskState, MAX_TASKS, TASK_STACK_SIZE};

static mut TASK_STACKS: [[u8; TASK_STACK_SIZE]; MAX_TASKS] = [[0; TASK_STACK_SIZE]; MAX_TASKS];

fn scheduler_boot_anchor() -> ! {
    panic!("scheduler_boot_anchor reached unexpectedly");
}

const fn empty_context() -> ExceptionContext {
    ExceptionContext {
        x: [0; 31],
        sp: 0,
        elr_el1: 0,
        spsr_el1: 0,
    }
}

struct SwitchPlan {
    old_ctx: *mut ExceptionContext,
    new_ctx: *const ExceptionContext,
}

pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    task_count: usize,
    current: Option<usize>,
    next_id: usize,
    idle_index: Option<usize>,
    boot_context: ExceptionContext,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: [None, None, None, None, None, None, None, None],
            task_count: 0,
            current: None,
            next_id: 0,
            idle_index: None,
            boot_context: empty_context(),
        }
    }

    pub fn init(&mut self) {
        self.task_count = 0;
        self.current = None;
        self.next_id = 0;
        self.idle_index = None;
        self.boot_context = empty_context();

        for slot in self.tasks.iter_mut() {
            *slot = None;
        }
    }

    fn alloc_stack_top(&self, slot: usize) -> usize {
        unsafe {
            let base = core::ptr::addr_of!(TASK_STACKS[slot]) as *const u8 as usize;
            let top = base + TASK_STACK_SIZE;
            top & !0xFusize
        }
    }

    pub fn spawn(&mut self, name: &'static str, entry: TaskEntry) -> Result<usize, &'static str> {
        if self.task_count >= MAX_TASKS {
            return Err("scheduler full");
        }

        let id = self.next_id;
        self.next_id += 1;

        let slot = self.task_count;
        let stack_top = self.alloc_stack_top(slot);

        self.tasks[slot] = Some(Task::new(
            id,
            name,
            entry,
            stack_top,
            task_trampoline as usize,
        ));
        self.task_count += 1;

        Ok(id)
    }

    pub fn set_idle_task(&mut self, entry: TaskEntry) -> Result<usize, &'static str> {
        if self.idle_index.is_some() {
            return Err("idle task already set");
        }

        if self.task_count >= MAX_TASKS {
            return Err("scheduler full");
        }

        let id = self.next_id;
        self.next_id += 1;

        let slot = self.task_count;
        let stack_top = self.alloc_stack_top(slot);

        self.tasks[slot] = Some(Task::new_idle(
            id,
            entry,
            stack_top,
            task_trampoline as usize,
        ));
        self.task_count += 1;
        self.idle_index = Some(slot);

        Ok(id)
    }

    pub fn current_task_id(&self) -> Option<usize> {
        self.current
            .and_then(|idx| self.tasks[idx].as_ref().map(|task| task.id.0))
    }

    fn next_ready_index(&self, include_current: bool) -> Option<usize> {
        if self.task_count == 0 {
            return None;
        }

        let start = match self.current {
            Some(i) => (i + 1) % self.task_count,
            None => 0,
        };

        let mut idle_candidate = None;

        for offset in 0..self.task_count {
            let idx = (start + offset) % self.task_count;

            if !include_current && self.current == Some(idx) {
                continue;
            }

            let Some(task) = &self.tasks[idx] else {
                continue;
            };

            if task.state != TaskState::Ready {
                continue;
            }

            if task.is_idle {
                if idle_candidate.is_none() {
                    idle_candidate = Some(idx);
                }
                continue;
            }

            return Some(idx);
        }

        idle_candidate
    }

    fn prepare_boot_dispatch(&mut self) -> Option<SwitchPlan> {
        let next_idx = self.next_ready_index(false)?;

        self.current = Some(next_idx);

        let next_task = self.tasks[next_idx].as_mut().expect("task slot empty");
        next_task.state = TaskState::Running;
        self.boot_context.x[30] = scheduler_boot_anchor as usize as u64;

        crate::log!(
            "SCHED",
            "dispatching first task id={} name={}",
            next_task.id.0,
            next_task.name
        );

        let old_ctx = &mut self.boot_context as *mut ExceptionContext;
        let new_ctx = &next_task.frame as *const ExceptionContext;

        Some(SwitchPlan { old_ctx, new_ctx })
    }

    fn prepare_switch_from_current(&mut self, current_next_state: TaskState) -> Option<SwitchPlan> {
        let current_idx = self.current?;

        {
            let current = self.tasks[current_idx]
                .as_mut()
                .expect("current task slot empty");

            match current_next_state {
                TaskState::Ready => {
                    if !current.is_idle {
                        crate::log!("SCHED", "task id={} yielding", current.id.0);
                    }
                }
                TaskState::Sleeping => {
                    crate::log!("SCHED", "task id={} sleeping forever", current.id.0);
                }
                TaskState::Finished => {
                    crate::log!("SCHED", "task id={} finished", current.id.0);
                }
                TaskState::Running => {
                    panic!("prepare_switch_from_current: invalid next state Running");
                }
            }

            current.state = current_next_state;
        }

        let include_current = current_next_state == TaskState::Ready;

        let next_idx = match self.next_ready_index(include_current) {
            Some(idx) => idx,
            None => {
                let current = self.tasks[current_idx]
                    .as_mut()
                    .expect("current task slot empty");
                current.state = TaskState::Running;
                return None;
            }
        };

        if next_idx == current_idx {
            let current = self.tasks[current_idx]
                .as_mut()
                .expect("current task slot empty");
            current.state = TaskState::Running;
            return None;
        }

        self.current = Some(next_idx);

        let old_id = self.tasks[current_idx].as_ref().unwrap().id.0;

        let new_id = {
            let next = self.tasks[next_idx].as_mut().unwrap();
            next.state = TaskState::Running;
            next.id.0
        };

        crate::log!("SCHED", "switching task id={} -> id={}", old_id, new_id);

        let old_ctx = {
            let current = self.tasks[current_idx].as_mut().unwrap();
            &mut current.frame as *mut ExceptionContext
        };

        let new_ctx = {
            let next = self.tasks[next_idx].as_ref().unwrap();
            &next.frame as *const ExceptionContext
        };

        Some(SwitchPlan { old_ctx, new_ctx })
    }

    fn prepare_preempt_from_irq(
        &mut self,
        interrupted_ctx: &ExceptionContext,
    ) -> Option<ExceptionContext> {
        let current_idx = self.current?;

        {
            let current = self.tasks[current_idx]
                .as_mut()
                .expect("current task slot empty");

            current.frame = *interrupted_ctx;
            current.state = TaskState::Ready;
        }

        let next_idx = match self.next_ready_index(true) {
            Some(idx) => idx,
            None => {
                let current = self.tasks[current_idx]
                    .as_mut()
                    .expect("current task slot empty");
                current.state = TaskState::Running;
                return None;
            }
        };

        if next_idx == current_idx {
            let current = self.tasks[current_idx]
                .as_mut()
                .expect("current task slot empty");
            current.state = TaskState::Running;
            return None;
        }

        self.current = Some(next_idx);

        let old_id = self.tasks[current_idx].as_ref().unwrap().id.0;

        let next_ctx = {
            let next = self.tasks[next_idx].as_mut().unwrap();
            next.state = TaskState::Running;

            crate::log!(
                "SCHED",
                "preempt task id={} -> id={}",
                old_id,
                next.id.0
            );

            next.frame
        };

        Some(next_ctx)
    }
}

static SCHEDULER: IrqSafeSpinLock<Scheduler> = IrqSafeSpinLock::new(Scheduler::new());

#[inline(always)]
fn perform_context_switch(plan: SwitchPlan) {
    let irq_state = save_and_disable_interrupts();
    unsafe { context_switch(plan.old_ctx, plan.new_ctx) };
    restore_interrupts(irq_state);
}

pub fn init() {
    SCHEDULER.lock().init();
}

pub fn spawn(name: &'static str, entry: TaskEntry) -> Result<usize, &'static str> {
    SCHEDULER.lock().spawn(name, entry)
}

pub fn set_idle_task(entry: TaskEntry) -> Result<usize, &'static str> {
    SCHEDULER.lock().set_idle_task(entry)
}

pub fn current_task_id() -> Option<usize> {
    SCHEDULER.lock().current_task_id()
}

pub fn yield_now() {
    let plan = {
        let mut sched = SCHEDULER.lock();
        sched.prepare_switch_from_current(TaskState::Ready)
    };

    if let Some(plan) = plan {
        perform_context_switch(plan);
    }
}

pub fn sleep_forever() -> ! {
    let plan = {
        let mut sched = SCHEDULER.lock();
        sched.prepare_switch_from_current(TaskState::Sleeping)
    };

    let plan = plan.expect("sleep_forever: no runnable task available");

    perform_context_switch(plan);

    panic!("sleep_forever: returned after context_switch");
}

pub fn mark_current_finished() -> ! {
    let plan = {
        let mut sched = SCHEDULER.lock();
        sched.prepare_switch_from_current(TaskState::Finished)
    };

    let plan = plan.expect("mark_current_finished: no runnable task available");

    perform_context_switch(plan);

    panic!("mark_current_finished: returned after context_switch");
}

pub fn run() -> ! {
    crate::log!("SCHED", "scheduler started");

    let plan = {
        let mut sched = SCHEDULER.lock();
        sched.prepare_boot_dispatch()
    };

    let plan = plan.expect("scheduler run: no runnable tasks");

    disable_interrupts();
    unsafe { context_switch(plan.old_ctx, plan.new_ctx) };

    panic!("scheduler run: returned from initial context_switch");
}

pub fn preempt_from_irq(interrupted_ctx: &ExceptionContext) -> Option<ExceptionContext> {
    let mut sched = SCHEDULER.lock();
    sched.prepare_preempt_from_irq(interrupted_ctx)
}

#[unsafe(no_mangle)]
pub extern "C" fn task_trampoline() -> ! {
    let entry_addr: usize;

    unsafe {
        core::arch::asm!("mov {}, x19", out(reg) entry_addr);
    }

    let entry: TaskEntry = unsafe { core::mem::transmute::<usize, TaskEntry>(entry_addr) };

    entry();
    mark_current_finished();
}