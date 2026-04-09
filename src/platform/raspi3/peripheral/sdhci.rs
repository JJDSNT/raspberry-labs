// src/platform/raspi3/peripheral/sdhci.rs
// Driver polling para o controlador Arasan SDHCI/eMMC do BCM2837.
// Raspberry Pi 3B.
// Suporta SDSC (byte-addressed) e SDHC/SDXC (block-addressed).

use core::arch::asm;
use crate::kernel::sync::IrqSafeSpinLock;
use crate::platform::raspi3::{mailbox, mmio};

// ---------------------------------------------------------------------------
// Endereços base
// ---------------------------------------------------------------------------

const SDHCI_BASE: usize = 0x3F30_0000;
const GPIO_BASE: usize = 0x3F20_0000;

// SDHCI registers
const SDHCI_BLKSIZECNT: usize = SDHCI_BASE + 0x04;
const SDHCI_ARG1: usize       = SDHCI_BASE + 0x08;
const SDHCI_CMDTM: usize      = SDHCI_BASE + 0x0C;
const SDHCI_RESP0: usize      = SDHCI_BASE + 0x10;
const SDHCI_RESP1: usize      = SDHCI_BASE + 0x14;
const SDHCI_RESP2: usize      = SDHCI_BASE + 0x18;
const SDHCI_RESP3: usize      = SDHCI_BASE + 0x1C;
const SDHCI_DATA: usize       = SDHCI_BASE + 0x20;
const SDHCI_STATUS: usize     = SDHCI_BASE + 0x24;
const SDHCI_CONTROL0: usize   = SDHCI_BASE + 0x28;
const SDHCI_CONTROL1: usize   = SDHCI_BASE + 0x2C;
const SDHCI_INTERRUPT: usize  = SDHCI_BASE + 0x30;
const SDHCI_IRPT_MASK: usize  = SDHCI_BASE + 0x34;
const SDHCI_IRPT_EN: usize    = SDHCI_BASE + 0x38;

// GPIO
const GPIO_GPFSEL4: usize = GPIO_BASE + 0x10;
const GPIO_GPFSEL5: usize = GPIO_BASE + 0x14;

// ---------------------------------------------------------------------------
// Bits
// ---------------------------------------------------------------------------

// STATUS
const SR_CMD_INHIBIT: u32 = 1 << 0;
const SR_DAT_INHIBIT: u32 = 1 << 1;

// CONTROL0
const C0_HCTL_DWIDTH: u32 = 1 << 1; // 4-bit mode

// CONTROL1
const C1_CLK_INTLEN: u32 = 1 << 0;
const C1_CLK_STABLE: u32 = 1 << 1;
const C1_CLK_EN: u32     = 1 << 2;
const C1_TOUNIT_MAX: u32 = 0xE << 16;
const C1_SRST_HC: u32    = 1 << 24;
const C1_SRST_CMD: u32   = 1 << 25;
const C1_SRST_DATA: u32  = 1 << 26;

// INTERRUPT
const INT_CMD_DONE: u32  = 1 << 0;
const INT_DATA_DONE: u32 = 1 << 1;
const INT_READ_RDY: u32  = 1 << 5;
const INT_CARD: u32      = 1 << 8;

// Bits de erro ficam em [31:16]
const INT_ERR_MASK: u32          = 0xFFFF_0000;
const INT_ERR_CMD_TIMEOUT: u32   = 1 << 16;
const INT_ERR_CMD_CRC: u32       = 1 << 17;
const INT_ERR_CMD_END_BIT: u32   = 1 << 18;
const INT_ERR_CMD_INDEX: u32     = 1 << 19;
const INT_ERR_DATA_TIMEOUT: u32  = 1 << 20;
const INT_ERR_DATA_CRC: u32      = 1 << 21;
const INT_ERR_DATA_END_BIT: u32  = 1 << 22;
const INT_ERR_CURRENT_LIMIT: u32 = 1 << 23;
const INT_ERR_AUTO_CMD12: u32    = 1 << 24;
const INT_ERR_ADMA: u32          = 1 << 25;

