// Beam.c — Amiga raster beam position tracker (PAL)

#include "Beam.h"

Beam_t g_beam;

void beam_init(void)
{
    g_beam.v = 0;
    g_beam.h = 0;
    g_beam.frame = 0;

    /*
     * Interlace / long-frame state
     */
    g_beam.lof = 0;
    g_beam.lof_toggle = 0;

    /*
     * Long-line state
     */
    g_beam.lol = 0;
    g_beam.lol_toggle = 0;
}

void beam_reset(void)
{
    beam_init();
}

void beam_eol(void)
{
    g_beam.h = 0;
    g_beam.v++;

    if (g_beam.lol_toggle) {
        g_beam.lol ^= 1;
    }
}

void beam_eof(void)
{
    g_beam.v = 0;
    g_beam.h = 0;
    g_beam.frame++;

    if (g_beam.lof_toggle) {
        g_beam.lof ^= 1;
    }

    /*
     * PAL default: no long-line alternation unless explicitly enabled.
     * If you later emulate modes that toggle LOL per frame, adjust here.
     */
    if (!g_beam.lol_toggle) {
        g_beam.lol = 0;
    }
}