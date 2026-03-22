// build.rs

fn main() {
    println!("cargo:rerun-if-changed=lib/tinyusb");
    println!("cargo:rerun-if-changed=src/usb/tusb_config.h");
    println!("cargo:rerun-if-changed=src/usb/hal_dwc2.c");

    let tinyusb = "lib/tinyusb/src";

    let endian = std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap_or_default();

    let mut builder = cc::Build::new();

    builder
        .compiler("aarch64-linux-gnu-gcc")

        // Core TinyUSB
        .file(format!("{}/tusb.c", tinyusb))
        .file(format!("{}/common/tusb_fifo.c", tinyusb))

        // Host stack
        .file(format!("{}/host/usbh.c", tinyusb))
        .file(format!("{}/host/hub.c", tinyusb))

        // Classes host
        .file(format!("{}/class/hid/hid_host.c", tinyusb))
        .file(format!("{}/class/msc/msc_host.c", tinyusb))

        // Driver DWC2
        .file(format!("{}/portable/synopsys/dwc2/hcd_dwc2.c", tinyusb))
        .file(format!("{}/portable/synopsys/dwc2/dwc2_common.c", tinyusb))

        // HAL do Pi 3
        .file("src/usb/hal_dwc2.c")

        // Includes
        .include(tinyusb)
        .include("src/usb")
        .include(format!("{}/portable/synopsys/dwc2", tinyusb))

        // Flags
        .flag("-ffreestanding")
        .flag("-nostdlib")
        .flag("-fno-builtin")
        .flag("-march=armv8-a")
        .flag("-mtune=cortex-a53")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        .flag("-Wall")
        .flag("-Wno-unused-parameter")
        // Desabilita _FORTIFY_SOURCE — em bare metal não existe __memcpy_chk
        // que o GCC insere automaticamente em builds com otimização
        .flag("-D_FORTIFY_SOURCE=0")
        // Suprime o warning de pointer->int cast no DMA (ponteiros 64-bit
        // truncados para uint32_t — seguro no Pi 3 com RAM < 4GB)
        .flag("-Wno-pointer-to-int-cast");

    // Em builds big-endian, compila o código C no mesmo modo
    if endian == "big" {
        builder.flag("-mbig-endian");
    }

    builder.compile("tinyusb");

    // Garante inclusão completa da biblioteca mesmo com --gc-sections
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_path = format!("{}/libtinyusb.a", out_dir);
    println!("cargo:rustc-link-arg=--whole-archive");
    println!("cargo:rustc-link-arg={}", lib_path);
    println!("cargo:rustc-link-arg=--no-whole-archive");
}
