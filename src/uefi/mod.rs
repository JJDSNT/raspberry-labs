// src/uefi/mod.rs
//
// Caminho de boot via UEFI EDK2 (pftf/RPi3).
//
// Usado quando compilado para o target aarch64-unknown-uefi:
//   make uefi  →  BOOTAA64.EFI  (coloque em EFI/BOOT/ no cartão SD)
//
// O firmware UEFI (RPI_EFI.fd) carrega BOOTAA64.EFI, chama efi_main,
// que loga o handoff e transita para kernel_main após ExitBootServices.
//
// O boot bare-metal (kernel8.img) continua funcional e inalterado.

pub mod types;
pub mod entry;
pub mod handoff;
pub mod be_jump;
