// src/platform/raspi3/emmc.rs
// Driver polling para o controlador Arasan eMMC do BCM2837.
// Suporta SDSC (byte-addressed) e SDHC/SDXC (block-addressed).

use core::arch::asm;
use crate::platform::raspi3::{mmio, mailbox};
use crate::kernel::sync::IrqSafeSpinLock;

// ---------------------------------------------------------------------------
// Endereços base
// ---------------------------------------------------------------------------

const EMMC_BASE: usize = 0x3F30_0000;
const GPIO_BASE:  usize = 0x3F20_0000;

// EMMC registers
const EMMC_ARG2:        usize = EMMC_BASE + 0x00;
const EMMC_BLKSIZECNT:  usize = EMMC_BASE + 0x04;
const EMMC_ARG1:        usize = EMMC_BASE + 0x08;
const EMMC_CMDTM:       usize = EMMC_BASE + 0x0C;
const EMMC_RESP0:       usize = EMMC_BASE + 0x10;
const EMMC_RESP1:       usize = EMMC_BASE + 0x14;
const EMMC_RESP2:       usize = EMMC_BASE + 0x18;
const EMMC_RESP3:       usize = EMMC_BASE + 0x1C;
const EMMC_DATA:        usize = EMMC_BASE + 0x20;
const EMMC_STATUS:      usize = EMMC_BASE + 0x24;
const EMMC_CONTROL0:    usize = EMMC_BASE + 0x28;
const EMMC_CONTROL1:    usize = EMMC_BASE + 0x2C;
const EMMC_INTERRUPT:   usize = EMMC_BASE + 0x30;
const EMMC_IRPT_MASK:   usize = EMMC_BASE + 0x34;
const EMMC_IRPT_EN:     usize = EMMC_BASE + 0x38;
const EMMC_CONTROL2:    usize = EMMC_BASE + 0x3C;

// GPIO
const GPIO_GPFSEL4:     usize = GPIO_BASE + 0x10;
const GPIO_GPFSEL5:     usize = GPIO_BASE + 0x14;
const GPIO_GPPUD:       usize = GPIO_BASE + 0x94;
const GPIO_GPPUDCLK1:   usize = GPIO_BASE + 0x9C; // GPIO 32-63

// ---------------------------------------------------------------------------
// Bits
// ---------------------------------------------------------------------------

// STATUS
const SR_CMD_INHIBIT:   u32 = 1 << 0;
const SR_DAT_INHIBIT:   u32 = 1 << 1;

// CONTROL1
const C1_CLK_INTLEN:    u32 = 1 << 0;
const C1_CLK_STABLE:    u32 = 1 << 1;
const C1_CLK_EN:        u32 = 1 << 2;
const C1_TOUNIT_MAX:    u32 = 0xE << 16;
const C1_SRST_HC:       u32 = 1 << 24;

// INTERRUPT
const INT_CMD_DONE:     u32 = 1 << 0;
const INT_DATA_DONE:    u32 = 1 << 1;
const INT_READ_RDY:     u32 = 1 << 5;
const INT_ERR:          u32 = 1 << 15;
const INT_ERR_MASK:     u32 = 0xFFFF_0002; // todos os bits de erro

// CMDTM
const CMD_RSPNS_NONE:   u32 = 0 << 16;
const CMD_RSPNS_136:    u32 = 1 << 16;
const CMD_RSPNS_48:     u32 = 2 << 16;
const CMD_RSPNS_48B:    u32 = 3 << 16;
const CMD_CRCCHK_EN:    u32 = 1 << 19;
const CMD_IXCHK_EN:     u32 = 1 << 20;
const CMD_ISDATA:       u32 = 1 << 21;
const TM_DAT_READ:      u32 = 1 << 4;
const TM_BLKCNT_EN:     u32 = 1 << 1;
const TM_MULTI_BLOCK:   u32 = 1 << 5;
const TM_AUTO_CMD12:    u32 = 2 << 2;

// Comandos pré-montados
const SD_CMD0:  u32 = 0 << 24 | CMD_RSPNS_NONE;
const SD_CMD2:  u32 = 2 << 24 | CMD_RSPNS_136;
const SD_CMD3:  u32 = 3 << 24 | CMD_RSPNS_48  | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_CMD7:  u32 = 7 << 24 | CMD_RSPNS_48B | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_CMD8:  u32 = 8 << 24 | CMD_RSPNS_48  | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_CMD16: u32 = 16 << 24 | CMD_RSPNS_48 | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_CMD17: u32 = 17 << 24 | CMD_RSPNS_48 | CMD_CRCCHK_EN | CMD_IXCHK_EN
                    | CMD_ISDATA | TM_DAT_READ;
