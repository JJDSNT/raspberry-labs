// src/uefi/types.rs
//
// Definições brutas dos tipos UEFI — sem dependências externas.
// Ref: UEFI Specification 2.10, aligned ao ABI AArch64 (LP64).

use core::ffi::c_void;

// ─── Primitivos ───────────────────────────────────────────────────────────────

/// Handle opaco — ponteiro de 64 bits para objeto UEFI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct EfiHandle(pub *mut c_void);

unsafe impl Send for EfiHandle {}
unsafe impl Sync for EfiHandle {}

/// Código de status UEFI. Bit 63=1 → erro; 0 → sucesso ou aviso.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct EfiStatus(pub usize);

impl EfiStatus {
    pub const SUCCESS: Self = Self(0);
    pub const BUFFER_TOO_SMALL: Self = Self(0x8000000000000005);
    pub const INVALID_PARAMETER: Self = Self(0x8000000000000002);

    #[inline]
    pub fn is_success(self) -> bool { self.0 == 0 }
    #[inline]
    pub fn is_error(self) -> bool { self.0 & (1 << 63) != 0 }
}

impl core::fmt::Debug for EfiStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_success() { write!(f, "SUCCESS") }
        else { write!(f, "ERROR({:#018x})", self.0) }
    }
}

// ─── GUID ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct EfiGuid {
    pub d1: u32,
    pub d2: u16,
    pub d3: u16,
    pub d4: [u8; 8],
}

impl EfiGuid {
    /// FDT / Device Tree Blob — passado pelo firmware RPi3.
    pub const DEVICE_TREE: Self = Self {
        d1: 0xb1b621d5, d2: 0xf19c, d3: 0x41a5,
        d4: [0x83, 0x0b, 0xd9, 0x15, 0x2c, 0x69, 0xaa, 0xe0],
    };
    /// ACPI 2.0 (RSDP)
    pub const ACPI_20: Self = Self {
        d1: 0x8868e871, d2: 0xe4f1, d3: 0x11d3,
        d4: [0xbc, 0x22, 0x00, 0x80, 0xc7, 0x3c, 0x88, 0x81],
    };
    /// SMBIOS 3.x (Entry Point)
    pub const SMBIOS3: Self = Self {
        d1: 0xf2fd1544, d2: 0x9794, d3: 0x4a2c,
        d4: [0x99, 0x2e, 0xe5, 0xbb, 0xcf, 0x20, 0xe3, 0x94],
    };

    pub const LOADED_IMAGE_PROTOCOL: Self = Self {
        d1: 0x5B1B31A1, d2: 0x9562, d3: 0x11d2,
        d4: [0x8E, 0x3F, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
    };
    pub const SIMPLE_FILE_SYSTEM_PROTOCOL: Self = Self {
        d1: 0x964E5B22, d2: 0x6459, d3: 0x11d2,
        d4: [0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
    };
    pub const FILE_INFO: Self = Self {
        d1: 0x09576E92, d2: 0x6D3F, d3: 0x11D2,
        d4: [0x8E, 0x39, 0x00, 0xA0, 0xC9, 0x69, 0x72, 0x3B],
    };

    pub fn known_name(&self) -> Option<&'static str> {
        if *self == Self::DEVICE_TREE                  { Some("FDT (Device Tree Blob)") }
        else if *self == Self::ACPI_20                 { Some("ACPI 2.0") }
        else if *self == Self::SMBIOS3                 { Some("SMBIOS 3") }
        else if *self == Self::LOADED_IMAGE_PROTOCOL   { Some("LoadedImageProtocol") }
        else if *self == Self::SIMPLE_FILE_SYSTEM_PROTOCOL { Some("SimpleFileSystemProtocol") }
        else if *self == Self::FILE_INFO               { Some("FileInfo") }
        else { None }
    }
}

impl core::fmt::Debug for EfiGuid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f,
            "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
            self.d1, self.d2, self.d3,
            self.d4[0], self.d4[1],
            self.d4[2], self.d4[3], self.d4[4],
            self.d4[5], self.d4[6], self.d4[7],
        )
    }
}

// ─── Cabeçalho de tabela ─────────────────────────────────────────────────────

/// Cabeçalho comum a todas as tabelas UEFI (System, Boot, Runtime).
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EfiTableHeader {
    pub signature:   u64,   // identificador da tabela
    pub revision:    u32,   // versão UEFI: major<<16 | minor
    pub header_size: u32,   // tamanho do cabeçalho em bytes
    pub crc32:       u32,   // CRC-32 do cabeçalho + tabela
    pub reserved:    u32,
}

// Assinaturas das tabelas (bytes ASCII em little-endian u64)
pub const SIG_SYSTEM_TABLE:  u64 = 0x5453595320494249; // "IBI SYST"
pub const SIG_BOOT_SERVICES: u64 = 0x56524553544f4f42; // "BOOTSERV"
pub const SIG_RUNTIME_SVC:   u64 = 0x56524553544e5552; // "RUNTSERV"

// ─── Configuration Table ─────────────────────────────────────────────────────

