// src/kernel/task.rs

use crate::arch::aarch64::exception::ExceptionContext;

pub const MAX_TASKS: usize = 8;
pub const TASK_STACK_SIZE: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Sleeping,
    Finished,
}

pub type TaskEntry = fn();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId(pub usize);

pub struct Task {
    pub id: TaskId,
    pub name: &'static str,
    pub state: TaskState,
    pub is_idle: bool,
    pub frame: ExceptionContext,
}

// SPSR_EL1 para voltar a EL1h.
// M[3:0] = 0101 => EL1h
const SPSR_EL1H: u64 = 0b0101;

impl Task {
    pub fn new(
        id: usize,
        name: &'static str,
        entry: TaskEntry,
        stack_top: usize,
        trampoline: usize,
    ) -> Self {
        Self::build(id, name, entry, stack_top, trampoline, false)
    }

    pub fn new_idle(
        id: usize,
        entry: TaskEntry,
        stack_top: usize,
        trampoline: usize,
    ) -> Self {
        Self::build(id, "idle", entry, stack_top, trampoline, true)
    }

    fn build(
        id: usize,
        name: &'static str,
        entry: TaskEntry,
        stack_top: usize,
        trampoline: usize,
        is_idle: bool,
    ) -> Self {
        let mut frame = ExceptionContext {
            x: [0; 31],
            sp: stack_top as u64,
            elr_el1: trampoline as u64,
            spsr_el1: SPSR_EL1H,
        };

        // task_trampoline lê a entry em x19.
        frame.x[19] = entry as usize as u64;

        Self {
            id: TaskId(id),
            name,
            state: TaskState::Ready,
            is_idle,
            frame,
        }
    }
}