// omega_probe.c — OmegaProbe implementation

#include "omega_probe.h"
#include "omega_host.h"

// ── Globals ───────────────────────────────────────────────────────────────────
OmegaProbe g_probe;
uint32_t   g_probe_cycle = 0;
uint16_t   g_probe_vpos  = 0;

void probe_init(void)  { g_probe.head = 0; g_probe.paused = 0; }
void probe_pause(void) { g_probe.paused = 1; }
void probe_resume(void){ g_probe.paused = 0; }

// ── Hex formatter (no libc required) ─────────────────────────────────────────
static const char HEX[] = "0123456789ABCDEF";

static void fmt_u32(char* out, uint32_t v) {
    for (int i = 7; i >= 0; i--) { out[i] = HEX[v & 0xF]; v >>= 4; }
}
static void fmt_u16(char* out, uint16_t v) {
    for (int i = 3; i >= 0; i--) { out[i] = HEX[v & 0xF]; v >>= 4; }
}

// ── Event name table (11 chars wide, space-padded) ────────────────────────────
static const char* evt_name(uint8_t t) {
    switch ((ProbeEvtType)t) {
    case EVT_VBL:          return "VBL        ";
    case EVT_INTR_FIRE:    return "INTR_FIRE  ";
    case EVT_INTR_ACK:     return "INTR_ACK   ";
    case EVT_CIA_WRITE:    return "CIA_WRITE  ";
    case EVT_CIA_READ:     return "CIA_READ   ";
    case EVT_COPPER_MOVE:  return "COPPER_MOVE";
    case EVT_COPPER_WAIT:  return "COPPER_WAIT";
    case EVT_CPU_STOP:     return "CPU_STOP   ";
    case EVT_CPU_EXCEPT:   return "CPU_EXCEPT ";
    case EVT_FLOPPY_CMD:   return "FLOPPY_CMD ";
    case EVT_SIGNAL_SENT:  return "SIG_SENT   ";
    case EVT_WATCHDOG:     return "WATCHDOG   ";
    case EVT_CUSTOM_WRITE: return "CUST_WRITE ";
    default:               return "???        ";
    }
}

// ── probe_dump_serial ─────────────────────────────────────────────────────────
// Output format (one line per event, 51 chars + newline):
//   [PROBE] CCCCCCCC VVV TYPENAME AAAAAAAA BBBBBBBB
//           cycle    vpos          a        b
void probe_dump_serial(uint32_t last_n) {
    probe_pause();

    uint32_t head  = g_probe.head;
    uint32_t count = head < PROBE_BUF_SIZE ? head : PROBE_BUF_SIZE;
    if (last_n > count) last_n = count;
    uint32_t start = head - last_n;

    omega_host_log("=== OmegaProbe ===");

    for (uint32_t i = 0; i < last_n; i++) {
        uint32_t idx = (start + i) & PROBE_BUF_MASK;
        ProbeEvt* e  = &g_probe.buf[idx];
        if (e->type == EVT_NONE) continue;

        // "[PROBE] " (8) + cycle(8) + " " + vpos(3) + " " + name(11) +
        // " " + a(8) + " " + b(8) + "\0"  =  52 bytes
        char line[56];
        line[0]='['; line[1]='P'; line[2]='R'; line[3]='O';
        line[4]='B'; line[5]='E'; line[6]=']'; line[7]=' ';
        fmt_u32(line + 8,  e->cycle);
        line[16] = ' ';
        // vpos: 3 hex digits (0-13F = 0-319)
        line[17] = HEX[(e->vpos >> 8) & 0xF];
        line[18] = HEX[(e->vpos >> 4) & 0xF];
        line[19] = HEX[ e->vpos       & 0xF];
        line[20] = ' ';
        const char* n = evt_name(e->type);
        for (int k = 0; k < 11; k++) line[21 + k] = n[k];
        line[32] = ' ';
        fmt_u32(line + 33, e->a);
        line[41] = ' ';
        fmt_u32(line + 42, e->b);
        line[50] = '\0';
        omega_host_log(line);
    }

    omega_host_log("=== end probe ===");
    probe_resume();
}
