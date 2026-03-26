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
pub const SVC_YIELD: u64 = 0;
pub const SVC_SLEEP: u64 = 1;
pub const SVC_FINISHED: u64 = 2;
pub const SVC_BOOT: u64 = 3;

// Preenchimento de stack para diagnóstico de overflow / high-water mark.
const STACK_POISON: u8 = 0xA5;

// Cada task ganha um stack fixo.
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

        for slot in 0..MAX_TASKS {
            Self::poison_stack(slot);
        }
    }

    #[inline]
    fn stack_base(slot: usize) -> usize {
        debug_assert!(slot < MAX_TASKS);
        unsafe { core::ptr::addr_of!(TASK_STACKS[slot]) as *const u8 as usize }
    }

    #[inline]
    fn stack_top(slot: usize) -> usize {
        let top = Self::stack_base(slot) + TASK_STACK_SIZE;
        top & !0xFusize
    }

    #[inline]
    fn poison_stack(slot: usize) {
        debug_assert!(slot < MAX_TASKS);
        unsafe {
            TASK_STACKS[slot].fill(STACK_POISON);
        }
    }

    #[inline]
    fn alloc_stack_top(&self, slot: usize) -> usize {
        assert!(slot < MAX_TASKS, "invalid stack slot");
        Self::stack_top(slot)
    }

    #[inline]
    fn validate_internal_state(&self) {
        assert!(self.task_count <= MAX_TASKS, "task_count corrupted");

        if let Some(cur) = self.current {
            assert!(cur < self.task_count, "current index corrupted");
            assert!(self.tasks[cur].is_some(), "current task missing");
        }

        if let Some(idle) = self.idle_index {
            assert!(idle < self.task_count, "idle_index corrupted");
            assert!(self.tasks[idle].is_some(), "idle task missing");
        }
    }

    #[inline]
    fn stack_bytes_used(slot: usize) -> usize {
        debug_assert!(slot < MAX_TASKS);

        unsafe {
            let stack = &TASK_STACKS[slot];
            let mut first_changed = TASK_STACK_SIZE;

            for (i, b) in stack.iter().enumerate() {
                if *b != STACK_POISON {
                    first_changed = i;
                    break;
                }
            }

            if first_changed == TASK_STACK_SIZE {
                0
            } else {
                TASK_STACK_SIZE - first_changed
            }
        }
    }

    #[inline]
    fn stack_overflow_suspected(slot: usize) -> bool {
        debug_assert!(slot < MAX_TASKS);

        unsafe {
            TASK_STACKS[slot][0] != STACK_POISON
                || TASK_STACKS[slot][1] != STACK_POISON
                || TASK_STACKS[slot][2] != STACK_POISON
                || TASK_STACKS[slot][3] != STACK_POISON
                || TASK_STACKS[slot][4] != STACK_POISON
                || TASK_STACKS[slot][5] != STACK_POISON
                || TASK_STACKS[slot][6] != STACK_POISON
                || TASK_STACKS[slot][7] != STACK_POISON
        }
    }

    #[inline]
    fn log_stack_usage(&self, slot: usize, reason: &str) {
        if slot >= self.task_count {
            return;
        }

        let Some(task) = &self.tasks[slot] else {
            return;
        };

        let used = Self::stack_bytes_used(slot);

        crate::log!(
            "SCHED",
            "stack check: reason={} id={} name={} used={} / {} bytes",
            reason,
            task.id.0,
            task.name,
            used,
            TASK_STACK_SIZE
        );

        if Self::stack_overflow_suspected(slot) {
            crate::log!(
                "SCHED",
                "WARNING: stack overflow suspected for id={} name={}",
                task.id.0,
                task.name
            );
        }
    }

    pub fn spawn(&mut self, name: &'static str, entry: TaskEntry) -> Result<usize, &'static str> {
        self.validate_internal_state();

        if self.task_count >= MAX_TASKS {
            return Err("scheduler full");
        }

        let id = self.next_id;
        self.next_id += 1;

        let slot = self.task_count;

        Self::poison_stack(slot);

        let stack_top = self.alloc_stack_top(slot);

        self.tasks[slot] = Some(Task::new(id, name, entry, stack_top));
        self.task_count += 1;

        crate::log!(
            "SCHED",
            "spawn id={} name={} slot={} stack=[0x{:X}..0x{:X}] size={}",
            id,
            name,
            slot,
            Self::stack_base(slot),
            Self::stack_top(slot),
            TASK_STACK_SIZE
        );

        Ok(id)
    }

    pub fn set_idle_task(&mut self, entry: TaskEntry) -> Result<usize, &'static str> {
        self.validate_internal_state();

        if self.idle_index.is_some() {
            return Err("idle task already set");
        }

        if self.task_count >= MAX_TASKS {
            return Err("scheduler full");
        }

        let id = self.next_id;
        self.next_id += 1;

        let slot = self.task_count;

        Self::poison_stack(slot);

        let stack_top = self.alloc_stack_top(slot);

        self.tasks[slot] = Some(Task::new_idle(id, entry, stack_top));
        self.task_count += 1;
        self.idle_index = Some(slot);

        crate::log!(
            "SCHED",
            "set idle id={} slot={} stack=[0x{:X}..0x{:X}] size={}",
            id,
            slot,
            Self::stack_base(slot),
            Self::stack_top(slot),
            TASK_STACK_SIZE
        );

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
        self.validate_internal_state();

        match svc_num {
            SVC_BOOT => {
                let next_idx = match self.next_ready_index(false) {
                    Some(idx) => idx,
                    None => panic!("svc boot dispatch: no runnable tasks"),
                };

                self.current = Some(next_idx);

                let frame = {
                    let task = self.tasks[next_idx].as_mut().unwrap();
                    task.state = TaskState::Running;

                    crate::log!(
                        "SCHED",
                        "boot dispatch id={} name={}",
                        task.id.0,
                        task.name
                    );

                    task.frame
                };

                self.log_stack_usage(next_idx, "boot");
                *ctx = frame;
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

                self.log_stack_usage(current_idx, "svc-exit");

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

                self.current = Some(next_idx);

                let new_frame = {
                    let next = self.tasks[next_idx].as_mut().unwrap();
                    next.state = TaskState::Running;
                    next.frame
                };

                self.log_stack_usage(next_idx, "svc-enter");
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
        self.validate_internal_state();

        let current_idx = match self.current {
            Some(i) => i,
            None => return,
        };

        {
            let current = self.tasks[current_idx].as_mut().unwrap();
            current.frame = *ctx;
            current.state = TaskState::Ready;
        }

        self.log_stack_usage(current_idx, "irq-exit");

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

        self.current = Some(next_idx);

        let new_frame = {
            let next = self.tasks[next_idx].as_mut().unwrap();
            next.state = TaskState::Running;
            next.frame
        };

        self.log_stack_usage(next_idx, "irq-enter");
        *ctx = new_frame;
    }

    // -----------------------------------------------------------------------
    // Acordar task (áudio, DMA, etc.)
    // -----------------------------------------------------------------------

    pub fn wake_task(&mut self, id: usize) -> bool {
        self.validate_internal_state();

        for slot in self.tasks.iter_mut().flatten() {
            if slot.id.0 == id && slot.state == TaskState::Sleeping {
                slot.state = TaskState::Ready;
                return true;
            }
        }

        false
    }

    // -----------------------------------------------------------------------
    // Debug helpers públicos
    // -----------------------------------------------------------------------

    pub fn debug_dump_stacks(&self) {
        self.validate_internal_state();

        for slot in 0..self.task_count {
            self.log_stack_usage(slot, "manual-dump");
        }
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

pub fn debug_dump_stacks() {
    SCHEDULER.lock().debug_dump_stacks();
}