// src/uefi/handoff.rs
//
// Ponto de handoff UEFI → kernel bare-metal.
//
// Fluxo:
//   1. Inicializa UART (acesso direto ao hardware)
//   2. Loga toda informação do SystemTable que a UEFI está entregando
//   3. Obtém o memory map e chama ExitBootServices
//   4. Constrói BootInfo a partir de dados UEFI (DTB via ConfigTable)
//   5. Chama early_arch_init() e kernel_main() — idêntico ao boot bare-metal

use super::types::*;
use crate::boot::boot_info::BootInfo;

// ─── Ponto de entrada do handoff ─────────────────────────────────────────────

pub fn run(image_handle: EfiHandle, system_table: *mut EfiSystemTable) -> ! {
    // Inicializa UART diretamente (PL011 @ 0x3F201000).
    // UEFI pode estar usando o mesmo UART para seu ConOut; ao reinicializar
    // assumimos controle do hardware. Isso é intencional — logo chamaremos
    // ExitBootServices e o ConOut UEFI se tornará inválido de qualquer forma.
    crate::kernel::console::init();

    crate::log!("UEFI", "╔══════════════════════════════════════════╗");
    crate::log!("UEFI", "║       HANDOFF POINT REACHED              ║");
    crate::log!("UEFI", "╚══════════════════════════════════════════╝");

    // Safety: system_table é garantido válido pela UEFI firmware.
    let st = unsafe { &*system_table };

    log_system_table(st);

    // Procura DTB na tabela de configuração antes de sair dos boot services.
    let dtb_ptr = unsafe { find_dtb(st) };

    // Tenta carregar kernel8-be.img para boot UEFI+BE.
    // Deve ocorrer ANTES de ExitBootServices (precisa do File System Protocol).
    let be_kernel_size = unsafe { try_load_be_kernel(image_handle, st) };

    // Última sequência com boot services: GetMemoryMap → ExitBootServices.
    // Nenhuma chamada UEFI pode ocorrer entre GetMemoryMap e ExitBootServices.
    let map_key = unsafe { get_memory_map_key(st) };

    crate::log!("UEFI", "calling ExitBootServices (map_key={:#x})...", map_key);

    let status = unsafe {
        ((*st.boot_services).exit_boot_services)(image_handle, map_key)
    };

    if !status.is_success() {
        // map_key ficou obsoleto — loop de segurança. Não tentamos retry
        // pois qualquer nova chamada UEFI poderia corromper o estado.
        crate::log!("UEFI", "ExitBootServices FAILED: {:?}", status);
        crate::log!("UEFI", "mapa de memoria ficou obsoleto — halt");
        loop { core::hint::spin_loop(); }
    }

    // ── A partir daqui: UEFI Boot Services inexistentes ──────────────────────
    // Apenas acesso direto ao hardware é permitido.

    crate::log!("UEFI", "boot services exited — bare-metal a partir daqui");
    crate::log!("UEFI", "dtb @ {:?}", dtb_ptr);

    // ── Modo UEFI+BE: kernel BE carregado → flush + switch + jump ────────────
    if let Some(size) = be_kernel_size {
        crate::log!("UEFI", "BE: kernel8-be.img ({} bytes) @ 0x80000", size);
        crate::log!("UEFI", "BE: flush cache → EE=1 → jump...");
        unsafe { crate::uefi::be_jump::switch_be_and_jump(0x80000, 0x80000, size); }
    }

    // Constrói BootInfo a partir do DTB encontrado na config table UEFI.
    let mut info = BootInfo::default_with_dtb(dtb_ptr.unwrap_or(0));

    if let Some(dtb) = dtb_ptr {
        let bootargs = unsafe {
            crate::platform::raspi3::dtb::Fdt::from_ptr(dtb)
                .and_then(|fdt| fdt.bootargs())
        };
        if let Some(args) = bootargs {
            crate::log!("UEFI", "cmdline: {}", args);
            info.cmdline = Some(args);
            crate::platform::raspi3::bootargs::apply_bootargs(
                args, &mut info.config, &mut info.target,
            );
        }
    }

    crate::boot::entry::set_boot_info(info);

    crate::log!("UEFI", "transitioning to early_arch_init...");
    crate::boot::entry::early_arch_init();

    crate::kernel::main::kernel_main(crate::boot::entry::boot_info())
}

