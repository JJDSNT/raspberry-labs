// src/kernel/main.rs

use crate::boot::boot_info::BootInfo;
use crate::demos::run_demo;
use crate::diagnostics::run_diag;
use crate::drivers::framebuffer::Framebuffer;
use crate::platform::raspi3::bootargs::BootTarget;

fn idle_task() {
    crate::log!("TASK", "idle task entered");

    loop {
        core::hint::spin_loop();
    }
}

fn scheduler_probe_task() {
    crate::log!("TEST", "scheduler probe task entered");

    let mut n: u64 = 0;

    loop {
        if n % 100 == 0 {
            crate::log!(
                "TEST",
                "probe {} pending={:#010x} cntp_ctl={:#x} ticks={} irq_enabled={}",
                n,
                crate::platform::raspi3::interrupts::core0_irq_pending(),
                crate::arch::aarch64::timer::control(),
                crate::kernel::time::ticks(),
                crate::arch::aarch64::exception::interrupts_enabled(),
            );
        }

        n += 1;
        crate::kernel::scheduler::yield_now();
    }
}

fn boot_task() {
    crate::log!("TASK", "boot task entered");

    let info = crate::boot::entry::boot_info();

    let config = crate::kernel::init::normalize_config(info.config);
    let target = info.target;

    crate::log!(
        "BOOT",
        "Config: {}x{}x{}",
        config.width,
        config.height,
        config.depth
    );

    crate::log!("BOOT", "Initializing framebuffer...");
    match Framebuffer::init(config.width, config.height, config.depth) {
        Some(fb) => {
            crate::log!("BOOT", "Framebuffer ready");
            crate::log!("BOOT", "Resolution: {}x{}", fb.width, fb.height);
            crate::log!("BOOT", "Pitch: {}", fb.pitch);
            crate::log!("BOOT", "Depth: {}", fb.depth);
            crate::log!("BOOT", "RGB order: {}", fb.isrgb);

            match target {
                BootTarget::Diag(d) => {
                    crate::log!("BOOT", "Selected diag: {}", d.as_str());
                    run_diag(d, fb);
                }
                BootTarget::Demo(d) => {
                    crate::log!("BOOT", "Selected demo: {}", d.as_str());
                    run_demo(d, fb);
                }
            }
        }
        None => {
            crate::log!("BOOT", "Framebuffer init failed");
            crate::kernel::scheduler::sleep_forever();
        }
    }
}

fn spawn_runtime_self_test() {
    crate::log!("TEST", "spawning runtime self-test task");

    crate::kernel::scheduler::spawn("sched-probe", scheduler_probe_task)
        .expect("failed to spawn sched-probe");
}

pub fn kernel_main(info: &BootInfo) -> ! {
    crate::kernel::init::early_init(info);
    crate::kernel::init::init_tasking();

    crate::kernel::scheduler::set_idle_task(idle_task)
        .expect("failed to register idle task");

    spawn_runtime_self_test();

    // Periféricos de armazenamento e I/O
    crate::platform::raspi3::emmc::init();
    #[cfg(not(target_os = "uefi"))]
    crate::drivers::usb::init();

    crate::kernel::scheduler::spawn("boot", boot_task)
        .expect("failed to spawn boot task");

    crate::kernel::scheduler::run()
}