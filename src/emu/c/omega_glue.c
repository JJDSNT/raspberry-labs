// src/emu/c/omega_glue.c
// Glue layer — replaces main.c, no SDL, no file I/O.
// Called from Rust via extern "C".

#include <stdint.h>
#include "omega_host.h"
#include "omega2/shared/omega_probe.h"
#include "omega2/cpu/m68k.h"
#include "omega2/Chipset.h"
#include "omega2/cia/CIA.h"
#include "omega2/agnus/DMA.h"
#include "omega2/paula/Floppy.h"
#include "omega2/memory/Memory.h"
#include "omega2/shared/Omega.h"
#include "omega2/shared/os_debug.h"

static Omega_t* g_omega = NULL;

// ChipsetState is the global pointer set by InitChipset() in Chipset.c.
// After the Omega_t restructure it is no longer accessible via g_omega->Chipstate.
extern Chipset_t* ChipsetState;

// Ajuste de posição do display Amiga no framebuffer.
// O beam começa 20 linhas acima e 180 pixels à esquerda da área visível.
#define FB_LINE_OFFSET  20
#define FB_PIXEL_OFFSET 180
#define FB_WIDTH        800

static void apply_fb_offset(Chipset_t* cs) {
    uint32_t* fb = omega_host_framebuffer();
    if (!fb) return;
    cs->frameBufferPitch = omega_host_pitch();
    cs->frameBuffer = fb - (FB_WIDTH * FB_LINE_OFFSET + FB_PIXEL_OFFSET);
}

void omega_init(void) {
    omega_host_log("Omega: init start");

    probe_init();

    g_omega = InitRAM(0);

    apply_fb_offset(ChipsetState);

    FloppyInit();

    m68k_init();
    m68k_set_cpu_type(M68K_CPU_TYPE_68000);
    m68k_pulse_reset();

    omega_host_log("Omega: init done");
}

static uint32_t g_frame_count = 0;

void omega_run_frame(void) {
    if (!g_omega) return;

    Chipset_t* cs = ChipsetState;

    uint32_t iters = 0;
    while (cs->VBL == 0) {
        m68k_execute(128);
        DMAExecute(ChipsetState, NULL);

        if (++iters > 5000000) {
            probe_emit(EVT_WATCHDOG, iters, m68k_get_reg(NULL, M68K_REG_PC));
            probe_dump_serial(512);
            break;
        }
    }

    g_frame_count++;

    // One-shot dump after frame 22 — capture after Copper list is configured
    if (g_frame_count == 22) {
        os_debug_dump();
        probe_dump_serial(512);
    }

    // Drain keyboard events
    uint8_t scancode;
    int pressed;
    while (omega_host_poll_key(&scancode, &pressed)) {
        if (pressed) pressKey(scancode);
        else         releaseKey(scancode);
    }

    // Frame complete — reset VBL and update framebuffer pointer
    cs->VBL = 0;
    cs->needsRedraw = 0;
    apply_fb_offset(cs);

    omega_host_vsync();
}

// Called from Rust to trigger a probe dump on demand (e.g. via serial command).
void omega_probe_dump(uint32_t last_n) {
    probe_dump_serial(last_n);
}
