// src/kernel/scheduler.rs
//
// Scheduler cooperativo/preemptivo unificado.
//
// Todo switch de contexto passa pelo mesmo caminho via eret:
//   - Preemptivo: IRQ de timer → preempt_from_irq → modifica frame → eret
//   - Cooperativo: task emite svc #N → rust_svc_handler → modifica frame → eret
//   - Boot dispatch: run() emite svc #3 → primeiro eret via mesmo caminho
//
// Não existe mais context_switch cooperativo separado.
//

use crate::arch::aarch64::exception::ExceptionContext;
use crate::kernel::sync::IrqSafeSpinLock;
use crate::kernel::task::{Task, TaskEntry, TaskState, MAX_TASKS, TASK_STACK_SIZE};

// Números de SVC — devem coincidir com os emitidos pelas funções públicas.
pub const SVC_YIELD:    u64 = 0;
pub const SVC_SLEEP:    u64 = 1;
pub const SVC_FINISHED: u64 = 2;
pub const SVC_BOOT:     u64 = 3;

static mut TASK_STACKS: [[u8; TASK_STACK_SIZE]; MAX_TASKS] = [[0; TASK_STACK_SIZE]; MAX_TASKS];

pub struct Scheduler {
    tasks: [Option<Task>; MAX_TASKS],
    task_count: usize,
    current: Option<usize>,
    next_id: usize,
    idle_index: Option<usize>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            tasks: [None, None, None, None, None, None, None, None],
            task_count: 0,
            current: None,
            next_id: 0,
            idle_index: None,
        }
    }

    pub fn init(&mut self) {
        self.task_count = 0;
        self.current = None;
        self.next_id = 0;
        self.idle_index = None;

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

        self.tasks[slot] = Some(Task::new(id, name, entry, stack_top));
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

        self.tasks[slot] = Some(Task::new_idle(id, entry, stack_top));
        self.task_count += 1;
        self.idle_index = Some(slot);

        Ok(id)
    }

    pub fn current_task_id(&self) -> Option<usize> {
        self.current
            .and_then(|idx| self.tasks[idx].as_ref().map(|t| t.id.0))
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

    // -----------------------------------------------------------------------
    // Switch cooperativo via SVC
    // -----------------------------------------------------------------------

    pub fn handle_svc(&mut self, svc_num: u64, ctx: &mut ExceptionContext) {
        match svc_num {
            SVC_BOOT => {
                // Primeiro dispatch — escolhe a primeira task pronta.
                let next_idx = match self.next_ready_index(false) {
                    Some(idx) => idx,
                    None => panic!("svc boot dispatch: no runnable tasks"),
                };

                self.current = Some(next_idx);
                let task = self.tasks[next_idx].as_mut().unwrap();
                task.state = TaskState::Running;

                crate::log!(
                    "SCHED",
                    "boot dispatch id={} name={}",
                    task.id.0,
                    task.name
                );

                *ctx = task.frame;
            }

            SVC_YIELD | SVC_SLEEP | SVC_FINISHED => {
                let current_idx = match self.current {
                    Some(i) => i,
                    None => return,
                };

                let next_state = match svc_num {
                    SVC_YIELD => TaskState::Ready,
                    SVC_SLEEP => {
                        crate::log!(
                            "SCHED",
                            "task id={} sleeping forever",
                            self.tasks[current_idx].as_ref().unwrap().id.0
                        );
                        TaskState::Sleeping
                    }
                    SVC_FINISHED => {
                        crate::log!(
                            "SCHED",
                            "task id={} finished",
                            self.tasks[current_idx].as_ref().unwrap().id.0
                        );
                        TaskState::Finished
                    }
                    _ => unreachable!(),
                };

                {
                    let current = self.tasks[current_idx].as_mut().unwrap();
                    current.frame = *ctx;
                    current.state = next_state;
                }

                let include_current = next_state == TaskState::Ready;

                let next_idx = match self.next_ready_index(include_current) {
                    Some(idx) => idx,
                    None => {
                        self.tasks[current_idx].as_mut().unwrap().state = TaskState::Running;
                        return;
                    }
                };

                if next_idx == current_idx {
                    self.tasks[current_idx].as_mut().unwrap().state = TaskState::Running;
                    return;
                }

                let old_id = self.tasks[current_idx].as_ref().unwrap().id.0;
                self.current = Some(next_idx);

                let new_frame = {
                    let next = self.tasks[next_idx].as_mut().unwrap();
                    next.state = TaskState::Running;
                    next.frame
                };

                *ctx = new_frame;
            }

            _ => {
                crate::log!("SCHED", "unknown svc_num={}", svc_num);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Preempção via IRQ de timer
    // -----------------------------------------------------------------------

    pub fn preempt_from_irq(&mut self, ctx: &mut ExceptionContext) {
        let current_idx = match self.current {
            Some(i) => i,
            None => return,
        };

        {
            let current = self.tasks[current_idx].as_mut().unwrap();
            current.frame = *ctx;
            current.state = TaskState::Ready;
        }

        let next_idx = match self.next_ready_index(true) {
            Some(idx) => idx,
            None => {
                self.tasks[current_idx].as_mut().unwrap().state = TaskState::Running;
                return;
            }
        };

        if next_idx == current_idx {
            self.tasks[current_idx].as_mut().unwrap().state = TaskState::Running;
            return;
        }

        let old_id = self.tasks[current_idx].as_ref().unwrap().id.0;
        self.current = Some(next_idx);

        let new_frame = {
            let next = self.tasks[next_idx].as_mut().unwrap();
            next.state = TaskState::Running;
            next.frame
        };

        *ctx = new_frame;
    }

    // -----------------------------------------------------------------------
    // Acordar task (áudio, DMA, etc.)
    // -----------------------------------------------------------------------

    pub fn wake_task(&mut self, id: usize) -> bool {
        for slot in self.tasks.iter_mut().flatten() {
            if slot.id.0 == id && slot.state == TaskState::Sleeping {
                slot.state = TaskState::Ready;
                return true;
            }
        }
        false
    }
}

static SCHEDULER: IrqSafeSpinLock<Scheduler> = IrqSafeSpinLock::new(Scheduler::new());

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

pub fn wake_task(id: usize) -> bool {
    SCHEDULER.lock().wake_task(id)
}

pub fn yield_now() {
    unsafe {
        core::arch::asm!("svc #0", options(nomem, nostack));
    }
}

pub fn sleep_forever() -> ! {
    unsafe {
        core::arch::asm!("svc #1", options(nomem, nostack));
    }
    panic!("sleep_forever: returned from svc");
}

pub fn mark_current_finished() -> ! {
    unsafe {
        core::arch::asm!("svc #2", options(nomem, nostack));
    }
    panic!("mark_current_finished: returned from svc");
}

/// Inicia o scheduler via SVC #3.
/// Passa pelo mesmo caminho de SAVE_CONTEXT + RESTORE_CONTEXT + eret
/// que todos os outros switches — sem eret direto fora de exceção.
pub fn run() -> ! {
    crate::log!("SCHED", "scheduler started");
    unsafe {
        core::arch::asm!("svc #3", options(nomem, nostack));
    }
    panic!("scheduler run: returned from svc");
}

pub fn preempt_from_irq(ctx: &mut ExceptionContext) {
    SCHEDULER.lock().preempt_from_irq(ctx);
}

pub fn handle_svc(svc_num: u64, ctx: &mut ExceptionContext) {
    SCHEDULER.lock().handle_svc(svc_num, ctx);
}