// ─── Log do System Table ──────────────────────────────────────────────────────

fn log_system_table(st: &EfiSystemTable) {
    crate::log!("UEFI", "── SystemTable ─────────────────────────────");
    crate::log!("UEFI", "  hdr.signature   = {:#018x}", st.hdr.signature);
    crate::log!("UEFI", "  hdr.revision    = {:#010x}  (UEFI {}.{})",
        st.hdr.revision,
        st.hdr.revision >> 16,
        st.hdr.revision & 0xFFFF,
    );
    crate::log!("UEFI", "  hdr.header_size = {} bytes", st.hdr.header_size);
    crate::log!("UEFI", "  hdr.crc32       = {:#010x}", st.hdr.crc32);

    crate::print!("[UEFI]   FirmwareVendor  = \"");
    print_utf16(st.firmware_vendor);
    crate::println!("\"");

    crate::log!("UEFI", "  FirmwareRevision = {:#010x}  ({}.{})",
        st.firmware_revision,
        st.firmware_revision >> 16,
        st.firmware_revision & 0xFFFF,
    );

    log_boot_services(st);
    log_runtime_services(st);
    log_config_tables(st);
}

fn log_boot_services(st: &EfiSystemTable) {
    let bs_ptr = st.boot_services;
    crate::log!("UEFI", "── BootServices @ {:p} ──────────────────────", bs_ptr);

    if bs_ptr.is_null() { crate::log!("UEFI", "  <null>"); return; }

    let bs = unsafe { &*bs_ptr };
    crate::log!("UEFI", "  hdr.signature = {:#018x}", bs.hdr.signature);
    crate::log!("UEFI", "  hdr.revision  = {:#010x}", bs.hdr.revision);
    crate::log!("UEFI", "  hdr.size      = {} bytes", bs.hdr.header_size);

    // Endereços das funções que usamos — úteis para verificar integridade.
    crate::log!("UEFI", "  GetMemoryMap     @ {:p}",
        bs.get_memory_map as *const ());
    crate::log!("UEFI", "  AllocatePool     @ {:p}",
        bs.allocate_pool as *const ());
    crate::log!("UEFI", "  ExitBootServices @ {:p}",
        bs.exit_boot_services as *const ());
}

fn log_runtime_services(st: &EfiSystemTable) {
    let rs_ptr = st.runtime_services;
    crate::log!("UEFI", "── RuntimeServices @ {:p} ───────────────────", rs_ptr);

    if rs_ptr.is_null() { crate::log!("UEFI", "  <null>"); return; }

    let rs = unsafe { &*rs_ptr };
    crate::log!("UEFI", "  hdr.signature = {:#018x}", rs.hdr.signature);
    crate::log!("UEFI", "  hdr.revision  = {:#010x}", rs.hdr.revision);
    crate::log!("UEFI", "  hdr.size      = {} bytes", rs.hdr.header_size);
    crate::log!("UEFI", "  (runtime services permanecem válidos após ExitBootServices)");
}

fn log_config_tables(st: &EfiSystemTable) {
    crate::log!("UEFI", "── ConfigurationTables ({} entries) ─────────", st.n_tables);

    for i in 0..st.n_tables {
        let entry = unsafe { &*st.config_table.add(i) };
        let guid = &entry.vendor_guid;
        let ptr = entry.vendor_table;

        match guid.known_name() {
            Some(name) => crate::log!("UEFI", "  [{:2}] {:?}  =  {}  @ {:p}",
                i, guid, name, ptr),
            None => crate::log!("UEFI", "  [{:2}] {:?}  @ {:p}",
                i, guid, ptr),
        }
    }
}

// ─── Procura DTB na ConfigTable ──────────────────────────────────────────────

/// Retorna o endereço do DTB se encontrado na tabela de configuração UEFI.
unsafe fn find_dtb(st: &EfiSystemTable) -> Option<usize> {
    for i in 0..st.n_tables {
        let entry = &*st.config_table.add(i);
        if entry.vendor_guid == EfiGuid::DEVICE_TREE {
            return Some(entry.vendor_table as usize);
        }
    }
    None
}

