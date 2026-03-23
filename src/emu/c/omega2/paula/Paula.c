// Paula.c — Amiga Paula 4-channel audio DMA state machine

#include "Paula.h"
#include "Scheduler.h"
#include "omega_host.h"
#include <stdint.h>

extern Chipset_t* ChipsetState;

AudChan_t g_aud[4];

// Chipram base addresses for each channel (after H/L pointer swap).
// *(uint32_t*)&chipram[AUD_BASE[n]] = (AUDnLCH<<16)|AUDnLCL = 32-bit pointer.
static const uint32_t AUD_BASE[4] = {
    0xDFF0A0, 0xDFF0B0, 0xDFF0C0, 0xDFF0D0
};

// DMACON AUDxEN bits
static const uint16_t AUD_DMAEN[4] = { 0x01, 0x02, 0x04, 0x08 };

// INTREQ audio interrupt bits (level 3: AUD0=bit7, AUD1=bit8, AUD2=bit9, AUD3=bit10)
static const uint16_t AUD_INTBIT[4] = { 0x0080, 0x0100, 0x0200, 0x0400 };

void PaulaInit(void) {
    for (int i = 0; i < 4; i++) {
        g_aud[i].lc_reload  = 0;
        g_aud[i].lc_current = 0;
        g_aud[i].len_reload = 1;
        g_aud[i].len_remain = 1;
        g_aud[i].per_cnt    = AUD_SAMPLE_PERIOD;
        g_aud[i].sample_hi  = 0;
        g_aud[i].sample_lo  = 0;
        g_aud[i].use_lo     = 0;
        g_aud[i].running    = 0;
    }
}

// ---------------------------------------------------------------------------
// Fetch one word from chip RAM for channel n; advance pointer; handle reload.
// ---------------------------------------------------------------------------
static void aud_fetch(int n) {
    AudChan_t*      ch    = &g_aud[n];
    uint8_t*        ram   = ChipsetState->chipram;
    uint32_t        addr  = ch->lc_current & 0x1FFFFEu; // word-align, chip RAM mask
    uint16_t        word;

    // Read big-endian word from chip RAM (stored as LE on ARM → byte-swap)
    word = (uint16_t)((ram[addr] << 8) | ram[addr + 1]);

    ch->sample_hi  = (int8_t)(word >> 8);
    ch->sample_lo  = (int8_t)(word & 0xFF);
    ch->lc_current += 2;
    ch->use_lo      = 0;    // start with hi byte

    // Decrement length; when exhausted: reload and fire interrupt
    if (ch->len_remain > 0) ch->len_remain--;
    if (ch->len_remain == 0) {
        // Reload from register values written by software
        uint32_t base     = AUD_BASE[n];
        ch->lc_reload  = *(uint32_t*)&ram[base];          // (LCH<<16)|LCL
        ch->len_reload = *(uint16_t*)&ram[base + 0x04];   // AUDxLEN
        if (ch->len_reload == 0) ch->len_reload = 1;

        ch->lc_current = ch->lc_reload;
        ch->len_remain = ch->len_reload;

        // Generate audio interrupt (SET bit, INTENA must allow level 3)
        ChipsetState->WriteWord[0x9C](0x8000u | AUD_INTBIT[n]);
    }
}

// ---------------------------------------------------------------------------
// SLOT_AUDIO handler — called every AUD_SAMPLE_PERIOD DMA cycles
// ---------------------------------------------------------------------------
static void sched_audio_handler(void) {
    if (!ChipsetState) {
        sched_schedule(SLOT_AUDIO, AUD_SAMPLE_PERIOD, sched_audio_handler);
        return;
    }

    uint16_t dmacon = ChipsetState->DMACONR;
    int      dma_master = (dmacon >> 9) & 1;   // DMAEN master enable (bit 9)
    int16_t  out_left   = 0;
    int16_t  out_right  = 0;

    for (int n = 0; n < 4; n++) {
        AudChan_t* ch       = &g_aud[n];
        int        chan_dma = dma_master && (dmacon & AUD_DMAEN[n]);

        // Detect DMA enable edge: start channel
        if (chan_dma && !ch->running) {
            uint32_t base     = AUD_BASE[n];
            uint8_t* ram      = ChipsetState->chipram;

            ch->lc_reload  = *(uint32_t*)&ram[base];
            ch->len_reload = *(uint16_t*)&ram[base + 0x04];
            if (ch->len_reload == 0) ch->len_reload = 1;

            ch->lc_current = ch->lc_reload;
            ch->len_remain = ch->len_reload;
            ch->use_lo     = 0;
            ch->running    = 1;
            aud_fetch(n);   // prefetch first word
        } else if (!chan_dma) {
            ch->running = 0;
        }

        if (!ch->running) continue;

        // Advance period counter
        if (ch->per_cnt > AUD_SAMPLE_PERIOD) {
            ch->per_cnt -= AUD_SAMPLE_PERIOD;
            // output the current sample without advancing
        } else {
            ch->per_cnt = *(uint16_t*)&ChipsetState->chipram[AUD_BASE[n] + 0x06];
            if (ch->per_cnt < 124) ch->per_cnt = 124;  // HRM minimum

            // Output current sample byte
            if (!ch->use_lo) {
                ch->use_lo = 1;
            } else {
                ch->use_lo = 0;
                aud_fetch(n);  // consumed both bytes: fetch next word
            }
        }

        // Mix sample: volume 0-64, sample is signed 8-bit
        uint8_t vol = (uint8_t)(*(uint16_t*)&ChipsetState->chipram[AUD_BASE[n] + 0x08] & 0x7F);
        if (vol > 64) vol = 64;
        int16_t s = (int16_t)(ch->use_lo ? ch->sample_hi : ch->sample_lo);
        int16_t scaled = (int16_t)((int32_t)s * vol);  // max ±64*127 = ±8128

        // Standard Amiga routing: ch0,ch3 = left; ch1,ch2 = right
        if (n == 0 || n == 3) out_left  = (int16_t)(out_left  + scaled);
        else                   out_right = (int16_t)(out_right + scaled);
    }

    // Scale to 16-bit range: ±8128*2 → multiply by 4 → ±65024 (fits in int16)
    omega_host_audio_sample((int16_t)(out_left << 2), (int16_t)(out_right << 2));

    sched_schedule(SLOT_AUDIO, AUD_SAMPLE_PERIOD, sched_audio_handler);
}

void sched_audio_init(void) {
    PaulaInit();
    sched_schedule(SLOT_AUDIO, AUD_SAMPLE_PERIOD, sched_audio_handler);
}