// CMDTM
const CMD_RSPNS_NONE: u32 = 0 << 16;
const CMD_RSPNS_136: u32  = 1 << 16;
const CMD_RSPNS_48: u32   = 2 << 16;
const CMD_RSPNS_48B: u32  = 3 << 16;
const CMD_CRCCHK_EN: u32  = 1 << 19;
const CMD_IXCHK_EN: u32   = 1 << 20;
const CMD_ISDATA: u32     = 1 << 21;

const TM_BLKCNT_EN: u32   = 1 << 1;
const TM_AUTO_CMD12: u32  = 2 << 2;
const TM_DAT_READ: u32    = 1 << 4;
const TM_MULTI_BLOCK: u32 = 1 << 5;

// Comandos pré-montados
const SD_CMD0: u32  = 0  << 24 | CMD_RSPNS_NONE;
const SD_CMD2: u32  = 2  << 24 | CMD_RSPNS_136;
const SD_CMD3: u32  = 3  << 24 | CMD_RSPNS_48  | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_CMD7: u32  = 7  << 24 | CMD_RSPNS_48B | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_CMD8: u32  = 8  << 24 | CMD_RSPNS_48  | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_CMD16: u32 = 16 << 24 | CMD_RSPNS_48  | CMD_CRCCHK_EN | CMD_IXCHK_EN;

const SD_CMD17: u32 = 17 << 24
    | CMD_RSPNS_48
    | CMD_CRCCHK_EN
    | CMD_IXCHK_EN
    | CMD_ISDATA
    | TM_DAT_READ
    | TM_BLKCNT_EN;

const SD_CMD18: u32 = 18 << 24
    | CMD_RSPNS_48
    | CMD_CRCCHK_EN
    | CMD_IXCHK_EN
    | CMD_ISDATA
    | TM_DAT_READ
    | TM_BLKCNT_EN
    | TM_MULTI_BLOCK
    | TM_AUTO_CMD12;

const SD_CMD55: u32  = 55 << 24 | CMD_RSPNS_48 | CMD_CRCCHK_EN | CMD_IXCHK_EN;
const SD_ACMD41: u32 = 41 << 24 | CMD_RSPNS_48;

// ---------------------------------------------------------------------------
// Estado global
// ---------------------------------------------------------------------------

struct SdState {
    rca: u32,
    is_sdhc: bool,
}

static STATE: IrqSafeSpinLock<Option<SdState>> = IrqSafeSpinLock::new(None);

// ---------------------------------------------------------------------------
// Utilitários
// ---------------------------------------------------------------------------

#[inline]
fn delay_nop(n: u32) {
    for _ in 0..n {
        unsafe {
            asm!("nop", options(nomem, nostack, preserves_flags));
        }
    }
}

fn wait_mask(reg: usize, mask: u32, expected: u32) -> bool {
    for _ in 0..500_000u32 {
        if mmio::read(reg) & mask == expected {
            return true;
        }
        delay_nop(10);
    }
    false
}

fn interrupt_error_string(v: u32) -> &'static str {
    if v & INT_ERR_CMD_TIMEOUT != 0 {
        "cmd timeout"
    } else if v & INT_ERR_CMD_CRC != 0 {
        "cmd crc"
    } else if v & INT_ERR_CMD_END_BIT != 0 {
        "cmd end bit"
    } else if v & INT_ERR_CMD_INDEX != 0 {
        "cmd index"
    } else if v & INT_ERR_DATA_TIMEOUT != 0 {
        "data timeout"
    } else if v & INT_ERR_DATA_CRC != 0 {
        "data crc"
    } else if v & INT_ERR_DATA_END_BIT != 0 {
        "data end bit"
    } else if v & INT_ERR_CURRENT_LIMIT != 0 {
        "current limit"
    } else if v & INT_ERR_AUTO_CMD12 != 0 {
        "auto cmd12"
    } else if v & INT_ERR_ADMA != 0 {
        "adma"
    } else {
        "unknown"
    }
}

