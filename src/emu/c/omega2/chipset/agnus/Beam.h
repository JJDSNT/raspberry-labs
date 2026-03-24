#ifndef BEAM_H
#define BEAM_H

#include <stdint.h>

/*
 * ---------------------------------------------------------------------------
 * PAL timing reference (Amiga)
 * ---------------------------------------------------------------------------
 *
 * Short line  : 227 DMA cycles (h = 0..226, hCnt = 227)
 * Long  line  : 228 DMA cycles (h = 0..227, hCnt = 228) — lol=1
 *
 * Short frame : 312 lines      (v = 0..311, vMax = 311)
 * Long  frame : 313 lines      (v = 0..312, vMax = 312) — lof=1
 */

#define PAL_HPOS_CNT     227
#define PAL_HPOS_CNT_LL  228
#define PAL_VPOS_CNT_SF  312
#define PAL_VPOS_CNT_LF  313

/*
 * ---------------------------------------------------------------------------
 * Beam state
 * ---------------------------------------------------------------------------
 *
 * Represents the current raster position of the Amiga beam.
 *
 * - v/h track the current scanline and DMA cycle
 * - lof controls long/short frame (interlace)
 * - lol controls long/short line (rarely used in PAL)
 */

typedef struct
{
    int32_t  v;            // vertical position (0 .. vMax)
    int32_t  h;            // horizontal position (0 .. hMax)

    uint32_t frame;        // absolute frame counter

    uint8_t  lof;          // long-frame flag (1 = 313 lines)
    uint8_t  lof_toggle;   // toggles lof each frame (interlace mode)

    uint8_t  lol;          // long-line flag (1 = 228 cycles)
    uint8_t  lol_toggle;   // toggles lol each line (normally 0 on PAL)

} Beam_t;

extern Beam_t g_beam;

/*
 * ---------------------------------------------------------------------------
 * Inline accessors (constant-time)
 * ---------------------------------------------------------------------------
 */

// Number of DMA cycles in the current line (227 or 228)
static inline int beam_hcnt(void)
{
    return g_beam.lol ? PAL_HPOS_CNT_LL : PAL_HPOS_CNT;
}

// Last valid horizontal position
static inline int beam_hmax(void)
{
    return beam_hcnt() - 1;
}

// Number of scanlines in the current frame (312 or 313)
static inline int beam_vcnt(void)
{
    return g_beam.lof ? PAL_VPOS_CNT_LF : PAL_VPOS_CNT_SF;
}

// Last valid vertical position
static inline int beam_vmax(void)
{
    return beam_vcnt() - 1;
}

/*
 * Pack current beam position into VHPOS format:
 * bits [16:8] = VPOS[8:0]
 * bits [7:0]  = HPOS[7:0]
 */
static inline uint32_t beam_vhpos(void)
{
    return ((uint32_t)(g_beam.v & 0x1FF) << 8) |
           ((uint32_t)(g_beam.h & 0xFF));
}

/*
 * VPOSR (0xDFF004)
 *
 * bit15 = LOF
 * bit0  = VPOS[8]
 *
 * NOTE: Chip ID bits are not modeled here.
 */
static inline uint16_t beam_vposr(void)
{
    return (uint16_t)(
        (g_beam.lof ? 0x8000u : 0u) |
        ((g_beam.v >> 8) & 0x0001u)
    );
}

/*
 * VHPOSR (0xDFF006)
 *
 * bits[15:8] = VPOS[7:0]
 * bits[7:0]  = HPOS[7:0]
 */
static inline uint16_t beam_vhposr(void)
{
    return (uint16_t)(
        ((g_beam.v & 0xFF) << 8) |
        (g_beam.h & 0xFF)
    );
}

/*
 * Set beam position from VHPOS-style value.
 *
 * NOTE:
 * Caller should ensure the value is valid. We clamp to safe ranges
 * to avoid invalid states during debugging or register writes.
 */
static inline void beam_set_vhpos(uint32_t vhpos)
{
    g_beam.v = (int32_t)((vhpos >> 8) & 0x1FF);
    g_beam.h = (int32_t)(vhpos & 0xFF);

    if (g_beam.v > beam_vmax())
        g_beam.v = beam_vmax();

    if (g_beam.h > beam_hmax())
        g_beam.h = beam_hmax();
}

/*
 * ---------------------------------------------------------------------------
 * Lifecycle
 * ---------------------------------------------------------------------------
 */

void beam_init(void);     // Initialize beam at start of frame 0
void beam_reset(void);    // Reset beam state (alias for init)
void beam_advance(void);  // Advance one DMA cycle (h++, handles wrap)
void beam_eol(void);      // End-of-line (h reset, v++)
void beam_eof(void);      // End-of-frame (v reset, frame++)

#endif /* BEAM_H */