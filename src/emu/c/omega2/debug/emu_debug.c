// emu_debug.c — DMA debugger, Copper debugger, Memory inspector
//
// No stdlib; all output via omega_host_log.

#include "emu_debug.h"
#include "omega_host.h"
#include "Chipset.h"
#include "Memory.h"
#include "Scheduler.h"

extern Chipset_t* ChipsetState;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

static const char HEX[] = "0123456789ABCDEF";

// Read a big-endian 16-bit chipset register from chipram.
// BigEndianWrite stores byte-swapped; reverse to get the Amiga value.
static uint16_t creg16(uint32_t addr) {
    if (addr + 1 >= 0x1000000) return 0;
    uint16_t raw = *(const uint16_t*)&RAM24bit[addr];
    return (uint16_t)((raw >> 8) | (raw << 8));
}

static uint32_t creg32(uint32_t addr) {
    return ((uint32_t)creg16(addr) << 16) | creg16(addr + 2);
}

static int put_str(char *b, int p, const char *s) {
    while (*s) b[p++] = *s++;
    return p;
}
static int put_hex8(char *b, int p, uint8_t v)  {
    b[p++]=HEX[(v>>4)&0xF]; b[p++]=HEX[v&0xF]; return p;
}
static int put_hex16(char *b, int p, uint16_t v) {
    b[p++]=HEX[(v>>12)&0xF]; b[p++]=HEX[(v>>8)&0xF];
    b[p++]=HEX[(v>>4)&0xF];  b[p++]=HEX[v&0xF]; return p;
}
static int put_hex32(char *b, int p, uint32_t v) {
    for (int s=28; s>=0; s-=4) { b[p++]=HEX[(v>>s)&0xF]; }
    return p;
}

static void log_line(const char *s) { omega_host_log(s); }

// ---------------------------------------------------------------------------
// DMA debugger
// ---------------------------------------------------------------------------

// DMACONR bit names (bit 15..0)
static const char *dma_bit_name(int bit) {
    switch (bit) {
    case 9:  return "DMAEN";
    case 8:  return "BPLEN";
    case 7:  return "COPEN";
    case 6:  return "BLTEN";
    case 5:  return "SPREN";
    case 4:  return "DSKEN";
    case 3:  return "AUD3E";
    case 2:  return "AUD2E";
    case 1:  return "AUD1E";
    case 0:  return "AUD0E";
    default: return NULL;
    }
}

void emu_debug_dma(void) {
    log_line("[DMA] ---- DMA state ----");

    // DMACONR — use ChipsetState which is kept current
    uint16_t dcon = ChipsetState->DMACONR;
    {
        char buf[80];
        int p = put_str(buf, 0, "[DMA] DMACONR=0x");
        p = put_hex16(buf, p, dcon);
        p = put_str(buf, p, "  enabled:");
        for (int bit = 9; bit >= 0; bit--) {
            if (dcon & (1u << bit)) {
                const char *n = dma_bit_name(bit);
                if (n) { buf[p++] = ' '; p = put_str(buf, p, n); }
            }
        }
        buf[p] = '\0';
        log_line(buf);
    }

    // Beam position
    {
        char buf[48];
        int p = put_str(buf, 0, "[DMA] Beam  V=0x");
        p = put_hex16(buf, p, (uint16_t)(ChipsetState->VHPOS >> 8));
        p = put_str(buf, p, " H=0x");
        p = put_hex16(buf, p, (uint16_t)(ChipsetState->VHPOS & 0xFF));
        p = put_str(buf, p, " clk=");
        p = put_hex32(buf, p, (uint32_t)sched_clock());
        buf[p] = '\0';
        log_line(buf);
    }

    // Floppy DMA pointer
    {
        char buf[48];
        int p = put_str(buf, 0, "[DMA] DSKPTR=0x");
        p = put_hex32(buf, p, creg32(0xDFF020));
        p = put_str(buf, p, " DSKLEN=0x");
        p = put_hex16(buf, p, creg16(0xDFF024));
        buf[p] = '\0';
        log_line(buf);
    }

    // Bitplane pointers
    static const uint32_t bpl_base[] = {
        0xDFF0E0, 0xDFF0E4, 0xDFF0E8, 0xDFF0EC, 0xDFF0F0, 0xDFF0F4
    };
    for (int i = 0; i < 6; i++) {
        char buf[48];
        int p = put_str(buf, 0, "[DMA] BPL");
        buf[p++] = (char)('1' + i);
        p = put_str(buf, p, "PT=0x");
        p = put_hex32(buf, p, creg32(bpl_base[i]));
        buf[p] = '\0';
        log_line(buf);
    }

    // Audio pointers
    static const uint32_t aud_base[] = {
        0xDFF0A0, 0xDFF0B0, 0xDFF0C0, 0xDFF0D0
    };
    for (int i = 0; i < 4; i++) {
        char buf[64];
        int p = put_str(buf, 0, "[DMA] AUD");
        buf[p++] = (char)('0' + i);
        p = put_str(buf, p, "LC=0x");
        p = put_hex32(buf, p, creg32(aud_base[i]));
        p = put_str(buf, p, " LEN=0x");
        p = put_hex16(buf, p, creg16(aud_base[i] + 4));
        p = put_str(buf, p, " PER=0x");
        p = put_hex16(buf, p, creg16(aud_base[i] + 6));
        buf[p] = '\0';
        log_line(buf);
    }

    log_line("[DMA] ---- end ----");
}

