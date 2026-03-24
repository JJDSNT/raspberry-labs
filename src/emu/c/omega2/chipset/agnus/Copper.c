//
//  Copper.c
//  Omega2
//
//  Created by Matt Parsons on 28/04/2022.
//

#include "Copper.h"
#include "Scheduler.h"
#include "Beam.h"
#include "omega_probe.h"
#include "omega_host.h"

static inline uint16_t copper_read_be16(const uint8_t *base, uint32_t addr)
{
    return ((uint16_t)base[addr] << 8) |
           ((uint16_t)base[addr + 1]);
}

static inline int copper_beam_past(uint16_t waitpos, uint16_t mask, uint16_t vhpos)
{
    const uint8_t vmask = (mask    >> 8) & 0xFF;
    const uint8_t vwait = (waitpos >> 8) & 0xFF;
    const uint8_t hmask =  mask         & 0xFE;
    const uint8_t hwait =  waitpos      & 0xFE;
    const uint8_t vcur  = (vhpos   >> 8) & 0xFF;
    const uint8_t hcur  =  vhpos         & 0xFE;

    if ((vcur & vmask) > (vwait & vmask)) {
        return 1;
    }

    if ((vcur & vmask) == (vwait & vmask)) {
        return ((hcur & hmask) >= (hwait & hmask));
    }

    return 0;
}

// ---------------------------------------------------------------------------
// copper_find_match — compute DMA cycles until the WAIT condition is met.
//
// Returns 0 if the condition is already satisfied.
// Returns the number of DMA cycles from NOW until the first cycle where
// (beam & mask) >= (waitpos & mask).
//
// Handles the common case (vmask=0xFF, hmask=0xFE) exactly.
// For unusual masks, returns 1 so the comparator is re-evaluated next cycle.
// ---------------------------------------------------------------------------
static uint64_t copper_find_match(
    uint8_t vwait, uint8_t hwait,
    uint8_t vmask, uint8_t hmask,
    uint8_t vcur,  uint8_t hcur)
{
    const uint8_t vtgt = vwait & vmask;
    const uint8_t htgt = hwait & hmask;
    const uint8_t vm   = vcur  & vmask;
    const uint8_t hm   = hcur  & hmask;

    if (vm > vtgt) {
        return 0;
    }

    if (vm == vtgt && hm >= htgt) {
        return 0;
    }

    if (vmask != 0xFF || hmask != 0xFE) {
        return 1;
    }

    if (vm == vtgt) {
        const uint32_t hcur_dma = hcur >> 1;
        const uint32_t htgt_dma = htgt >> 1;
        return (htgt_dma > hcur_dma) ? (uint64_t)(htgt_dma - hcur_dma) : 1;
    }

    {
        const uint32_t lines_left = (uint32_t)(vtgt - vm);
        const uint32_t hcur_dma   = hcur >> 1;
        const uint32_t htgt_dma   = htgt >> 1;
        const uint32_t hcnt       = (uint32_t)beam_hcnt();

        const uint64_t delta =
            (uint64_t)(hcnt - hcur_dma) +
            (uint64_t)(lines_left - 1) * hcnt +
            (uint64_t)htgt_dma;

        return delta + 1;
    }
}

void ExecuteCopper(Chipset_t *ChipsetState)
{
    switch (ChipsetState->CopperState) {

        case 0:
            // Load IR1
            ChipsetState->CopperIR1 =
                copper_read_be16(ChipsetState->chipram, ChipsetState->CopperPC);
            ChipsetState->CopperPC += 2;

            if (ChipsetState->CopperIR1 & 1) {
                ChipsetState->CopperState = 1; // WAIT or SKIP
            } else {
                ChipsetState->CopperState = 2; // MOVE
            }
            break;

        case 1:
            // Load IR2
            ChipsetState->CopperIR2 =
                copper_read_be16(ChipsetState->chipram, ChipsetState->CopperPC);
            ChipsetState->CopperPC += 2;

            {
                const uint16_t waitpos = ChipsetState->CopperIR1;
                const uint16_t mask    = ChipsetState->CopperIR2 | 0x0001;
                const uint16_t vhpos   = ChipsetState->VHPOS & 0xFFFF;

                const uint8_t vmask = (mask    >> 8) & 0xFF;
                const uint8_t vwait = (waitpos >> 8) & 0xFF;
                const uint8_t hmask =  mask         & 0xFE;
                const uint8_t hwait =  waitpos      & 0xFE;
                const uint8_t vcur  = (vhpos   >> 8) & 0xFF;
                const uint8_t hcur  =  vhpos         & 0xFE;

                const int beam_past = copper_beam_past(waitpos, mask, vhpos);

                // SKIP (IR2 bit 0 = 1)
                if (ChipsetState->CopperIR2 & 1) {
                    if (beam_past) {
                        ChipsetState->CopperPC += 4;
                    }
                    ChipsetState->CopperState = 0;
                    break;
                }

                // WAIT
                if (beam_past) {
                    ChipsetState->CopperState = 0;
                } else {
                    const uint64_t delta = copper_find_match(
                        vwait, hwait, vmask, hmask, vcur, hcur);

                    probe_emit(EVT_COPPER_WAIT, waitpos, mask);

                    ChipsetState->CopperIR1 = waitpos;
                    ChipsetState->CopperIR2 = mask;
                    ChipsetState->CopperState = 3;
                    ChipsetState->copper_wake_cycle = sched_clock() + delta;
                }
            }
            break;

        case 2:
            // Load IR2
            ChipsetState->CopperIR2 =
                copper_read_be16(ChipsetState->chipram, ChipsetState->CopperPC);
            ChipsetState->CopperPC += 2;

            {
                const uint16_t addr = ChipsetState->CopperIR1 & 0x1FE;
                int illegal;

#if defined(CHIPSET_ECS)
                illegal = (!ChipsetState->CopperCDANG && addr < 0x80);
#else
                illegal = (addr < 0x40) || (!ChipsetState->CopperCDANG && addr < 0x80);
#endif

                if (illegal) {
                    ChipsetState->CopperState = 4;
                    return;
                }

                probe_emit(EVT_COPPER_MOVE, addr, ChipsetState->CopperIR2);
                ChipsetState->WriteWord[addr](ChipsetState->CopperIR2);
            }

            ChipsetState->CopperState = 0;
            break;

        case 3:
            // WAIT state
            if (sched_clock() < ChipsetState->copper_wake_cycle) {
                break;
            }

            {
                const uint16_t waitpos = ChipsetState->CopperIR1;
                const uint16_t mask    = ChipsetState->CopperIR2;
                const uint16_t vhpos   = ChipsetState->VHPOS & 0xFFFF;

                if (copper_beam_past(waitpos, mask, vhpos)) {
                    ChipsetState->CopperState = 0;
                } else {
                    const uint8_t vmask = (mask    >> 8) & 0xFF;
                    const uint8_t vwait = (waitpos >> 8) & 0xFF;
                    const uint8_t hmask =  mask         & 0xFE;
                    const uint8_t hwait =  waitpos      & 0xFE;
                    const uint8_t vcur  = (vhpos   >> 8) & 0xFF;
                    const uint8_t hcur  =  vhpos         & 0xFE;

                    const uint64_t delta = copper_find_match(
                        vwait, hwait, vmask, hmask, vcur, hcur);

                    ChipsetState->copper_wake_cycle =
                        sched_clock() + (delta ? delta : 1);
                }
            }
            break;

        case 4:
            // HALT until next VBL where COP1LC will be reloaded.
            break;
    }
}