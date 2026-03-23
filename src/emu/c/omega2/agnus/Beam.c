// Beam.c — Amiga raster beam position tracker (PAL)

#include "Beam.h"

Beam_t g_beam;

void beam_init(void) {
    g_beam.v          = 0;
    g_beam.h          = 0;
    g_beam.frame      = 0;
    g_beam.lof        = 0;     // start in short-frame mode
    g_beam.lof_toggle = 0;     // no interlace by default (BPLCON0 LACE bit)
    g_beam.lol        = 0;     // short lines
    g_beam.lol_toggle = 0;     // PAL: no LOL toggling
}

// Called when h reaches beam_hcnt() — advance to the next scan line.
// The caller is responsible for checking v > beam_vmax() after this call
// and triggering beam_eof() if necessary.
void beam_eol(void) {
    g_beam.h = 0;
    g_beam.v++;
    if (g_beam.lol_toggle)
        g_beam.lol ^= 1;
}

// Called when v exceeds beam_vmax() — advance to the next frame.
void beam_eof(void) {
    g_beam.v = 0;
    g_beam.frame++;
    if (g_beam.lof_toggle)
        g_beam.lof ^= 1;
}