fn reset_cmd_circuit() {
    let c1 = mmio::read(SDHCI_CONTROL1);
    mmio::write(SDHCI_CONTROL1, c1 | C1_SRST_CMD);
    for _ in 0..1000u32 {
        if mmio::read(SDHCI_CONTROL1) & C1_SRST_CMD == 0 {
            break;
        }
        delay_nop(10);
    }
}

fn reset_data_circuit() {
    let c1 = mmio::read(SDHCI_CONTROL1);
    mmio::write(SDHCI_CONTROL1, c1 | C1_SRST_DATA);
    for _ in 0..1000u32 {
        if mmio::read(SDHCI_CONTROL1) & C1_SRST_DATA == 0 {
            break;
        }
        delay_nop(10);
    }
}

fn wait_interrupt(mask: u32) -> Result<u32, &'static str> {
    for _ in 0..500_000u32 {
        let raw = mmio::read(SDHCI_INTERRUPT);

        if raw & INT_CARD != 0 {
            mmio::write(SDHCI_INTERRUPT, INT_CARD);
        }
        let v = raw & !INT_CARD;

        if v & INT_ERR_MASK != 0 {
            mmio::write(SDHCI_INTERRUPT, v & INT_ERR_MASK);
            reset_cmd_circuit();
            reset_data_circuit();
            return Err(interrupt_error_string(v));
        }

        if v & mask != 0 {
            mmio::write(SDHCI_INTERRUPT, v & mask);
            return Ok(v);
        }

        delay_nop(10);
    }

    mmio::write(SDHCI_INTERRUPT, 0xFFFF_FFFF);
    reset_cmd_circuit();
    reset_data_circuit();
    Err("timeout")
}

fn gpio_setup() {
    // GPIO 48..53 -> ALT3 (SD/eMMC)
    let mut v = mmio::read(GPIO_GPFSEL4);
    v &= !(7 << 24 | 7 << 27);
    v |=   7 << 24 | 7 << 27;
    mmio::write(GPIO_GPFSEL4, v);

    let mut v = mmio::read(GPIO_GPFSEL5);
    v &= !(7 | 7 << 3 | 7 << 6 | 7 << 9);
    v |=   7 | 7 << 3 | 7 << 6 | 7 << 9;
    mmio::write(GPIO_GPFSEL5, v);

    // Não mexer em GPPUD/GPPUDCLK no Pi 3B durante o bring-up.
}

fn sdhci_base_clock() -> u32 {
    #[repr(align(16))]
    struct Buf([u32; 8]);

    let mut buf = Buf([7 * 4, 0, 0x0003_0002, 8, 0, 1, 0, 0]);
    mailbox::mailbox_call(8, buf.0.as_mut_ptr());
    if buf.0[6] == 0 { 100_000_000 } else { buf.0[6] }
}

fn set_clock(base_hz: u32, target_hz: u32) -> bool {
    if !wait_mask(SDHCI_STATUS, SR_CMD_INHIBIT | SR_DAT_INHIBIT, 0) {
        return false;
    }

    let c1 = mmio::read(SDHCI_CONTROL1) & !C1_CLK_EN;
    mmio::write(SDHCI_CONTROL1, c1);
    delay_nop(300);

    let divisor = ((base_hz + 2 * target_hz - 1) / (2 * target_hz)).clamp(1, 0x3FF);
    let cdiv = ((divisor & 0xFF) << 8) | ((divisor >> 8) << 6);

    mmio::write(
        SDHCI_CONTROL1,
        (c1 & !0xFFE0) | cdiv | C1_CLK_INTLEN | C1_TOUNIT_MAX,
    );
    delay_nop(300);

    if !wait_mask(SDHCI_CONTROL1, C1_CLK_STABLE, C1_CLK_STABLE) {
        return false;
    }

    let c1 = mmio::read(SDHCI_CONTROL1);
    mmio::write(SDHCI_CONTROL1, c1 | C1_CLK_EN);
    delay_nop(300);
    true
}

