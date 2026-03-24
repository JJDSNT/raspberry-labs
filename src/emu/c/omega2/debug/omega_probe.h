// omega_probe.h — lock-free ring-buffer event tracer for OmegaEmu.
//
// Usage:
//   probe_emit(EVT_INTR_FIRE, level, intreqr);
//   probe_dump_serial(256);  // print last 256 events via omega_host_log
//
// 4096 entries × 16 bytes = 64 KB.  Zero overhead when paused.

#pragma once
#include <stdint.h>

// ── Event types ───────────────────────────────────────────────────────────────
typedef enum {
    EVT_NONE        = 0,
    EVT_VBL,            // a=frame,      b=DMACONR
    EVT_INTR_FIRE,      // a=level,      b=INTREQR (Paula fires m68k_set_irq)
    EVT_INTR_ACK,       // a=level,      b=vector (CPU IACK)
    EVT_CIA_WRITE,      // a=addr,       b=value  (ICR/PRB/PRA key regs)
    EVT_CIA_READ,       // a=addr,       b=value
    EVT_COPPER_MOVE,    // a=reg_offset, b=value
    EVT_COPPER_WAIT,    // a=waitpos,    b=mask
    EVT_CPU_STOP,       // a=PC,         b=new_SR (imediato do STOP)
    EVT_CPU_EXCEPT,     // a=vector_addr,b=PC
    EVT_FLOPPY_CMD,     // a=DSKLEN,     b=DSKPTR
    EVT_SIGNAL_SENT,    // a=task_ptr,   b=signals
    EVT_WATCHDOG,       // a=iters,      b=PC
    EVT_CUSTOM_WRITE,   // a=addr,       b=value  (catch-all for custom regs)
} ProbeEvtType;

// ── Event record (16 bytes) ───────────────────────────────────────────────────
typedef struct {
    uint32_t cycle;     // DMA cycle counter at time of event
    uint32_t a;         // payload A
    uint32_t b;         // payload B
    uint16_t vpos;      // beam line (VHPOS >> 8)
    uint8_t  type;      // ProbeEvtType
    uint8_t  flags;     // reserved
} ProbeEvt;

// ── Ring buffer ───────────────────────────────────────────────────────────────
#define PROBE_BUF_BITS 12                        // 4096 entries
#define PROBE_BUF_SIZE (1u << PROBE_BUF_BITS)
#define PROBE_BUF_MASK (PROBE_BUF_SIZE - 1u)

typedef struct {
    ProbeEvt buf[PROBE_BUF_SIZE];
    uint32_t head;      // monotonically increasing write index
    uint32_t paused;    // non-zero → emit is a no-op (snapshot mode)
} OmegaProbe;

extern OmegaProbe g_probe;
extern uint32_t   g_probe_cycle;   // updated by DMAExecute every tick
extern uint16_t   g_probe_vpos;    // current beam line (VHPOS >> 8)

// ── API ───────────────────────────────────────────────────────────────────────
void probe_init(void);
void probe_pause(void);
void probe_resume(void);

// Print last `last_n` events (capped at ring size) via omega_host_log.
void probe_dump_serial(uint32_t last_n);

// ── Inline emit (hot path) ────────────────────────────────────────────────────
static inline void probe_emit(ProbeEvtType t, uint32_t a, uint32_t b) {
    if (g_probe.paused) return;
    ProbeEvt* e = &g_probe.buf[g_probe.head & PROBE_BUF_MASK];
    e->cycle = g_probe_cycle;
    e->vpos  = g_probe_vpos;
    e->type  = (uint8_t)t;
    e->flags = 0;
    e->a     = a;
    e->b     = b;
    g_probe.head++;          // no atomic needed: single-threaded emulator
}
