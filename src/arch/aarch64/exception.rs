// src/arch/aarch64/exception.rs

use core::arch::asm;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExceptionContext {
    pub x: [u64; 31], // x0..x30
    pub sp: u64,
    pub elr_el1: u64,
    pub spsr_el1: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct IrqState {
    daif: u64,
}

unsafe extern "C" {
    static __exception_vectors_start: u8;
}

#[inline(always)]
pub fn current_el() -> u64 {
    let el: u64;

    unsafe {
        asm!(
            "mrs {0}, CurrentEL",
            out(reg) el,
            options(nomem, nostack, preserves_flags)
        );
    }

    (el >> 2) & 0b11
}

#[inline(always)]
pub fn init() {
    let vbar = unsafe { &__exception_vectors_start as *const u8 as u64 };

    unsafe {
        match current_el() {
            1 => asm!(
                "msr VBAR_EL1, {vbar}",
                "isb",
                vbar = in(reg) vbar,
                options(nostack, preserves_flags)
            ),
            2 => asm!(
                "msr VBAR_EL2, {vbar}",
                "isb",
                vbar = in(reg) vbar,
                options(nostack, preserves_flags)
            ),
            el => panic!("exception::init: unsupported CurrentEL {}", el),
        }
    }
}

#[inline(always)]
fn read_daif() -> u64 {
    let daif: u64;

    unsafe {
        asm!(
            "mrs {0}, DAIF",
            out(reg) daif,
            options(nomem, nostack, preserves_flags)
        );
    }

    daif
}

#[inline(always)]
pub fn interrupts_enabled() -> bool {
    (read_daif() & (1 << 7)) == 0
}

#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        asm!("msr daifclr, #2", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        asm!("msr daifset, #2", options(nomem, nostack, preserves_flags));
    }
}

#[inline(always)]
pub fn save_and_disable_interrupts() -> IrqState {
    let daif = read_daif();

    unsafe {
        asm!("msr daifset, #2", options(nomem, nostack, preserves_flags));
    }

    IrqState { daif }
}

#[inline(always)]
pub fn restore_interrupts(state: IrqState) {
    if (state.daif & (1 << 7)) == 0 {
        enable_interrupts();
    } else {
        disable_interrupts();
    }
}

#[no_mangle]
extern "C" fn rust_sync_exception_current_el_sp0(ctx: &mut ExceptionContext) {
    unhandled_exception("sync current_el_sp0", ctx);
}

#[no_mangle]
extern "C" fn rust_irq_exception_current_el_sp0(ctx: &mut ExceptionContext) {
    handle_irq(ctx);
}

#[no_mangle]
extern "C" fn rust_fiq_exception_current_el_sp0(ctx: &mut ExceptionContext) {
    unhandled_exception("fiq current_el_sp0", ctx);
}

#[no_mangle]
extern "C" fn rust_serror_exception_current_el_sp0(ctx: &mut ExceptionContext) {
    unhandled_exception("serror current_el_sp0", ctx);
}

#[no_mangle]
extern "C" fn rust_sync_exception_current_el_spx(ctx: &mut ExceptionContext) {
    unhandled_exception("sync current_el_spx", ctx);
}

#[no_mangle]
extern "C" fn rust_irq_exception_current_el_spx(ctx: &mut ExceptionContext) {
    handle_irq(ctx);
}

#[no_mangle]
extern "C" fn rust_fiq_exception_current_el_spx(ctx: &mut ExceptionContext) {
    unhandled_exception("fiq current_el_spx", ctx);
}

#[no_mangle]
extern "C" fn rust_serror_exception_current_el_spx(ctx: &mut ExceptionContext) {
    unhandled_exception("serror current_el_spx", ctx);
}

#[no_mangle]
extern "C" fn rust_sync_exception_lower_el_aarch64(ctx: &mut ExceptionContext) {
    unhandled_exception("sync lower_el_aarch64", ctx);
}

#[no_mangle]
extern "C" fn rust_irq_exception_lower_el_aarch64(ctx: &mut ExceptionContext) {
    handle_irq(ctx);
}

#[no_mangle]
extern "C" fn rust_fiq_exception_lower_el_aarch64(ctx: &mut ExceptionContext) {
    unhandled_exception("fiq lower_el_aarch64", ctx);
}

#[no_mangle]
extern "C" fn rust_serror_exception_lower_el_aarch64(ctx: &mut ExceptionContext) {
    unhandled_exception("serror lower_el_aarch64", ctx);
}

