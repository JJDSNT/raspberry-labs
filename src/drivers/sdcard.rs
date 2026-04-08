// src/drivers/sdcard.rs
//
// Driver de cartão SD do sistema.
//
// Este módulo é a interface que o restante do kernel deve usar.
// A implementação concreta do host controller fica na plataforma.
//
// Por enquanto:
// - plataforma: Raspberry Pi 3B
// - backend: Arasan SDHCI/eMMC host controller

/// Inicializa o cartão SD via controlador SDHCI da plataforma.
///
/// Retorna `true` em caso de sucesso.
#[inline]
pub fn init() -> bool {
    crate::platform::raspi3::peripheral::sdhci::init()
}

/// Lê `buf.len() / 512` blocos a partir do LBA informado.
///
/// Requisitos:
/// - `buf.len()` deve ser múltiplo de 512
///
/// Retorna `true` em caso de sucesso.
#[inline]
pub fn read_blocks(lba: u32, buf: &mut [u8]) -> bool {
    crate::platform::raspi3::peripheral::sdhci::read_blocks(lba, buf)
}

/// Lê exatamente um bloco de 512 bytes.
///
/// Retorna `true` em caso de sucesso.
#[inline]
pub fn read_block(lba: u32, buf: &mut [u8; 512]) -> bool {
    crate::platform::raspi3::peripheral::sdhci::read_blocks(lba, buf)
}