/// Entrada na tabela de configuração do System Table.
#[repr(C)]
pub struct EfiConfigTable {
    pub vendor_guid:  EfiGuid,       // 16 bytes
    pub vendor_table: *mut c_void,   //  8 bytes → ponteiro para dado real
}

// ─── Memory Map ──────────────────────────────────────────────────────────────

/// Descritor de região de memória retornado por GetMemoryMap.
/// UEFI 2.10 §7.2.3 — tamanho varia (descriptor_size), tipicamente 48 bytes.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct EfiMemoryDescriptor {
    pub ty:             u32,   // tipo (EfiMemoryType)
    _pad:               u32,
    pub physical_start: u64,
    pub virtual_start:  u64,
    pub number_of_pages: u64,
    pub attribute:      u64,
}

impl EfiMemoryDescriptor {
    pub fn type_name(&self) -> &'static str {
        match self.ty {
            0  => "Reserved",
            1  => "LoaderCode",
            2  => "LoaderData",
            3  => "BootServicesCode",
            4  => "BootServicesData",
            5  => "RuntimeServicesCode",
            6  => "RuntimeServicesData",
            7  => "ConventionalMemory",
            8  => "UnusableMemory",
            9  => "AcpiReclaimMemory",
            10 => "AcpiMemoryNvs",
            11 => "MemoryMappedIo",
            12 => "MemoryMappedIoPortSpace",
            13 => "PalCode",
            14 => "PersistentMemory",
            _  => "<unknown>",
        }
    }
}

// Tipo de pool para AllocatePool
pub const EFI_LOADER_DATA: u32 = 2;

// ─── Constantes para AllocatePages / File ─────────────────────────────────────

/// AllocatePages: AllocateAddress — aloca em endereço físico específico.
pub const ALLOCATE_ADDRESS: u32 = 2;
/// Tipo de memória EfiLoaderData para AllocatePages/AllocatePool.
pub const EFI_LOADER_DATA_TYPE: u32 = 2;
/// OpenMode: read-only.
pub const FILE_MODE_READ: u64 = 0x0000000000000001;

// ─── Boot Services ───────────────────────────────────────────────────────────
//
// Layout segundo UEFI 2.10 Tabela 7.1 — AArch64 LP64 (ponteiros = 8 bytes).
// Offset de cada campo relativo ao início da estrutura:
//
//   0x00  EfiTableHeader (24 bytes)
//   0x18  RaiseTpl
//   0x20  RestoreTpl
//   0x28  AllocatePages
//   0x30  FreePages
//   0x38  GetMemoryMap        ← usamos
//   0x40  AllocatePool        ← usamos
//   0x48  FreePool
//   0x50–0x78  CreateEvent … CheckEvent  (6 entradas)
//   0x80–0xC0  InstallProtocol … InstallConfigTable  (9 entradas)
//   0xC8–0xD8  LoadImage, StartImage, Exit  (3 entradas)
//   0xE0  UnloadImage
//   0xE8  ExitBootServices    ← usamos

type EfiFnPtr = *const ();   // placeholder para funções que não chamamos

#[repr(C)]
pub struct EfiBootServices {
    pub hdr: EfiTableHeader,                           // 0x00  24 bytes

    _tpl:        [EfiFnPtr; 2],                        // 0x18, 0x20

    pub allocate_pages: unsafe extern "efiapi" fn(     // 0x28
        allocate_type: u32,
        memory_type:   u32,
        pages:         usize,
        memory:        *mut u64,
    ) -> EfiStatus,

    _free_pages: EfiFnPtr,                             // 0x30

    pub get_memory_map: unsafe extern "efiapi" fn(     // 0x38
        memory_map_size: *mut usize,
        memory_map:      *mut EfiMemoryDescriptor,
        map_key:         *mut usize,
        descriptor_size: *mut usize,
        descriptor_ver:  *mut u32,
    ) -> EfiStatus,

    pub allocate_pool: unsafe extern "efiapi" fn(      // 0x40
        pool_type: u32,
        size:      usize,
        buffer:    *mut *mut u8,
    ) -> EfiStatus,

    _free_pool:  EfiFnPtr,                             // 0x48
    _events:     [EfiFnPtr; 6],                        // 0x50–0x78

    _proto_pre:  [EfiFnPtr; 3],                        // 0x80, 0x88, 0x90

    pub handle_protocol: unsafe extern "efiapi" fn(    // 0x98
        handle:    EfiHandle,
        protocol:  *const EfiGuid,
        interface: *mut *mut c_void,
    ) -> EfiStatus,

    _proto_post: [EfiFnPtr; 5],                        // 0xA0, 0xA8, 0xB0, 0xB8, 0xC0
    _image_svc:  [EfiFnPtr; 3],                        // 0xC8, 0xD0, 0xD8
    _unload:     EfiFnPtr,                             // 0xE0

    pub exit_boot_services: unsafe extern "efiapi" fn( // 0xE8
        image_handle: EfiHandle,
        map_key:      usize,
    ) -> EfiStatus,
}

// ─── Runtime Services ────────────────────────────────────────────────────────

