// src/emu/c/omega_glue.c
// Glue layer — replaces main.c, no SDL, no file I/O.
// Called from Rust via extern "C".

#include <stdint.h>
#include "omega_host.h"
#include "omega2/m68k.h"
#include "omega2/Chipset.h"
#include "omega2/CIA.h"
#include "omega2/DMA.h"
#include "omega2/Floppy.h"
#include "omega2/Memory.h"
#include "omega2/Omega.h"

static Omega_t* g_omega = NULL;

void omega_init(void) {
    omega_host_log("Omega: init start");

    g_omega = InitRAM(0);

    Chipset_t* cs = (Chipset_t*)g_omega->Chipstate;
    cs->frameBuffer = omega_host_framebuffer();
    cs->frameBufferPitch = omega_host_pitch();

    FloppyInit();

    m68k_init();
    m68k_set_cpu_type(M68K_CPU_TYPE_68000);
    m68k_pulse_reset();

    omega_host_log("Omega: init done");
}

void omega_run_frame(void) {
    if (!g_omega) return;

    Chipset_t* cs = (Chipset_t*)g_omega->Chipstate;

    m68k_execute(128);
    DMAExecute(g_omega->Chipstate, NULL);

    if (cs->VBL == 1) {
        cs->VBL = 0;
        cs->needsRedraw = 0;
        cs->frameBuffer = omega_host_framebuffer();
        cs->frameBufferPitch = omega_host_pitch();
        omega_host_vsync();
    }
}
