// src/uefi/entry.rs
//
// Ponto de entrada UEFI (PE32+ entry point).
//
// A UEFI firmware chama efi_main com:
//   image_handle  — handle da imagem EFI carregada
//   system_table  — ponteiro para EFI_SYSTEM_TABLE (válido até ExitBootServices)
//
// A convenção "efiapi" em AArch64 segue o ABI AAPCS64 padrão;
// o compilador/linker do target aarch64-unknown-uefi define efi_main
// como entry point do PE32+.

use super::types::{EfiHandle, EfiStatus, EfiSystemTable};

#[no_mangle]
pub extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> EfiStatus {
    // Repassa para o handoff — não retorna.
    super::handoff::run(image_handle, system_table)
}
