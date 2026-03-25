// src/arch/aarch64/irq.rs

use crate::arch::aarch64::exception::ExceptionContext;
use crate::platform::raspi3::interrupts;

pub fn dispatch_pending_irqs(ctx: &mut ExceptionContext) {
    // -----------------------------------------------------------------------
    // 1. Timer local do ARM (CNTP) — maior prioridade
    // -----------------------------------------------------------------------
    if crate::arch::aarch64::timer::handle_irq() {
        crate::kernel::time::on_tick();
        crate::kernel::scheduler::preempt_from_irq(ctx);
        return;
    }

    // -----------------------------------------------------------------------
    // 2. IRQs do VIC (GPU/periféricos) — bit 8 do pending local indica
    //    que há IRQ pendente no VIC
    // -----------------------------------------------------------------------
    let local_pending = interrupts::core0_irq_pending();

    if local_pending & interrupts::LOCAL_IRQ_GPU_BIT != 0 {
        dispatch_vic_irqs();
        return;
    }

    // -----------------------------------------------------------------------
    // 3. IRQ local desconhecida — log para debug
    // -----------------------------------------------------------------------
    if local_pending != 0 {
        crate::log!("IRQ", "unknown local irq pending={:#010x}", local_pending);
    }
}

fn dispatch_vic_irqs() {
    // USB HCD (DWC2) — IRQ 9. Não disponível no path UEFI (sem TinyUSB).
    #[cfg(not(target_os = "uefi"))]
    if interrupts::vic_usb_irq_pending() {
        crate::drivers::usb::handle_irq();
        return;
    }

    // IRQ do VIC não identificada — log para debug
    crate::log!(
        "IRQ",
        "unknown VIC irq pending1={:#010x}",
        crate::platform::raspi3::mmio::read(0x3F00_B204)
    );
}