fn send_cmd(cmd: u32, arg: u32) -> Option<[u32; 4]> {
    if !wait_mask(SDHCI_STATUS, SR_CMD_INHIBIT, 0) {
        crate::log!("SDHCI", "cmd inhibit timeout");
        return None;
    }

    if cmd & CMD_ISDATA != 0 {
        if !wait_mask(SDHCI_STATUS, SR_DAT_INHIBIT, 0) {
            crate::log!("SDHCI", "data inhibit timeout");
            return None;
        }
    }

    mmio::write(SDHCI_INTERRUPT, 0xFFFF_FFFF);
    mmio::write(SDHCI_ARG1, arg);
    mmio::write(SDHCI_CMDTM, cmd);

    if let Err(e) = wait_interrupt(INT_CMD_DONE) {
        crate::log!("SDHCI", "cmd failed: {}", e);
        return None;
    }

    Some([
        mmio::read(SDHCI_RESP0),
        mmio::read(SDHCI_RESP1),
        mmio::read(SDHCI_RESP2),
        mmio::read(SDHCI_RESP3),
    ])
}

fn send_acmd(rca: u32, cmd: u32, arg: u32) -> Option<[u32; 4]> {
    send_cmd(SD_CMD55, rca << 16)?;
    send_cmd(cmd, arg)
}

// ---------------------------------------------------------------------------
// Inicialização
// ---------------------------------------------------------------------------

pub fn is_ready() -> bool {
    STATE.lock().is_some()
}

pub fn init() -> bool {
    if STATE.lock().is_some() {
        return true;
    }

    gpio_setup();

    let base_hz = sdhci_base_clock();
    crate::log!("SDHCI", "base clock {}Hz", base_hz);

    mmio::write(SDHCI_CONTROL0, 0);
    mmio::write(SDHCI_CONTROL1, C1_SRST_HC);
    if !wait_mask(SDHCI_CONTROL1, C1_SRST_HC, 0) {
        crate::log!("SDHCI", "reset timeout");
        return false;
    }

    mmio::write(SDHCI_CONTROL1, C1_CLK_INTLEN | C1_TOUNIT_MAX);
    delay_nop(300);

    if !wait_mask(SDHCI_CONTROL1, C1_CLK_STABLE, C1_CLK_STABLE) {
        crate::log!("SDHCI", "clock unstable");
        return false;
    }

    mmio::write(SDHCI_IRPT_EN, 0);
    mmio::write(SDHCI_IRPT_MASK, 0xFFFF_FFFF);
    mmio::write(SDHCI_INTERRUPT, 0xFFFF_FFFF);

    if !set_clock(base_hz, 400_000) {
        crate::log!("SDHCI", "set clock failed");
        return false;
    }

    let _ = send_cmd(SD_CMD0, 0);
    delay_nop(1000);

    let is_v2 = send_cmd(SD_CMD8, 0x0000_01AA)
        .map(|r| r[0] & 0xFFF == 0x1AA)
        .unwrap_or(false);

    let acmd41_arg = if is_v2 { 0x51FF_8000 } else { 0x00FF_8000 };
    let mut ocr = 0u32;

    for _ in 0..1000u32 {
        if let Some(r) = send_acmd(0, SD_ACMD41, acmd41_arg) {
            ocr = r[0];
            if ocr & (1 << 31) != 0 {
                break;
            }
        }
        delay_nop(1000);
    }

    if ocr & (1 << 31) == 0 {
        crate::log!("SDHCI", "ACMD41 timeout");
        return false;
    }

    let is_sdhc = is_v2 && (ocr & (1 << 30) != 0);
    crate::log!("SDHCI", "OCR={:#x} sdhc={}", ocr, is_sdhc);

    if send_cmd(SD_CMD2, 0).is_none() {
        crate::log!("SDHCI", "CMD2 failed");
        return false;
    }

    let rca = match send_cmd(SD_CMD3, 0) {
        Some(r) => (r[0] >> 16) & 0xFFFF,
        None => {
            crate::log!("SDHCI", "CMD3 failed");
            return false;
        }
    };
    crate::log!("SDHCI", "RCA={:#x}", rca);

    if !set_clock(base_hz, 400_000) {
        crate::log!("SDHCI", "transfer clock failed");
        return false;
    }

    if send_cmd(SD_CMD7, rca << 16).is_none() {
        crate::log!("SDHCI", "CMD7 failed");
        return false;
    }

    if !wait_mask(SDHCI_STATUS, SR_DAT_INHIBIT, 0) {
        crate::log!("SDHCI", "CMD7 busy timeout");
        return false;
    }

    if !is_sdhc {
        if send_cmd(SD_CMD16, 512).is_none() {
            crate::log!("SDHCI", "CMD16 failed");
            return false;
        }
    }

    // Força 1-bit durante bring-up
    let c0 = mmio::read(SDHCI_CONTROL0);
    mmio::write(SDHCI_CONTROL0, c0 & !C0_HCTL_DWIDTH);

    *STATE.lock() = Some(SdState { rca, is_sdhc });
    crate::log!("SDHCI", "initialized ok");
    true
}

