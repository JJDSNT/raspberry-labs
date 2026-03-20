// src/boot/entry.rs

#![allow(dead_code)]

use crate::boot::boot_info::BootInfo;
use crate::platform::raspi3::bootargs::apply_bootargs;
use crate::platform::raspi3::dtb::Fdt;

static mut BOOT_INFO: Option<BootInfo> = None;

#[no_mangle]
pub extern "C" fn rust_entry(dtb_ptr: usize) -> ! {
    let mut info = BootInfo::default_with_dtb(dtb_ptr);

    let bootargs = unsafe {
        match Fdt::from_ptr(dtb_ptr) {
            Some(fdt) => fdt.bootargs(),
            None => None,
        }
    };

    if let Some(args) = bootargs {
        info.cmdline = Some(args);
        apply_bootargs(args, &mut info.config, &mut info.target);
    }

    unsafe {
        BOOT_INFO = Some(info);
    }

    early_arch_init();

    crate::kernel::main::kernel_main(boot_info())
}

fn early_arch_init() {

    crate::log!("BOOT", "early_arch_init: exceptions");
    crate::arch::aarch64::exception::init();
    crate::log!("BOOT", "CurrentEL={}", crate::arch::aarch64::exception::current_el());

    crate::log!("BOOT", "early_arch_init: local irq route");
    crate::platform::raspi3::interrupts::init_core0_timer_irq();
    crate::log!(
        "BOOT",
        "core0 timer int control = {:#010x}",
        crate::platform::raspi3::interrupts::core0_timer_int_control()
    );

    crate::log!("BOOT", "early_arch_init: timer");
    crate::arch::aarch64::timer::init(100);
    crate::log!(
        "BOOT",
        "cntfrq={} cntp_ctl={:#x}",
        crate::arch::aarch64::timer::counter_frequency(),
        crate::arch::aarch64::timer::control()
    );

    crate::log!("BOOT", "early_arch_init: irq enable");
    crate::arch::aarch64::exception::enable_interrupts();
}

pub fn boot_info() -> &'static BootInfo {
    unsafe { BOOT_INFO.as_ref().expect("boot info not initialized") }
}