const SD_CMD18: u32 = 18 << 24 | CMD_RSPNS_48 | CMD_CRCCHK_EN | CMD_IXCHK_EN
                    | CMD_ISDATA | TM_DAT_READ | TM_BLKCNT_EN | TM_MULTI_BLOCK | TM_AUTO_CMD12;
const SD_CMD55: u32 = 55 << 24 | CMD_RSPNS_48 | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_ACMD6: u32 = 6  << 24 | CMD_RSPNS_48 | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_ACMD41: u32 = 41 << 24 | CMD_RSPNS_48;  // sem CRC/IX check

// ---------------------------------------------------------------------------
// Estado global
// ---------------------------------------------------------------------------

struct EmmcState {
    rca:     u32,
    is_sdhc: bool,
}

static STATE: IrqSafeSpinLock<Option<EmmcState>> = IrqSafeSpinLock::new(None);

// ---------------------------------------------------------------------------
// Utilitários
// ---------------------------------------------------------------------------

#[inline]
fn delay_nop(n: u32) {
    for _ in 0..n {
        unsafe { asm!("nop", options(nomem, nostack, preserves_flags)); }
    }
}

fn wait_mask(reg: usize, mask: u32, expected: u32) -> bool {
    for _ in 0..500_000u32 {
        if mmio::read(reg) & mask == expected { return true; }
        delay_nop(10);
    }
    false
}

fn wait_interrupt(mask: u32) -> Result<(), &'static str> {
    for _ in 0..500_000u32 {
        let v = mmio::read(EMMC_INTERRUPT);
        if v & INT_ERR_MASK != 0 && v & mask == 0 {
            mmio::write(EMMC_INTERRUPT, 0xFFFF_FFFF);
            return Err("emmc: interrupt error");
        }
        if v & mask != 0 {
            mmio::write(EMMC_INTERRUPT, mask);
            return Ok(());
        }
        delay_nop(10);
    }
    mmio::write(EMMC_INTERRUPT, 0xFFFF_FFFF);
    Err("emmc: timeout")
}

// ---------------------------------------------------------------------------
// GPIO: pinos 48-53 em ALT3 (eMMC), pull-up em CMD+DAT
// ---------------------------------------------------------------------------

fn gpio_setup() {
    // GPFSEL4: GPIO 48 (bits 26:24) e 49 (bits 29:27) → ALT3 = 0b111
    let mut v = mmio::read(GPIO_GPFSEL4);
    v &= !(7 << 24 | 7 << 27);
    v |= 7 << 24 | 7 << 27;
    mmio::write(GPIO_GPFSEL4, v);

    // GPFSEL5: GPIO 50-53 (bits 2:0, 5:3, 8:6, 11:9) → ALT3 = 0b111
    let mut v = mmio::read(GPIO_GPFSEL5);
    v &= !(7 | 7 << 3 | 7 << 6 | 7 << 9);
    v |= 7 | 7 << 3 | 7 << 6 | 7 << 9;
    mmio::write(GPIO_GPFSEL5, v);

    // Pull-up em GPIO 49-53 (CMD + DAT0-3), pull-down em 48 (CLK)
    // Passo 1: ativa pull-up
    mmio::write(GPIO_GPPUD, 2); // pull-down para CLK
    delay_nop(150);
    mmio::write(GPIO_GPPUDCLK1, 1 << 16); // GPIO 48
    delay_nop(150);
    mmio::write(GPIO_GPPUD, 0);
    mmio::write(GPIO_GPPUDCLK1, 0);

    mmio::write(GPIO_GPPUD, 1); // pull-up
    delay_nop(150);
    // GPIO 49-53 = bits 17-21 de GPPUDCLK1
    mmio::write(GPIO_GPPUDCLK1, 0x3E << 17);
    delay_nop(150);
    mmio::write(GPIO_GPPUD, 0);
    mmio::write(GPIO_GPPUDCLK1, 0);
}

// ---------------------------------------------------------------------------
// Clock
// ---------------------------------------------------------------------------