/// Serviços de runtime — apenas o cabeçalho é relevante para o handoff log.
#[repr(C)]
pub struct EfiRuntimeServices {
    pub hdr: EfiTableHeader,
    _svc:    [EfiFnPtr; 14],
}

// ─── System Table ─────────────────────────────────────────────────────────────
//
// Layout UEFI 2.10 §4.3 — AArch64 LP64:
//
//   0x00  EfiTableHeader     (24 bytes)
//   0x18  *FirmwareVendor    ( 8 bytes, UTF-16 string)
//   0x20  FirmwareRevision   ( 4 bytes, u32)
//   0x24  _pad               ( 4 bytes, alinhamento do próximo ponteiro)
//   0x28  ConsoleInHandle    ( 8 bytes, EfiHandle)
//   0x30  *ConIn             ( 8 bytes)
//   0x38  ConsoleOutHandle   ( 8 bytes, EfiHandle)
//   0x40  *ConOut            ( 8 bytes)
//   0x48  StdErrHandle       ( 8 bytes, EfiHandle)
//   0x50  *StdErr            ( 8 bytes)
//   0x58  *RuntimeServices   ( 8 bytes)
//   0x60  *BootServices      ( 8 bytes)
//   0x68  NumberOfTableEntries ( 8 bytes, usize)
//   0x70  *ConfigurationTable  ( 8 bytes)

#[repr(C)]
pub struct EfiSystemTable {
    pub hdr:                EfiTableHeader,          // 0x00
    pub firmware_vendor:    *const u16,              // 0x18  UTF-16
    pub firmware_revision:  u32,                     // 0x20
    _pad:                   u32,                     // 0x24
    pub con_in_handle:      EfiHandle,               // 0x28
    pub con_in:             *mut c_void,             // 0x30
    pub con_out_handle:     EfiHandle,               // 0x38
    pub con_out:            *mut c_void,             // 0x40
    pub stderr_handle:      EfiHandle,               // 0x48
    pub std_err:            *mut c_void,             // 0x50
    pub runtime_services:   *mut EfiRuntimeServices, // 0x58
    pub boot_services:      *mut EfiBootServices,    // 0x60
    pub n_tables:           usize,                   // 0x68
    pub config_table:       *mut EfiConfigTable,     // 0x70
}

// ─── EFI_LOADED_IMAGE_PROTOCOL ───────────────────────────────────────────────

/// EFI_LOADED_IMAGE_PROTOCOL — apenas os campos usados (até device_handle).
#[repr(C)]
pub struct EfiLoadedImageProtocol {
    pub revision:      u32,
    _pad:              u32,
    _parent_handle:    EfiHandle,               // 0x08
    _system_table:     *mut EfiSystemTable,     // 0x10
    pub device_handle: EfiHandle,               // 0x18
    // restante omitido — não necessário
}

// ─── EFI_SIMPLE_FILE_SYSTEM_PROTOCOL ─────────────────────────────────────────

#[repr(C)]
pub struct EfiSimpleFileSystemProtocol {
    pub revision: u64,
    pub open_volume: unsafe extern "efiapi" fn(
        this: *mut EfiSimpleFileSystemProtocol,
        root: *mut *mut EfiFileProtocol,
    ) -> EfiStatus,
}

// ─── EFI_FILE_PROTOCOL ───────────────────────────────────────────────────────

#[repr(C)]
pub struct EfiFileProtocol {
    pub revision:     u64,                                  // 0x00
    pub open: unsafe extern "efiapi" fn(                    // 0x08
        this:       *mut EfiFileProtocol,
        new_handle: *mut *mut EfiFileProtocol,
        file_name:  *const u16,
        open_mode:  u64,
        attributes: u64,
    ) -> EfiStatus,
    pub close: unsafe extern "efiapi" fn(                   // 0x10
        this: *mut EfiFileProtocol,
    ) -> EfiStatus,
    _delete:          EfiFnPtr,                             // 0x18
    pub read: unsafe extern "efiapi" fn(                    // 0x20
        this:        *mut EfiFileProtocol,
        buffer_size: *mut usize,
        buffer:      *mut u8,
    ) -> EfiStatus,
    _write:           EfiFnPtr,                             // 0x28
    _get_position:    EfiFnPtr,                             // 0x30
    pub set_position: unsafe extern "efiapi" fn(            // 0x38
        this:     *mut EfiFileProtocol,
        position: u64,
    ) -> EfiStatus,
    pub get_info: unsafe extern "efiapi" fn(                // 0x40
        this:             *mut EfiFileProtocol,
        information_type: *const EfiGuid,
        buffer_size:      *mut usize,
        buffer:           *mut u8,
    ) -> EfiStatus,
}

// ─── EFI_FILE_INFO ────────────────────────────────────────────────────────────

/// Buffer para EFI_FILE_INFO — campo FileName limitado a 64 caracteres UTF-16.
#[repr(C)]
pub struct EfiFileInfo {
    pub struct_size:   u64,
    pub file_size:     u64,
    pub physical_size: u64,
    _times:            [u8; 48],    // 3 × EFI_TIME (16 bytes cada)
    pub attribute:     u64,
    _file_name:        [u16; 64],   // até 64 chars
}

impl EfiFileInfo {
    pub fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
