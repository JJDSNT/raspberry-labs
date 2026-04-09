// src/emu/c/omega_glue.c
// Glue layer — replaces main.c.
// Called from Rust via extern "C".

#include <stdint.h>

#include "omega_host.h"

#include "omega2/debug/omega_probe.h"
#include "omega2/debug/os_debug.h"
#include "omega2/debug/emu_debug.h"

#include "omega2/chipset/agnus/Scheduler.h"
#include "omega2/chipset/agnus/Beam.h"
#include "omega2/chipset/agnus/DMA.h"
#include "omega2/chipset/Chipset.h"
#include "omega2/chipset/cia/CIA.h"
#include "omega2/chipset/paula/Floppy.h"

#include "omega2/cpu/m68k.h"
#include "omega2/memory/Memory.h"
#include "omega2/memory/memory_map.h"

#include "omega2/storage/HdfDevice.h"

/* ------------------------------------------------------------------------- */

static MemoryMap* g_memory = 0;

/* HDF global device */
HdfDevice g_hdf_device;
int g_hdf_enabled = 0;

/*
 * ChipsetState is the global pointer set by InitChipset() in Chipset.c.
 */
extern Chipset_t* ChipsetState;

/* ------------------------------------------------------------------------- */
/* Framebuffer offset logic                                                    */
/* ------------------------------------------------------------------------- */

#define FB_LINE_OFFSET           20
#define AMIGA_LORES_HOST_WIDTH  640
#define AMIGA_HSTART_STANDARD   129

static void apply_fb_offset(Chipset_t* cs)
{
    uint32_t* fb = omega_host_framebuffer();
    int32_t stride_px;
    int32_t hstart;
    int32_t hstart2;
    int32_t x_start;
    int32_t pixel_offset;

    if (!fb || !cs) {
        return;
    }

    stride_px = omega_host_pitch() / (int32_t)sizeof(uint32_t);
    if (stride_px <= 0) {
        stride_px = 640;
    }

    cs->frameBufferStride = stride_px;

    hstart = (cs->HSTART > 0) ? (int32_t)cs->HSTART
                              : AMIGA_HSTART_STANDARD;

    hstart2 = hstart * 2;

    x_start = (stride_px > AMIGA_LORES_HOST_WIDTH)
            ? (stride_px - AMIGA_LORES_HOST_WIDTH) / 2
            : 0;

    pixel_offset = hstart2 - x_start;

    cs->frameBuffer = fb - (stride_px * FB_LINE_OFFSET + pixel_offset);
}

/* ------------------------------------------------------------------------- */
/* INIT                                                                        */
/* ------------------------------------------------------------------------- */

void omega_init(void)
{
    omega_host_log("Omega: init start");

    probe_init();

    g_memory = memory_init(0);

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

/* ------------------------------------------------------------------------- */
/* HDF ATTACH                                                                  */
/* ------------------------------------------------------------------------- */

int omega_attach_hdf(const char* path)
{
    int rc;

    if (!path || !path[0]) {
        g_hdf_enabled = 0;
        omega_host_log("HDF: empty path");
        return -1;
    }

    if (g_hdf_enabled) {
        hdf_device_shutdown(&g_hdf_device);
        g_hdf_enabled = 0;
    }

    rc = hdf_device_init(&g_hdf_device, path);
    if (rc != 0) {
        g_hdf_enabled = 0;
        omega_host_log("HDF: init failed");
        return rc;
    }

    g_hdf_enabled = 1;
    omega_host_log("HDF: attached");
    return 0;
}

/* ------------------------------------------------------------------------- */
/* FRAME LOOP                                                                  */
/* ------------------------------------------------------------------------- */

static uint32_t g_frame_count = 0;

void omega_run_frame(void)
{
    Chipset_t* cs;
    uint64_t frame_start;
    uint8_t scancode;
    int pressed;

    if (!g_memory) {
        return;
    }

    cs = ChipsetState;
    frame_start = sched_clock();

    while (cs->VBL == 0) {
        m68k_execute(SCHED_BATCH * CPU_CYCLES_PER_DMA);
        sched_advance_n(SCHED_BATCH);

        if ((sched_clock() - frame_start) > 1000000) {
            probe_emit(
                EVT_WATCHDOG,
                (uint32_t)(sched_clock() - frame_start),
                m68k_get_reg(0, M68K_REG_PC)
            );
            probe_dump_serial(512);
            break;
        }
    }

    g_frame_count++;

    if (g_frame_count == 2) {
        probe_dump_serial(512);
    }

    if (g_frame_count == 22) {
        os_debug_dump();
        emu_debug_dma();
        emu_debug_copper(32);
        emu_debug_mem(0, 256);
        probe_dump_serial(512);
    }

    while (omega_host_poll_key(&scancode, &pressed)) {
        if (pressed) {
            pressKey(scancode);
        } else {
            releaseKey(scancode);
        }
    }

    cs->VBL = 0;
    cs->needsRedraw = 0;
    apply_fb_offset(cs);

    omega_host_vsync();
}

/* ------------------------------------------------------------------------- */
/* DEBUG                                                                       */
/* ------------------------------------------------------------------------- */

void omega_probe_dump(uint32_t last_n)
{
    probe_dump_serial(last_n);
}