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

// Ajuste de posição do display Amiga no framebuffer.
// O beam começa 20 linhas acima e 180 pixels à esquerda da área visível.
// Subtrair do ponteiro inicial faz o índice 0 da DMA cair fora da tela
// enquanto a área visível começa no offset correto.
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

    g_omega = InitRAM(0);

    Chipset_t* cs = (Chipset_t*)g_omega->Chipstate;
    apply_fb_offset(cs);

    FloppyInit();

    m68k_init();
    m68k_set_cpu_type(M68K_CPU_TYPE_68000);
    m68k_pulse_reset();

    omega_host_log("Omega: init done");
}

void omega_run_frame(void) {
    if (!g_omega) return;

    Chipset_t* cs = (Chipset_t*)g_omega->Chipstate;

    // Loop apertado — replica o main.c original.
    // Executa 128 ciclos M68k + um passo DMA até que a chipset sinalize VBL.
    while (cs->VBL == 0) {
        m68k_execute(128);
        DMAExecute(g_omega->Chipstate, NULL);
    }

    // Drena eventos de teclado acumulados pelo USB HID
    uint8_t scancode;
    int pressed;
    while (omega_host_poll_key(&scancode, &pressed)) {
        if (pressed) pressKey(scancode);
        else         releaseKey(scancode);
    }

    // Frame completo — reseta VBL e atualiza ponteiro do framebuffer
    cs->VBL = 0;
    cs->needsRedraw = 0;
    apply_fb_offset(cs);

    omega_host_vsync();
}
