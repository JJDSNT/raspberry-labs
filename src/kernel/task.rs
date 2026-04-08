// src/kernel/task.rs

use crate::arch::aarch64::exception::ExceptionContext;

pub const MAX_TASKS: usize = 8;
pub const TASK_STACK_SIZE: usize = 256 * 1024; // 256KB — enough for large demo structs (Tunnel ~100KB, Parallax ~64KB) + Renderer (~6KB) + overhead

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

// SPSR_EL1 para retornar a EL1h com IRQs habilitadas.
//
// Bits [3:0] = 0b0101 => EL1h (usa SP_EL1)
// Bit  [7]   = 0      => I = 0 => IRQ não mascarada (habilitada)
// Bit  [8]   = 0      => A = 0 => SError não mascarado
// Bit  [9]   = 0      => D = 0 => Debug não mascarado
// Bit  [6]   = 0      => F = 0 => FIQ não mascarado
//
// Valor: 0x0000_0005
const SPSR_EL1H_IRQ_ENABLED: u64 = 0b0101;

impl Task {
    pub fn new(
        id: usize,
        name: &'static str,
        entry: TaskEntry,
        stack_top: usize,
    ) -> Self {
        Self::build(id, name, entry, stack_top, false)
    }

    pub fn new_idle(
        id: usize,
        entry: TaskEntry,
        stack_top: usize,
    ) -> Self {
        Self::build(id, "idle", entry, stack_top, true)
    }

    fn build(
        id: usize,
        name: &'static str,
        entry: TaskEntry,
        stack_top: usize,
        is_idle: bool,
    ) -> Self {
        let frame = ExceptionContext {
            x: [0; 31],
            sp: stack_top as u64,
            // eret vai pular diretamente para a entry da task.
            // Não há mais trampoline — IRQs são habilitadas pelo spsr.
            elr_el1: entry as usize as u64,
            spsr_el1: SPSR_EL1H_IRQ_ENABLED,
        };

        Self {
            id: TaskId(id),
            name,
            state: TaskState::Ready,
            is_idle,
            frame,
        }
    }
}