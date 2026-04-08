// build.rs

use std::env;
use std::path::Path;

fn apply_common_flags(build: &mut cc::Build, endian: &str) {
    build
        .compiler("aarch64-linux-gnu-gcc")
        .flag("-ffreestanding")
        .flag("-nostdlib")
        .flag("-fno-builtin")
        .flag("-march=armv8-a")
        .flag("-mtune=cortex-a53")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        .flag("-Wall")
        .flag("-Wno-unused-parameter")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-D_FORTIFY_SOURCE=0");

    if endian == "big" {
        build.flag("-mbig-endian");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=lib/tinyusb");
    println!("cargo:rerun-if-changed=src/usb/tusb_config.h");
    println!("cargo:rerun-if-changed=src/usb/hal_dwc2.c");
    println!("cargo:rerun-if-changed=src/usb/dwc2_raspi3.h");
    println!("cargo:rerun-if-changed=src/emu");

    // AROS
    println!("cargo:rerun-if-changed=src/AROS/src/rom/kernel");
    println!("cargo:rerun-if-changed=src/AROS/src/compiler/include");

    // Para o target UEFI não compilamos código C bare-metal
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "uefi" {
        return;
    }

    let endian = env::var("CARGO_CFG_TARGET_ENDIAN").unwrap_or_default();
    let aros_sad_enabled = env::var_os("CARGO_FEATURE_AROS_SAD").is_some();

    // ------------------------------------------------------------
    // TinyUSB + Omega2
    // ------------------------------------------------------------

    std::fs::copy(
        "src/usb/dwc2_raspi3.h",
        "lib/tinyusb/src/portable/synopsys/dwc2/dwc2_bcm.h",
    )
    .expect("falha ao copiar dwc2_raspi3.h -> dwc2_bcm.h");

    let tinyusb = "lib/tinyusb/src";

    let mut builder = cc::Build::new();

    builder
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

        // Omega2 emulator — glue
        .file("src/emu/c/omega_glue.c")
        .file("src/emu/c/omega_input.c")
        .file("src/emu/c/omega_stubs.c")

        // shared
        .file("src/emu/c/omega2/debug/omega_probe.c")
        .file("src/emu/c/omega2/debug/os_debug.c")
        .file("src/emu/c/omega2/debug/emu_debug.c")
        .file("src/emu/c/omega2/shared/EventQueue.c")
        .file("src/emu/c/omega2/chipset/agnus/Scheduler.c")
        .file("src/emu/c/omega2/chipset/agnus/Beam.c")

        // Chipset hub
        .file("src/emu/c/omega2/chipset/Chipset.c")

        // memory
        .file("src/emu/c/omega2/memory/Memory.c")

        // cia
        .file("src/emu/c/omega2/chipset/cia/CIA.c")

        // agnus
        .file("src/emu/c/omega2/chipset/agnus/DMA.c")
        .file("src/emu/c/omega2/chipset/agnus/Blitter.c")
        .file("src/emu/c/omega2/chipset/agnus/Copper.c")
        .file("src/emu/c/omega2/chipset/agnus/Bitplane.c")

        // denise
        .file("src/emu/c/omega2/chipset/denise/Denise.c")

        // paula
        .file("src/emu/c/omega2/chipset/paula/Floppy.c")
        .file("src/emu/c/omega2/chipset/paula/Paula.c")

        // cpu
        .file("src/emu/c/omega2/cpu/m68kcpu.c")
        .file("src/emu/c/omega2/cpu/m68kops.c")
        .file("src/emu/c/omega2/cpu/m68kopac.c")
        .file("src/emu/c/omega2/cpu/m68kopdm.c")
        .file("src/emu/c/omega2/cpu/m68kopnz.c")
        .file("src/emu/c/omega2/cpu/m68kdasm.c")

        // Includes
        .include(tinyusb)
        .include("src/usb")
        .include(format!("{}/portable/synopsys/dwc2", tinyusb))
        .include("src/emu/c")
        .include("src/emu/c/omega2")
        .include("src/emu/c/omega2/shared")
        .include("src/emu/c/omega2/debug")
        .include("src/emu/c/omega2/chipset")
        .include("src/emu/c/omega2/chipset/agnus")
        .include("src/emu/c/omega2/chipset/cia")
        .include("src/emu/c/omega2/cpu")
        .include("src/emu/c/omega2/chipset/denise")
        .include("src/emu/c/omega2/memory")
        .include("src/emu/c/omega2/chipset/paula")
        .flag("-Wno-pointer-to-int-cast");

    apply_common_flags(&mut builder, &endian);

    let aros_main = Path::new("src/emu/c/omega2/memory/aros_main.h");
    let aros_ext = Path::new("src/emu/c/omega2/memory/aros_ext.h");
    if aros_main.exists() && aros_ext.exists() {
        builder.define("USE_AROS", None);
        builder.define("CHIPSET_ECS", None);
        println!("cargo:warning=AROS ROM headers found — building with ECS chipset");
    }

    builder.compile("tinyusb");

    // Garante inclusão completa da biblioteca mesmo com --gc-sections
    let out_dir = env::var("OUT_DIR").unwrap();
    let lib_path = format!("{}/libtinyusb.a", out_dir);
    println!("cargo:rustc-link-arg=--whole-archive");
    println!("cargo:rustc-link-arg={}", lib_path);
    println!("cargo:rustc-link-arg=--no-whole-archive");

    // ------------------------------------------------------------
    // AROS kernel.resource subset para SAD
    // Ativado com: --features aros-sad
    // ------------------------------------------------------------
    if aros_sad_enabled {
        let aros_kernel = "src/AROS/src/rom/kernel";
        let aros_include = "src/AROS/src/compiler/include";

        let mut aros = cc::Build::new();

        aros
            .file(format!("{}/_bug.c", aros_kernel))
            .file(format!("{}/bug.c", aros_kernel))
            .file(format!("{}/kernel_debug.c", aros_kernel))
            .file(format!("{}/kernel_globals.c", aros_kernel))
            .file(format!("{}/kernel_romtags.c", aros_kernel))
            .file(format!("{}/maygetchar.c", aros_kernel))
            .file(format!("{}/putchar.c", aros_kernel))

            // Includes do próprio subset
            .include(aros_kernel)
            .include("src/AROS/src")

            // SDK headers do AROS
            .include(aros_include)
            .include(format!("{}/aros", aros_include))
            .include(format!("{}/exec", aros_include))
            .include(format!("{}/proto", aros_include))
            .include(format!("{}/utility", aros_include))

            // Seus stubs / overrides locais
            .include("src")
            .include("src/include")

            .define("AROS_PORT_MC68K64", None)
            .define("USE_SERIAL_DEBUG", None)
            .define("__WORDSIZE", Some("64"));

        apply_common_flags(&mut aros, &endian);

        aros.flag("-Wno-unused-function");

        // Ainda não incluir:
        //   kernel_init.c
        //   prepareexecbase.c
        // Primeiro milestone = bug()/putchar()/maygetchar() pela serial

        aros.compile("aros_kernel_sad");
        println!("cargo:warning=AROS SAD subset enabled");
    }
}