fn emmc_base_clock() -> u32 {
    #[repr(align(16))]
    struct Buf([u32; 8]);
    let mut buf = Buf([7 * 4, 0, 0x0003_0002, 8, 0, 1, 0, 0]);
    mailbox::mailbox_call(8, buf.0.as_mut_ptr());
    if buf.0[6] == 0 { 100_000_000 } else { buf.0[6] }
}

fn set_clock(base_hz: u32, target_hz: u32) -> bool {
    if !wait_mask(EMMC_STATUS, SR_CMD_INHIBIT | SR_DAT_INHIBIT, 0) { return false; }

    // Desliga o clock SD
    let c1 = mmio::read(EMMC_CONTROL1) & !C1_CLK_EN;
    mmio::write(EMMC_CONTROL1, c1);
    delay_nop(300);

    // divisor = ceil(base / (2 * target)), máx 10 bits
    let divisor = ((base_hz + 2 * target_hz - 1) / (2 * target_hz)).clamp(1, 0x3FF);
    let cdiv = ((divisor & 0xFF) << 8) | ((divisor >> 8) << 6);

    mmio::write(EMMC_CONTROL1, (c1 & !0xFFE0) | cdiv | C1_CLK_INTLEN | C1_TOUNIT_MAX);
    delay_nop(300);

    if !wait_mask(EMMC_CONTROL1, C1_CLK_STABLE, C1_CLK_STABLE) { return false; }

    let c1 = mmio::read(EMMC_CONTROL1);
    mmio::write(EMMC_CONTROL1, c1 | C1_CLK_EN);
    delay_nop(300);
    true
}

// ---------------------------------------------------------------------------
// Envio de comando
// ---------------------------------------------------------------------------

fn send_cmd(cmd: u32, arg: u32) -> Option<[u32; 4]> {
    if !wait_mask(EMMC_STATUS, SR_CMD_INHIBIT, 0) { return None; }
    if cmd & CMD_ISDATA != 0 {
        if !wait_mask(EMMC_STATUS, SR_DAT_INHIBIT, 0) { return None; }
    }

    mmio::write(EMMC_INTERRUPT, 0xFFFF_FFFF);
    mmio::write(EMMC_ARG1, arg);
    mmio::write(EMMC_CMDTM, cmd);

    wait_interrupt(INT_CMD_DONE).ok()?;

    Some([
        mmio::read(EMMC_RESP0),
        mmio::read(EMMC_RESP1),
        mmio::read(EMMC_RESP2),
        mmio::read(EMMC_RESP3),
    ])
}

fn send_acmd(rca: u32, cmd: u32, arg: u32) -> Option<[u32; 4]> {
    send_cmd(SD_CMD55, rca << 16)?;
    send_cmd(cmd, arg)
}

// ---------------------------------------------------------------------------
// Inicialização
// ---------------------------------------------------------------------------

