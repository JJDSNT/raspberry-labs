// src/arch/aarch64/context.rs

use crate::arch::aarch64::exception::ExceptionContext;

pub type CpuContext = ExceptionContext;

unsafe extern "C" {
    pub fn context_switch(old: *mut CpuContext, new: *const CpuContext);
}