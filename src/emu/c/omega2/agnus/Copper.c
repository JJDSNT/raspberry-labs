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

uint16_t swap(uint16_t* p){
    return (*p >> 8) | (*p << 8);
}

// ---------------------------------------------------------------------------
// copper_find_match — compute DMA cycles until the WAIT condition is met.
//
// Returns 0 if the condition is already satisfied (caller should not enter
// state 3 in this case, but 0 is safe).  Returns the number of DMA cycles
// from NOW until the first cycle where (beam & mask) >= (waitpos & mask).
//
// Handles the common case (vmask=0xFF, hmask=0xFE) exactly.  For unusual
// masks, returns 1 so the comparator is re-evaluated next cycle (correct,
// just not optimal).
// ---------------------------------------------------------------------------
static uint64_t copper_find_match(
    uint8_t vwait, uint8_t hwait,
    uint8_t vmask, uint8_t hmask,
    uint8_t vcur,  uint8_t hcur)
{
    uint8_t vtgt = vwait & vmask;
    uint8_t htgt = hwait & hmask;
    uint8_t vm   = vcur  & vmask;
    uint8_t hm   = hcur  & hmask;

    // Already past?
    if (vm > vtgt) return 0;
    if (vm == vtgt && hm >= htgt) return 0;

    // Unusual mask: give up on exact prediction, try again next cycle.
    if (vmask != 0xFF || hmask != 0xFE) return 1;

    // Same masked line: just advance within the line.
    // hwait is in beam units where bit 0 is always clear (0xFE mask); each
    // unit = 2 colour clocks = 1 DMA cycle.  htgt >> 1 = DMA cycle in line.
    if (vm == vtgt) {
        uint32_t hcur_dma = hcur >> 1;
        uint32_t htgt_dma = htgt >> 1;
        return (htgt_dma > hcur_dma) ? (uint64_t)(htgt_dma - hcur_dma) : 1;
    }

    // Different line: count remaining cycles in this line + full lines to
    // vwait + cycles into the target line.
    uint32_t lines_left  = (uint32_t)(vtgt - vm);           // lines still to go
    uint32_t hcur_dma    = hcur >> 1;
    uint32_t htgt_dma    = htgt >> 1;
    uint32_t hcnt        = (uint32_t)beam_hcnt();           // DMA cycles per line
    uint64_t delta       = (uint64_t)(hcnt - hcur_dma)      // rest of current line
                         + (uint64_t)(lines_left - 1) * hcnt // full lines between
                         + (uint64_t)htgt_dma;               // position in target line
    return delta + 1; // +1: ensure we're at or past the target when we wake
}