pub fn init() -> bool {
    gpio_setup();

    let base_hz = emmc_base_clock();
    crate::log!("EMMC", "base clock {}Hz", base_hz);

    // Reset controller
    mmio::write(EMMC_CONTROL0, 0);
    mmio::write(EMMC_CONTROL1, C1_SRST_HC);
    if !wait_mask(EMMC_CONTROL1, C1_SRST_HC, 0) {
        crate::log!("EMMC", "reset timeout");
        return false;
    }

    // Habilita clock interno, timeout máximo
    mmio::write(EMMC_CONTROL1, C1_CLK_INTLEN | C1_TOUNIT_MAX);
    delay_nop(300);
    if !wait_mask(EMMC_CONTROL1, C1_CLK_STABLE, C1_CLK_STABLE) {
        crate::log!("EMMC", "internal clock unstable");
        return false;
    }

    // Máscara de interrupções — apenas polling, sem IRQ
    mmio::write(EMMC_IRPT_EN,   0);
    mmio::write(EMMC_IRPT_MASK, 0xFFFF_FFFF);

    // Clock de identificação ~400kHz
    if !set_clock(base_hz, 400_000) {
        crate::log!("EMMC", "set clock 400kHz failed");
        return false;
    }

    // CMD0 — GO_IDLE
    send_cmd(SD_CMD0, 0);
    delay_nop(1000);

    // CMD8 — SEND_IF_COND (3.3V, check pattern 0xAA)
    let is_v2 = send_cmd(SD_CMD8, 0x0000_01AA)
        .map(|r| r[0] & 0xFF == 0xAA)
        .unwrap_or(false);

    // ACMD41 — inicializa, aguarda busy=0
    let acmd41_arg = if is_v2 { 0x51FF_8000 } else { 0x00FF_8000 };
    let mut ocr = 0u32;
    for _ in 0..1000u32 {
        if let Some(r) = send_acmd(0, SD_ACMD41, acmd41_arg) {
            ocr = r[0];
            if ocr & (1 << 31) != 0 { break; }
        }
        delay_nop(1000);
    }
    if ocr & (1 << 31) == 0 {
        crate::log!("EMMC", "ACMD41 timeout ocr={:#x}", ocr);
        return false;
    }
    let is_sdhc = is_v2 && (ocr & (1 << 30) != 0);
    crate::log!("EMMC", "OCR={:#x} sdhc={}", ocr, is_sdhc);

    // CMD2 — ALL_SEND_CID (transição para Ident)
    if send_cmd(SD_CMD2, 0).is_none() {
        crate::log!("EMMC", "CMD2 failed");
        return false;
    }

    // CMD3 — SEND_RELATIVE_ADDR → obtém RCA
    let rca = match send_cmd(SD_CMD3, 0) {
        Some(r) => (r[0] >> 16) & 0xFFFF,
        None => { crate::log!("EMMC", "CMD3 failed"); return false; }
    };
    crate::log!("EMMC", "RCA={:#x}", rca);

    // Clock de transferência ~25MHz
    if !set_clock(base_hz, 25_000_000) {
        crate::log!("EMMC", "set clock 25MHz failed");
        return false;
    }

    // CMD7 — SELECT_CARD
    if send_cmd(SD_CMD7, rca << 16).is_none() {
        crate::log!("EMMC", "CMD7 failed");
        return false;
    }

    // CMD16 — SET_BLOCKLEN (512)
    if send_cmd(SD_CMD16, 512).is_none() {
        crate::log!("EMMC", "CMD16 failed");
        return false;
    }

    // ACMD6 — SET_BUS_WIDTH 4-bit
    if send_acmd(rca, SD_ACMD6, 2).is_none() {
        crate::log!("EMMC", "ACMD6 failed (continuando em 1-bit)");
    } else {
        let c0 = mmio::read(EMMC_CONTROL0);
        mmio::write(EMMC_CONTROL0, c0 | (1 << 1)); // 4-bit bus
    }

    *STATE.lock() = Some(EmmcState { rca, is_sdhc });
    crate::log!("EMMC", "initialized ok");
    true
}

// ---------------------------------------------------------------------------
// Leitura de blocos
// ---------------------------------------------------------------------------

/// Lê `buf.len() / 512` blocos a partir do LBA `lba`.
/// `buf.len()` deve ser múltiplo de 512.
pub fn read_blocks(lba: u32, buf: &mut [u8]) -> bool {
    assert!(buf.len() % 512 == 0, "buf não é múltiplo de 512");

    let state = STATE.lock();
    let st = match state.as_ref() {
        Some(s) => s,
        None => { crate::log!("EMMC", "not initialized"); return false; }
    };

    let block_count = (buf.len() / 512) as u32;
    let addr = if st.is_sdhc { lba } else { lba * 512 };

    // CMD17 (bloco único) ou CMD18 (múltiplos)
    let cmd = if block_count == 1 { SD_CMD17 } else { SD_CMD18 };

    mmio::write(EMMC_BLKSIZECNT, (block_count << 16) | 512);
    if send_cmd(cmd, addr).is_none() {
        crate::log!("EMMC", "read cmd failed lba={}", lba);
        return false;
    }

    let words = buf.len() / 4;
    // SAFETY: buf é alinhado a &u8; lemos word a word e escrevemos como bytes
    for i in 0..words {
        if wait_interrupt(INT_READ_RDY).is_err() {
            crate::log!("EMMC", "read timeout blk={}", i / 128);
            return false;
        }
        let w = mmio::read(EMMC_DATA);
        let off = i * 4;
        buf[off]     = (w & 0xFF) as u8;
        buf[off + 1] = ((w >> 8)  & 0xFF) as u8;
        buf[off + 2] = ((w >> 16) & 0xFF) as u8;
        buf[off + 3] = ((w >> 24) & 0xFF) as u8;
    }

    if wait_interrupt(INT_DATA_DONE).is_err() {
        crate::log!("EMMC", "data done timeout");
        return false;
    }

    true
}
