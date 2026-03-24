// src/emu/c/omega_glue.c
// Glue layer — replaces main.c, no SDL, no file I/O.
// Called from Rust via extern "C".

#include <stdint.h>
#include "omega_host.h"
#include "omega2/shared/omega_probe.h"
#include "omega2/agnus/Scheduler.h"
#include "omega2/agnus/Beam.h"
#include "omega2/cpu/m68k.h"
#include "omega2/Chipset.h"
#include "omega2/cia/CIA.h"
#include "omega2/agnus/DMA.h"
#include "omega2/paula/Floppy.h"
#include "omega2/memory/Memory.h"
#include "omega2/shared/Omega.h"
#include "omega2/shared/os_debug.h"
#include "omega2/shared/emu_debug.h"

static Omega_t* g_omega = NULL;

// ChipsetState is the global pointer set by InitChipset() in Chipset.c.
// After the Omega_t restructure it is no longer accessible via g_omega->Chipstate.
extern Chipset_t* ChipsetState;

// Ajuste de posição do display Amiga no framebuffer.
//
// O beam começa acima e à esquerda da área visível.  A fórmula garante que
// o primeiro pixel do DIW (quando VPOS = DIWSTRT_V e HPOS = DIWSTRT_H)
// aterrise exatamente na posição desejada no framebuffer do host.
//
//   FrameBufferLine = &frameBuffer[stride * VPOS + HSTART*2]
//   frameBuffer     = fb - (stride * FB_LINE_OFFSET + pixel_offset)
//
//   → pixel_offset = HSTART*2 - x_start
//
// Onde x_start é a coluna no host onde queremos que o display Amiga comece.
// Centralizamos: x_start = (stride - AMIGA_LORES_HOST_WIDTH) / 2.
// Para stride == 640, x_start == 0 → imagem alinhada à esquerda.
//
#define FB_LINE_OFFSET           20   // linhas de recuo vertical (beam start)
#define AMIGA_LORES_HOST_WIDTH  640   // lores: 320 pixels × 2 (cada pixel duplicado)
#define AMIGA_HSTART_STANDARD   129   // DIWSTRT_H padrão PAL (0x81) usado como fallback

static void apply_fb_offset(Chipset_t* cs) {
    uint32_t* fb = omega_host_framebuffer();
    if (!fb) return;

    // omega_host_pitch() retorna pitch em bytes; converter para pixels (uint32_t).
    int32_t stride_px = omega_host_pitch() / (int32_t)sizeof(uint32_t);
    if (stride_px <= 0) stride_px = 640;
    cs->frameBufferStride = stride_px;

    // HSTART: atualizado pelo Copper/CPU quando DIWSTRT é escrito.
    // Antes da primeira escrita (init) vale 0 — usar padrão PAL.
    int32_t hstart   = (cs->HSTART > 0) ? (int32_t)cs->HSTART
                                         : AMIGA_HSTART_STANDARD;
    int32_t hstart2  = hstart * 2; // host pixels até o início da janela de display

    // Centralizar a imagem Amiga no framebuffer; para larguras <= 640 alinhar à esquerda.
    int32_t x_start  = (stride_px > AMIGA_LORES_HOST_WIDTH)
                       ? (stride_px - AMIGA_LORES_HOST_WIDTH) / 2
                       : 0;
    int32_t pixel_offset = hstart2 - x_start;

    cs->frameBuffer = fb - (stride_px * FB_LINE_OFFSET + pixel_offset);
}

void omega_init(void) {
    omega_host_log("Omega: init start");

    probe_init();

    g_omega = InitRAM(0);

    apply_fb_offset(ChipsetState);

    FloppyInit();

    beam_init();
    sched_init();
    sched_dma_init();

    m68k_init();
    m68k_set_cpu_type(M68K_CPU_TYPE_68000);
    m68k_pulse_reset();

    omega_host_log("Omega: init done");
}

static uint32_t g_frame_count = 0;

void omega_run_frame(void) {
    if (!g_omega) return;

    Chipset_t* cs = ChipsetState;

    // Each DMA cycle = CPU_CYCLES_PER_DMA (2) CPU cycles.
    // Run in batches of SCHED_BATCH DMA cycles for performance.
    // Watchdog: abort frame if we exceed ~1M DMA cycles without VBL
    // (1M / 71k cycles-per-frame ≈ 14 PAL frames — enough to catch hangs).
    uint64_t frame_start = sched_clock();
    while (cs->VBL == 0) {
        m68k_execute(SCHED_BATCH * CPU_CYCLES_PER_DMA);
        sched_advance_n(SCHED_BATCH);

        if ((sched_clock() - frame_start) > 1000000) {
            probe_emit(EVT_WATCHDOG, (uint32_t)(sched_clock() - frame_start),
                       m68k_get_reg(NULL, M68K_REG_PC));
            probe_dump_serial(512);
            break;
        }
    }

    g_frame_count++;

    // Early dump after frame 2 — capture AROS startup DMACON/BPLCON0 writes
    if (g_frame_count == 2) {
        probe_dump_serial(512);
    }

    // One-shot dump after frame 22 — capture after Copper list is configured
    if (g_frame_count == 22) {
        os_debug_dump();
        emu_debug_dma();
        emu_debug_copper(32);
        emu_debug_mem(0, 256);
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
