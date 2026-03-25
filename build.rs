// build.rs

fn main() {
    println!("cargo:rerun-if-changed=lib/tinyusb");
    println!("cargo:rerun-if-changed=src/usb/tusb_config.h");
    println!("cargo:rerun-if-changed=src/usb/hal_dwc2.c");
    println!("cargo:rerun-if-changed=src/usb/dwc2_raspi3.h");
    println!("cargo:rerun-if-changed=src/emu");

    // Para o target UEFI (aarch64-unknown-uefi) não compilamos código C:
    //   - TinyUSB e Omega2 são subsistemas bare-metal, não usados via UEFI
    //   - O linker LLD do target UEFI não aceita os link-args de whole-archive
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "uefi" {
        return;
    }

    // Substitui dwc2_bcm.h da biblioteca pelo nosso header para Pi 3 em tempo de build.
    // O submodule lib/tinyusb não é modificado no git — apenas o arquivo em disco
    // é sobrescrito durante a compilação.
    std::fs::copy(
        "src/usb/dwc2_raspi3.h",
        "lib/tinyusb/src/portable/synopsys/dwc2/dwc2_bcm.h",
    ).expect("falha ao copiar dwc2_raspi3.h -> dwc2_bcm.h");

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

    // Ativa AROS + chipset ECS se os headers gerados pelo gen_rom.py existirem
    let aros_main = std::path::Path::new("src/emu/c/omega2/memory/aros_main.h");
    let aros_ext  = std::path::Path::new("src/emu/c/omega2/memory/aros_ext.h");
    if aros_main.exists() && aros_ext.exists() {
        builder.define("USE_AROS",    None);
        builder.define("CHIPSET_ECS", None);
        println!("cargo:warning=AROS ROM headers found — building with ECS chipset");
    }

    builder.compile("tinyusb");

    // Garante inclusão completa da biblioteca mesmo com --gc-sections
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_path = format!("{}/libtinyusb.a", out_dir);
    println!("cargo:rustc-link-arg=--whole-archive");
    println!("cargo:rustc-link-arg={}", lib_path);
    println!("cargo:rustc-link-arg=--no-whole-archive");
}