// ─── Memory Map ──────────────────────────────────────────────────────────────

// Buffer de stack para o memory map.
// RPi3 tipicamente tem 20-40 regiões; 8 KiB é folgado (50 × 48 bytes = 2400).
const MAP_BUFFER_SIZE: usize = 8192;

/// Chama GetMemoryMap, loga o mapa e retorna o map_key para ExitBootServices.
/// Nenhuma chamada UEFI deve ocorrer entre este retorno e ExitBootServices.
unsafe fn get_memory_map_key(st: &EfiSystemTable) -> usize {
    let bs = &*st.boot_services;

    let mut buf = [0u8; MAP_BUFFER_SIZE];
    let mut map_size   = MAP_BUFFER_SIZE;
    let mut map_key    = 0usize;
    let mut desc_size  = 0usize;
    let mut desc_ver   = 0u32;

    let status = (bs.get_memory_map)(
        &mut map_size,
        buf.as_mut_ptr() as *mut EfiMemoryDescriptor,
        &mut map_key,
        &mut desc_size,
        &mut desc_ver,
    );

    if status.is_success() && desc_size > 0 {
        log_memory_map(&buf, map_size, desc_size);
    } else {
        crate::log!("UEFI", "GetMemoryMap: {:?} (map_size={}, desc_size={})",
            status, map_size, desc_size);
    }

    map_key
}

fn log_memory_map(buf: &[u8], map_size: usize, desc_size: usize) {
    let n_entries = map_size / desc_size;
    crate::log!("UEFI", "── Memory Map ({} entries, desc_size={}) ────", n_entries, desc_size);

    let max_shown = 48usize; // mostra até 48 entradas (todas em RPi3)
    let shown = n_entries.min(max_shown);

    for i in 0..shown {
        let offset = i * desc_size;
        if offset + core::mem::size_of::<EfiMemoryDescriptor>() > buf.len() {
            break;
        }
        let desc = unsafe {
            &*(buf.as_ptr().add(offset) as *const EfiMemoryDescriptor)
        };
        let size_kb = desc.number_of_pages * 4; // 1 page = 4 KiB
        crate::log!("UEFI", "  [{:2}] {:<24}  phys={:#011x}  {:5} pages ({} KiB)",
            i,
            desc.type_name(),
            desc.physical_start,
            desc.number_of_pages,
            size_kb,
        );
    }

    if n_entries > max_shown {
        crate::log!("UEFI", "  ... ({} entries omitted)", n_entries - max_shown);
    }
}

// ─── Loader do kernel BE ─────────────────────────────────────────────────────