// ---------------------------------------------------------------------------
// Copper debugger
// ---------------------------------------------------------------------------

// Lookup table: chipset register offset (word-aligned) → name string.
// Only the most common registers; others shown as "0xNNN".
static const char *cop_reg_name(uint16_t offset) {
    switch (offset) {
    case 0x020: return "DSKPTH";   case 0x022: return "DSKPTL";
    case 0x040: return "BLTCON0";  case 0x042: return "BLTCON1";
    case 0x044: return "BLTAFWM";  case 0x046: return "BLTALWM";
    case 0x060: return "BLTDPTH";  case 0x062: return "BLTDPTL";
    case 0x064: return "BLTAPTL";  case 0x066: return "BLTAPTH"; // intentionally swapped in chipram
    case 0x07E: return "BLTSIZE";
    case 0x080: return "COP1LCH";  case 0x082: return "COP1LCL";
    case 0x084: return "COP2LCH";  case 0x086: return "COP2LCL";
    case 0x088: return "COPJMP1";  case 0x08A: return "COPJMP2";
    case 0x08E: return "DIWSTRT";  case 0x090: return "DIWSTOP";
    case 0x092: return "DDFSTRT";  case 0x094: return "DDFSTOP";
    case 0x096: return "DMACON";   case 0x09A: return "INTENA";
    case 0x09C: return "INTREQ";   case 0x09E: return "ADKCON";
    case 0x100: return "BPLCON0";  case 0x102: return "BPLCON1";
    case 0x104: return "BPLCON2";  case 0x108: return "BPL1MOD";
    case 0x10A: return "BPL2MOD";
    case 0x0E0: return "BPL1PTH";  case 0x0E2: return "BPL1PTL";
    case 0x0E4: return "BPL2PTH";  case 0x0E6: return "BPL2PTL";
    case 0x0E8: return "BPL3PTH";  case 0x0EA: return "BPL3PTL";
    case 0x0EC: return "BPL4PTH";  case 0x0EE: return "BPL4PTL";
    case 0x0F0: return "BPL5PTH";  case 0x0F2: return "BPL5PTL";
    case 0x0F4: return "BPL6PTH";  case 0x0F6: return "BPL6PTL";
    case 0x120: return "SPR0PTH";  case 0x122: return "SPR0PTL";
    case 0x124: return "SPR1PTH";  case 0x126: return "SPR1PTL";
    case 0x128: return "SPR2PTH";  case 0x12A: return "SPR2PTL";
    case 0x12C: return "SPR3PTH";  case 0x12E: return "SPR3PTL";
    case 0x130: return "SPR4PTH";  case 0x132: return "SPR4PTL";
    case 0x134: return "SPR5PTH";  case 0x136: return "SPR5PTL";
    case 0x138: return "SPR6PTH";  case 0x13A: return "SPR6PTL";
    case 0x13C: return "SPR7PTH";  case 0x13E: return "SPR7PTL";
    case 0x180: return "COLOR00";  case 0x182: return "COLOR01";
    case 0x184: return "COLOR02";  case 0x186: return "COLOR03";
    case 0x188: return "COLOR04";  case 0x18A: return "COLOR05";
    case 0x18C: return "COLOR06";  case 0x18E: return "COLOR07";
    case 0x190: return "COLOR08";  case 0x192: return "COLOR09";
    case 0x194: return "COLOR10";  case 0x196: return "COLOR11";
    case 0x198: return "COLOR12";  case 0x19A: return "COLOR13";
    case 0x19C: return "COLOR14";  case 0x19E: return "COLOR15";
    case 0x1A0: return "COLOR16";  case 0x1A2: return "COLOR17";
    case 0x1A4: return "COLOR18";  case 0x1A6: return "COLOR19";
    case 0x1A8: return "COLOR20";  case 0x1AA: return "COLOR21";
    case 0x1AC: return "COLOR22";  case 0x1AE: return "COLOR23";
    case 0x1B0: return "COLOR24";  case 0x1B2: return "COLOR25";
    case 0x1B4: return "COLOR26";  case 0x1B6: return "COLOR27";
    case 0x1B8: return "COLOR28";  case 0x1BA: return "COLOR29";
    case 0x1BC: return "COLOR30";  case 0x1BE: return "COLOR31";
    default: return NULL;
    }
}

