// src/arch/aarch64/irq.rs

use crate::arch::aarch64::exception::ExceptionContext;

pub fn dispatch_pending_irqs(ctx: &mut ExceptionContext) {
    if crate::arch::aarch64::timer::handle_irq() {
        crate::kernel::time::on_tick();

        if let Some(next_ctx) = crate::kernel::scheduler::preempt_from_irq(ctx) {
            *ctx = next_ctx;
        }

        return;
    }

    let pending = crate::platform::raspi3::interrupts::core0_irq_pending();

    if pending != 0 {
        crate::log!("IRQ", "unknown local irq pending={:#010x}", pending);
    }
}