void ExecuteCopper(Chipset_t* ChipsetState){
    
    // Cycle 1: Load IR1
    // Cycle 2: Load IR2
    //          Decode IR1
    // Cycle 3: Write Back
    
    
   // if(ChipsetState->CopperPC == 0 || ChipsetState->CopperPC > 2097151){
       //ChipsetState->CopperState = 4;
  //  }
    
    
    uint32_t VHPOS = ChipsetState->VHPOS & 0xFFFF; //ignore the top bit
    switch(ChipsetState->CopperState){
            
        case 0:
            //Load IR1
            ChipsetState->CopperIR1 = swap((uint16_t*)&ChipsetState->chipram[ChipsetState->CopperPC]);
            ChipsetState->CopperPC += 2;
            
            
            if( ChipsetState->CopperIR1 & 1){
                ChipsetState->CopperState = 1;  // Wait or Skip
            }else{
                ChipsetState->CopperState = 2; // it's a Move instruction
            }
            break;
            
            
            
        case 1:
            //Load IR2
            ChipsetState->CopperIR2 = swap((uint16_t*)&ChipsetState->chipram[ChipsetState->CopperPC]);
            ChipsetState->CopperPC += 2;

            {
                uint16_t waitpos = ChipsetState->CopperIR1;
                uint16_t mask    = ChipsetState->CopperIR2 | 0x0001; // bit 0 always set per HRM

                // Comparador V/H separado com máscara (referência: vAmiga Copper::runComparator)
                uint8_t vmask = (mask    >> 8) & 0xFF;
                uint8_t vwait = (waitpos >> 8) & 0xFF;
                uint8_t hmask = mask    & 0xFE;
                uint8_t hwait = waitpos & 0xFE;
                uint8_t vcur  = (VHPOS  >> 8) & 0xFF;
                uint8_t hcur  = VHPOS   & 0xFE;

                int beam_past = 0;
                if      ((vcur & vmask) > (vwait & vmask)) beam_past = 1;
                else if ((vcur & vmask) == (vwait & vmask))
                    beam_past = ((hcur & hmask) >= (hwait & hmask));

                // Skip instruction (IR2 bit 0 = 1)
                if (ChipsetState->CopperIR2 & 1) {
                    if (beam_past) ChipsetState->CopperPC += 4;
                    ChipsetState->CopperState = 0;
                    break;
                }

                // Wait instruction
                if (beam_past) {
                    ChipsetState->CopperState = 0;
                } else {
                    probe_emit(EVT_COPPER_WAIT, waitpos, mask);
                    ChipsetState->CopperIR1 = waitpos;
                    ChipsetState->CopperIR2 = mask;
                    ChipsetState->CopperState = 3;
                    // findMatch: pre-compute the earliest DMA cycle when the
                    // condition can be true so case 3 skips the spin entirely.
                    uint64_t delta = copper_find_match(
                        vwait, hwait, vmask, hmask, vcur, hcur);
                    ChipsetState->copper_wake_cycle = sched_clock() + delta;
                }
            }
            break;
        
            
            
            
        case 2:
            //Load IR2
            ChipsetState->CopperIR2 = swap((uint16_t*)&ChipsetState->chipram[ChipsetState->CopperPC]);
            ChipsetState->CopperPC += 2;

            {
                uint16_t addr = ChipsetState->CopperIR1 & 0x1FE; // word-aligned register offset
                // Illegal address check (referência: vAmiga Copper::isIllegalAddress)
                // CDANG=0: bloqueia addr < 0x80
                // CDANG=1 + ECS: sem restrição
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
            // Wait state — skip evaluation until copper_wake_cycle is reached.
            // findMatch() set wake_cycle when we entered this state; checking
            // sched_clock() here costs one compare per DMA cycle instead of the
            // full comparator, reducing work by ~10× for typical wait intervals.
            if (sched_clock() < ChipsetState->copper_wake_cycle) break;

            {
                uint16_t waitpos = ChipsetState->CopperIR1;
                uint16_t mask    = ChipsetState->CopperIR2;
                uint8_t vmask = (mask    >> 8) & 0xFF;
                uint8_t vwait = (waitpos >> 8) & 0xFF;
                uint8_t hmask = mask    & 0xFE;
                uint8_t hwait = waitpos & 0xFE;
                uint8_t vcur  = (VHPOS  >> 8) & 0xFF;
                uint8_t hcur  = VHPOS   & 0xFE;

                int beam_past = 0;
                if      ((vcur & vmask) > (vwait & vmask)) beam_past = 1;
                else if ((vcur & vmask) == (vwait & vmask))
                    beam_past = ((hcur & hmask) >= (hwait & hmask));

                if (beam_past) {
                    ChipsetState->CopperState = 0;
                } else {
                    // Woke too early (e.g. masked wait or off-by-one) — recompute.
                    uint64_t delta = copper_find_match(
                        vwait, hwait, vmask, hmask, vcur, hcur);
                    ChipsetState->copper_wake_cycle = sched_clock() + (delta ? delta : 1);
                }
            }
            break;
            
        case 4:
            //HALT until next VBL where COP1LOC will be reloaded.
            break;
            
    }
    
}
