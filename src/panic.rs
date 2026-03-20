// src/panic.rs

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    crate::log!("PANIC", "Kernel panic");

    if let Some(location) = info.location() {
        crate::log!(
            "PANIC",
            "at {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }

    crate::log!("PANIC", "message: {}", info.message());

    loop {
        core::hint::spin_loop();
    }
}