void emu_debug_copper(uint32_t max_insn) {
    log_line("[COP] ---- Copper state ----");

    // State header
    {
        char buf[80];
        static const char *state_name[] = { "FETCH1","FETCH2","MOVE","WAIT","HALT" };
        uint32_t cs = ChipsetState->CopperState;
        int p = put_str(buf, 0, "[COP] state=");
        p = put_str(buf, p, cs <= 4 ? state_name[cs] : "?");
        p = put_str(buf, p, " PC=0x");
        p = put_hex32(buf, p, ChipsetState->CopperPC);
        p = put_str(buf, p, " wake=");
        p = put_hex32(buf, p, (uint32_t)ChipsetState->copper_wake_cycle);
        buf[p] = '\0';
        log_line(buf);
    }

    // Copper list origin pointers
    {
        char buf[64];
        int p = put_str(buf, 0, "[COP] COP1LC=0x");
        p = put_hex32(buf, p, creg32(0xDFF080));
        p = put_str(buf, p, " COP2LC=0x");
        p = put_hex32(buf, p, creg32(0xDFF084));
        buf[p] = '\0';
        log_line(buf);
    }

    // Decode instructions from current PC
    uint32_t pc = ChipsetState->CopperPC;
    for (uint32_t i = 0; i < max_insn && pc + 3 < 0x200000; i++, pc += 4) {
        uint16_t ir1 = creg16(pc);
        uint16_t ir2 = creg16(pc + 2);

        char buf[72];
        int p = put_str(buf, 0, "[COP]   0x");
        p = put_hex32(buf, p, pc);
        p = put_str(buf, p, ": ");

        if (ir1 & 1) {
            // WAIT or SKIP
            uint8_t vwait = (uint8_t)(ir1 >> 8);
            uint8_t hwait = (uint8_t)(ir1 & 0xFE);
            uint8_t vmask = (uint8_t)(ir2 >> 8);
            uint8_t hmask = (uint8_t)(ir2 & 0xFE);
            if (ir2 & 1) {
                p = put_str(buf, p, "SKIP V=0x");
            } else {
                p = put_str(buf, p, "WAIT V=0x");
            }
            p = put_hex8(buf, p, vwait);
            p = put_str(buf, p, " H=0x");
            p = put_hex8(buf, p, hwait);
            p = put_str(buf, p, " VM=0x");
            p = put_hex8(buf, p, vmask);
            p = put_str(buf, p, " HM=0x");
            p = put_hex8(buf, p, hmask);
            // Mark the instruction the CPU is currently stalled on
            if (ChipsetState->CopperState == 3 && pc == ChipsetState->CopperPC - 4) {
                p = put_str(buf, p, " <--");
            }
            // Stop decoding at WAIT $FFFF,$FFFE (end-of-list sentinel)
            if (ir1 == 0xFFFF && ir2 == 0xFFFE) {
                buf[p] = '\0'; log_line(buf);
                log_line("[COP]   (end of list)");
                break;
            }
        } else {
            // MOVE
            uint16_t reg_off = ir1 & 0x1FE;
            const char *rname = cop_reg_name(reg_off);
            p = put_str(buf, p, "MOVE 0x");
            p = put_hex16(buf, p, ir2);
            p = put_str(buf, p, " -> ");
            if (rname) {
                p = put_str(buf, p, rname);
            } else {
                p = put_str(buf, p, "0x");
                p = put_hex16(buf, p, reg_off);
            }
        }
        buf[p] = '\0';
        log_line(buf);
    }

    log_line("[COP] ---- end ----");
}

// ---------------------------------------------------------------------------
// Memory inspector
// ---------------------------------------------------------------------------

void emu_debug_mem(uint32_t addr, uint32_t len) {
    char buf[80];
    log_line("[MEM] ---- memory dump ----");

    // Print 16 bytes per line: "AAAAAAAA  XX XX XX ... |ASCII|"
    for (uint32_t off = 0; off < len; off += 16) {
        int p = 0;
        p = put_hex32(buf, p, addr + off);
        buf[p++] = ' '; buf[p++] = ' ';

        for (int col = 0; col < 16; col++) {
            uint32_t a = addr + off + (uint32_t)col;
            uint8_t  v = (a < 0x1000000) ? RAM24bit[a] : 0xFF;
            p = put_hex8(buf, p, v);
            buf[p++] = ' ';
            if (col == 7) buf[p++] = ' ';  // extra gap in the middle
        }

        buf[p++] = '|';
        for (int col = 0; col < 16; col++) {
            uint32_t a = addr + off + (uint32_t)col;
            uint8_t  v = (a < 0x1000000) ? RAM24bit[a] : 0xFF;
            buf[p++] = (v >= 0x20 && v < 0x7F) ? (char)v : '.';
            if ((off + (uint32_t)col + 1) >= len) break;
        }
        buf[p++] = '|';
        buf[p]   = '\0';
        log_line(buf);
    }

    log_line("[MEM] ---- end ----");
}