// ---------------------------------------------------------------------------
// Leitura de blocos
// ---------------------------------------------------------------------------

pub fn read_blocks(lba: u32, buf: &mut [u8]) -> bool {
    if buf.len() % 512 != 0 {
        crate::log!("SDHCI", "buffer not multiple of 512");
        return false;
    }

    let state = STATE.lock();
    let st = match state.as_ref() {
        Some(s) => s,
        None => {
            crate::log!("SDHCI", "not initialized");
            return false;
        }
    };

    let block_count = (buf.len() / 512) as u32;
    let addr = if st.is_sdhc { lba } else { lba * 512 };
    let cmd = if block_count == 1 { SD_CMD17 } else { SD_CMD18 };

    if !wait_mask(SDHCI_STATUS, SR_CMD_INHIBIT | SR_DAT_INHIBIT, 0) {
        crate::log!("SDHCI", "controller busy");
        return false;
    }

    mmio::write(SDHCI_INTERRUPT, 0xFFFF_FFFF);
    mmio::write(SDHCI_BLKSIZECNT, (block_count << 16) | 512);

    if send_cmd(cmd, addr).is_none() {
        crate::log!("SDHCI", "read cmd failed");
        return false;
    }

    // INT_READ_RDY dispara uma vez por bloco de 512 bytes (tanto em QEMU
    // quanto no BCM2837 real com FIFO cheio). Espera por bloco, lê 128 words.
    for blk in 0..block_count as usize {
        if wait_interrupt(INT_READ_RDY).is_err() {
            crate::log!("SDHCI", "read timeout blk={}", blk);
            return false;
        }
        let base = blk * 512;
        for i in 0..128usize {
            let w = mmio::read(SDHCI_DATA);
            let off = base + i * 4;
            buf[off]     = (w & 0xFF) as u8;
            buf[off + 1] = ((w >> 8) & 0xFF) as u8;
            buf[off + 2] = ((w >> 16) & 0xFF) as u8;
            buf[off + 3] = ((w >> 24) & 0xFF) as u8;
        }
    }

    // Sempre espera DATA_DONE — mesmo em bloco único — para limpar o estado
    // do controller antes do próximo comando.
    if wait_interrupt(INT_DATA_DONE).is_err() {
        crate::log!("SDHCI", "data done timeout");
        return false;
    }

    true
}