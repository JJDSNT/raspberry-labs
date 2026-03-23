// Beam.h — Amiga raster beam position tracker (PAL)
//
// Inspired by vAmiga's Beam struct. Tracks the DMA cycle position as
// separate v (vertical) and h (horizontal) counters with correct PAL
// timing including LOF (long frame) and LOL (long line) flipflops.
//
// PAL timing reference:
//   Short line  : 227 DMA cycles (h = 0..226, hCnt = 227)
//   Long  line  : 228 DMA cycles (h = 0..227, hCnt = 228) — lol=1
//   Short frame : 312 lines      (v = 0..311, vMax = 311)
//   Long  frame : 313 lines      (v = 0..312, vMax = 312) — lof=1
//
// VHPOS encoding (Amiga HRM, backward-compatible with Chipset.c):
//   bits [16:8]  = VPOS (9-bit vertical counter)
//   bits [7:0]   = HPOS (horizontal DMA cycle, low 8 bits)
//
// CPU register layout:
//   VPOSR  (0xDFF004): bit15=LOF, bits[14:1]=chipID(ECS), bit0=VPOS[8]
//   VHPOSR (0xDFF006): bits[15:8]=VPOS[7:0], bits[7:0]=HPOS[7:0]

#ifndef Beam_h
#define Beam_h

#include <stdint.h>

// ---------------------------------------------------------------------------
// PAL timing constants
// ---------------------------------------------------------------------------
#define PAL_HPOS_CNT     227    // DMA cycles per short line
#define PAL_HPOS_CNT_LL  228    // DMA cycles per long line  (lol=1)
#define PAL_VPOS_CNT_SF  312    // scan lines in a short frame (lof=0)
#define PAL_VPOS_CNT_LF  313    // scan lines in a long frame  (lof=1)

// ---------------------------------------------------------------------------
// Beam struct
// ---------------------------------------------------------------------------
typedef struct {
    int32_t  v;            // vertical   position 0 .. vMax (PAL: 311 or 312)
    int32_t  h;            // horizontal position 0 .. hMax (PAL: 226 or 227)
    uint32_t frame;        // absolute frame counter (since reset)
    uint8_t  lof;          // long-frame flipflop:  1 = current frame has 313 lines
    uint8_t  lof_toggle;   // 1 = flip lof each frame (enables interlace)
    uint8_t  lol;          // long-line  flipflop:  1 = current line has 228 cycles
    uint8_t  lol_toggle;   // 1 = flip lol each line  (PAL: normally 0)
} Beam_t;

extern Beam_t g_beam;

// ---------------------------------------------------------------------------
// Inline accessors (constant-time)
// ---------------------------------------------------------------------------

// Number of DMA cycles in the current line (227 or 228)
static inline int beam_hcnt(void) { return g_beam.lol ? PAL_HPOS_CNT_LL : PAL_HPOS_CNT; }

// Last valid h position in the current line
static inline int beam_hmax(void) { return beam_hcnt() - 1; }

// Number of scan lines in the current frame (312 or 313)
static inline int beam_vcnt(void) { return g_beam.lof ? PAL_VPOS_CNT_LF : PAL_VPOS_CNT_SF; }

// Last valid v position in the current frame
static inline int beam_vmax(void) { return beam_vcnt() - 1; }

// Pack v/h into the VHPOS register encoding used by Chipset.c
// bits[16:8] = VPOS[8:0], bits[7:0] = HPOS[7:0]
static inline uint32_t beam_vhpos(void) {
    return ((uint32_t)(g_beam.v & 0x1FF) << 8) | (uint32_t)(g_beam.h & 0xFF);
}

// VPOSR word (0xDFF004): bit15=LOF, bits[14:1]=chipID, bit0=VPOS[8]
static inline uint16_t beam_vposr(void) {
    return (uint16_t)((g_beam.lof ? 0x8000u : 0u) | (g_beam.v >> 8));
}

// VHPOSR word (0xDFF006): bits[15:8]=VPOS[7:0], bits[7:0]=HPOS[7:0]
static inline uint16_t beam_vhposr(void) {
    return (uint16_t)(((g_beam.v & 0xFF) << 8) | (g_beam.h & 0xFF));
}

// Unpack a VHPOS value into the beam (used by VPOSW / VHPOSW register writes)
static inline void beam_set_vhpos(uint32_t vhpos) {
    g_beam.v = (int32_t)((vhpos >> 8) & 0x1FF);
    g_beam.h = (int32_t)(vhpos & 0xFF);
}

// ---------------------------------------------------------------------------
// Lifecycle
// ---------------------------------------------------------------------------

void beam_init(void);   // Reset beam to start of frame 0
void beam_eol(void);    // End-of-line: advance v, reset h, toggle lol
void beam_eof(void);    // End-of-frame: reset v, advance frame, toggle lof

#endif /* Beam_h */