#[no_mangle]
extern "C" fn rust_sync_exception_lower_el_aarch32(ctx: &mut ExceptionContext) {
    unhandled_exception("sync lower_el_aarch32", ctx);
}

#[no_mangle]
extern "C" fn rust_irq_exception_lower_el_aarch32(ctx: &mut ExceptionContext) {
    handle_irq(ctx);
}

#[no_mangle]
extern "C" fn rust_fiq_exception_lower_el_aarch32(ctx: &mut ExceptionContext) {
    unhandled_exception("fiq lower_el_aarch32", ctx);
}

#[no_mangle]
extern "C" fn rust_serror_exception_lower_el_aarch32(ctx: &mut ExceptionContext) {
    unhandled_exception("serror lower_el_aarch32", ctx);
}

#[inline(never)]
fn handle_irq(ctx: &mut ExceptionContext) {
    crate::arch::aarch64::irq::dispatch_pending_irqs(ctx);
}

fn unhandled_exception(kind: &str, ctx: &ExceptionContext) -> ! {
    let daif = read_daif();
    let el = current_el();

    match el {
        1 => {
            let esr: u64;
            let far: u64;

            unsafe {
                asm!(
                    "mrs {0}, ESR_EL1",
                    out(reg) esr,
                    options(nomem, nostack, preserves_flags)
                );
                asm!(
                    "mrs {0}, FAR_EL1",
                    out(reg) far,
                    options(nomem, nostack, preserves_flags)
                );
            }

            panic!(
                concat!(
                    "unhandled exception: {}\n",
                    "CurrentEL={}\n",
                    "ESR_EL1={:#018x}\n",
                    "FAR_EL1={:#018x}\n",
                    "DAIF={:#018x}\n",
                    "ELR(saved)={:#018x}\n",
                    "SPSR(saved)={:#018x}\n",
                    "SP={:#018x}\n",
                    "X0={:#018x}\n",
                    "X1={:#018x}\n",
                    "X2={:#018x}\n",
                    "X3={:#018x}"
                ),
                kind,
                el,
                esr,
                far,
                daif,
                ctx.elr_el1,
                ctx.spsr_el1,
                ctx.sp,
                ctx.x[0],
                ctx.x[1],
                ctx.x[2],
                ctx.x[3],
            );
        }
        2 => {
            let esr: u64;
            let far: u64;
            let elr: u64;
            let spsr: u64;

            unsafe {
                asm!(
                    "mrs {0}, ESR_EL2",
                    out(reg) esr,
                    options(nomem, nostack, preserves_flags)
                );
                asm!(
                    "mrs {0}, FAR_EL2",
                    out(reg) far,
                    options(nomem, nostack, preserves_flags)
                );
                asm!(
                    "mrs {0}, ELR_EL2",
                    out(reg) elr,
                    options(nomem, nostack, preserves_flags)
                );
                asm!(
                    "mrs {0}, SPSR_EL2",
                    out(reg) spsr,
                    options(nomem, nostack, preserves_flags)
                );
            }

            panic!(
                concat!(
                    "unhandled exception: {}\n",
                    "CurrentEL={}\n",
                    "ESR_EL2={:#018x}\n",
                    "FAR_EL2={:#018x}\n",
                    "DAIF={:#018x}\n",
                    "ELR_EL2={:#018x}\n",
                    "SPSR_EL2={:#018x}\n",
                    "SP={:#018x}\n",
                    "X0={:#018x}\n",
                    "X1={:#018x}\n",
                    "X2={:#018x}\n",
                    "X3={:#018x}"
                ),
                kind,
                el,
                esr,
                far,
                daif,
                elr,
                spsr,
                ctx.sp,
                ctx.x[0],
                ctx.x[1],
                ctx.x[2],
                ctx.x[3],
            );
        }
        _ => {
            panic!(
                concat!(
                    "unhandled exception: {}\n",
                    "CurrentEL={}\n",
                    "DAIF={:#018x}\n",
                    "SP={:#018x}\n",
                    "X0={:#018x}\n",
                    "X1={:#018x}\n",
                    "X2={:#018x}\n",
                    "X3={:#018x}"
                ),
                kind,
                el,
                daif,
                ctx.sp,
                ctx.x[0],
                ctx.x[1],
                ctx.x[2],
                ctx.x[3],
            );
        }
    }
}