/// Tenta carregar kernel8-be.img em 0x80000 via EFI Simple File System.
///
/// Retorna Some(size) se o arquivo foi encontrado e carregado com sucesso.
/// Retorna None silenciosamente se o arquivo não existe (boot LE normal).
/// Loga erros inesperados mas não faz panic.
///
/// Deve ser chamada ANTES de ExitBootServices.
unsafe fn try_load_be_kernel(
    image_handle: EfiHandle,
    st: &EfiSystemTable,
) -> Option<usize> {
    use core::ffi::c_void;
    let bs = &*st.boot_services;

    // 1. LoadedImageProtocol → device_handle do volume de boot
    let mut loaded_img: *mut EfiLoadedImageProtocol = core::ptr::null_mut();
    let s = (bs.handle_protocol)(
        image_handle,
        &EfiGuid::LOADED_IMAGE_PROTOCOL,
        &mut loaded_img as *mut *mut EfiLoadedImageProtocol as *mut *mut c_void,
    );
    if !s.is_success() {
        crate::log!("UEFI", "BE: LoadedImageProtocol: {:?} — skip", s);
        return None;
    }

    let dev = (*loaded_img).device_handle;

    // 2. SimpleFileSystemProtocol no device
    let mut fs: *mut EfiSimpleFileSystemProtocol = core::ptr::null_mut();
    let s = (bs.handle_protocol)(
        dev,
        &EfiGuid::SIMPLE_FILE_SYSTEM_PROTOCOL,
        &mut fs as *mut *mut EfiSimpleFileSystemProtocol as *mut *mut c_void,
    );
    if !s.is_success() {
        crate::log!("UEFI", "BE: SimpleFileSystem: {:?} — skip", s);
        return None;
    }

    // 3. Abrir volume raiz
    let mut root: *mut EfiFileProtocol = core::ptr::null_mut();
    let s = ((*fs).open_volume)(fs, &mut root);
    if !s.is_success() {
        crate::log!("UEFI", "BE: open_volume: {:?}", s);
        return None;
    }

    // 4. Abrir "kernel8-be.img" (UTF-16LE)
    const FNAME: &[u16] = &[
        b'k' as u16, b'e' as u16, b'r' as u16, b'n' as u16,
        b'e' as u16, b'l' as u16, b'8' as u16, b'-' as u16,
        b'b' as u16, b'e' as u16, b'.' as u16,
        b'i' as u16, b'm' as u16, b'g' as u16, 0u16,
    ];

    let mut file: *mut EfiFileProtocol = core::ptr::null_mut();
    let s = ((*root).open)(root, &mut file, FNAME.as_ptr(), FILE_MODE_READ, 0);
    if !s.is_success() {
        // Arquivo ausente = boot LE normal; não é erro.
        crate::log!("UEFI", "BE: kernel8-be.img nao encontrado — boot LE");
        let _ = ((*root).close)(root);
        return None;
    }

    crate::log!("UEFI", "BE: kernel8-be.img encontrado — carregando...");

    // 5. GetInfo → tamanho do arquivo
    let mut info = EfiFileInfo::zeroed();
    let mut info_sz = core::mem::size_of::<EfiFileInfo>();
    let s = ((*file).get_info)(
        file,
        &EfiGuid::FILE_INFO,
        &mut info_sz,
        &mut info as *mut EfiFileInfo as *mut u8,
    );
    if !s.is_success() {
        crate::log!("UEFI", "BE: GetInfo: {:?}", s);
        let _ = ((*file).close)(file);
        let _ = ((*root).close)(root);
        return None;
    }

    let file_size = info.file_size as usize;
    crate::log!("UEFI", "BE: tamanho = {} bytes ({} KiB)", file_size, file_size / 1024);

    // 6. AllocatePages em 0x80000 (mesmo endereço do boot bare-metal)
    const KERNEL_PHYS: u64 = 0x80000;
    let pages = (file_size + 4095) / 4096;
    let mut addr: u64 = KERNEL_PHYS;
    let s = (bs.allocate_pages)(ALLOCATE_ADDRESS, EFI_LOADER_DATA_TYPE, pages, &mut addr);
    if !s.is_success() {
        crate::log!("UEFI", "BE: AllocatePages @ {:#x}: {:?}", KERNEL_PHYS, s);
        let _ = ((*file).close)(file);
        let _ = ((*root).close)(root);
        return None;
    }

    // 7. Posicionar no início + ler
    let _ = ((*file).set_position)(file, 0);

    let dest = KERNEL_PHYS as *mut u8;
    let mut read_sz = file_size;
    let s = ((*file).read)(file, &mut read_sz, dest);

    let _ = ((*file).close)(file);
    let _ = ((*root).close)(root);

    if !s.is_success() || read_sz != file_size {
        crate::log!("UEFI", "BE: Read: {:?} ({}/{} bytes)", s, read_sz, file_size);
        return None;
    }

    crate::log!("UEFI", "BE: carregado em {:#x} — {} bytes OK", KERNEL_PHYS, read_sz);
    Some(read_sz)
}

// ─── Utilitário: UTF-16 → UART ───────────────────────────────────────────────

/// Imprime uma string UTF-16LE via UART, convertendo apenas ASCII imprimível.
fn print_utf16(ptr: *const u16) {
    if ptr.is_null() {
        crate::print!("<null>");
        return;
    }
    for i in 0..128usize {
        let c = unsafe { ptr.add(i).read_volatile() };
        if c == 0 { break; }
        let byte = if c < 0x80 { c as u8 } else { b'?' };
        if byte >= 0x20 {
            crate::print!("{}", byte as char);
        }
    }
}
