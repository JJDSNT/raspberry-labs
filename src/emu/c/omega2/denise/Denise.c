// Denise.c — Amiga Denise chip: pixel output engine
//
// Architecture:
//   Agnus (Bitplane.c) handles DMA fetch: reads bitplane words from chip RAM
//   into PixelBuffer via planar-to-chunky (P2C) conversion.
//
//   Denise (this file) handles pixel output: reads PixelBuffer and writes
//   RGBA pixels to the host framebuffer, applying palette lookup and
//   special color modes (HAM, EHB).
//
//   The actual per-DMA-cycle output loop lives inside BitplaneExecuteLores /
//   BitplaneExecuteHires (called from DMAExecute) because the display timing
//   is tightly coupled to the Agnus fetch pipeline.  DeniseExecute() is a
//   hook called by DMAExecute that handles state that is Denise-specific and
//   independent of the fetch slot: currently it serves as a no-op placeholder
//   for future shift-register and sprite priority work.
//
// Color modes supported:
//   Normal   : up to 5 planes → 32 colors (Colour[0..31])
//   EHB      : 6 planes, HOMOD=0 → 64 colors; upper 32 are half-bright
//   HAM6     : 6 planes, HOMOD=1 → Hold-And-Modify; bits[5:4] = ctrl
//                ctrl=00 → palette[bits[3:0]]
//                ctrl=01 → hold, modify blue  channel (bits[3:0] → 4-bit blue)
//                ctrl=10 → hold, modify red   channel
//                ctrl=11 → hold, modify green channel
//              HAM carry color is reset to Colour[0] at display-window start
//              (see BitplaneExecuteLores, DIWSTRT trigger).

#include "Denise.h"

void DeniseExecute(Chipset_t* ChipsetState){
    // Pixel output is driven by BitplaneExecuteLores / BitplaneExecuteHires.
    // No work needed here per DMA cycle at this time.
    (void)ChipsetState;
}
