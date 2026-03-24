// Paula.h — Amiga Paula chip: 4-channel audio DMA state machine
//
// Each channel (0-3) has its own period counter, sample DMA pointer,
// length counter, and volume.  When enabled, it fetches one word (two
// 8-bit samples) per DMA period from chip RAM and generates an interrupt
// when the block length expires.
//
// Standard Amiga stereo routing:
//   Left  channel = ch0 + ch3
//   Right channel = ch1 + ch2
//
// Registers (per channel, offsets from 0xDFF0A0 / 0xDFF0B0 / 0xDFF0C0 / 0xDFF0D0):
//   +0x00 / +0x02 : AUDxLC  (32-bit DMA pointer; H/L swapped in chipram)
//   +0x04         : AUDxLEN (word count; reloaded on block end)
//   +0x06         : AUDxPER (period in DMA cycles; min 124 per HRM)
//   +0x08         : AUDxVOL (volume 0-64)
//   +0x0A         : AUDxDAT (manual/DMA data latch)
//
// DMACON bits 3:0 = AUD3EN..AUD0EN

#ifndef Paula_h
#define Paula_h

#include <stdint.h>
#include "Chipset.h"

// DMA cycles between audio handler invocations.
// PAL DMA rate ≈ 3.579 MHz; 80 cycles → ≈ 44744 Hz output rate.
#define AUD_SAMPLE_PERIOD  80

// Per-channel internal state
typedef struct {
    uint32_t lc_reload;    // AUDxLC value at DMA-enable time (reload after block end)
    uint32_t lc_current;   // current DMA pointer (advances 2 bytes per word fetch)
    uint16_t len_reload;   // AUDxLEN at DMA-enable time
    uint16_t len_remain;   // words remaining in current block
    uint16_t per_cnt;      // DMA cycles until next sample output (counts down)
    int8_t   sample_hi;    // high byte (output first) from last fetched word
    int8_t   sample_lo;    // low byte (output second) from last fetched word
    uint8_t  use_lo;       // 0 = output hi byte next, 1 = output lo byte
    uint8_t  running;      // 1 = DMA active for this channel
} AudChan_t;

extern AudChan_t g_aud[4];

// Call once after chipset init
void PaulaInit(void);

// Arm SLOT_AUDIO in the scheduler (called from sched_dma_init)
void sched_audio_init(void);

#endif /* Paula_h */
