//
//  Copper.c
//  Omega2
//
//  Created by Matt Parsons on 28/04/2022.
//

#include "Copper.h"
#include "omega_probe.h"
#include "omega_host.h"

uint16_t swap(uint16_t* p){
    return (*p >> 8) | (*p << 8);
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
            // Wait state — reavalia a condição a cada ciclo
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

                if (beam_past) ChipsetState->CopperState = 0;
            }
            break;
            
        case 4:
            //HALT until next VBL where COP1LOC will be reloaded.
            break;
            
    }
    
}
