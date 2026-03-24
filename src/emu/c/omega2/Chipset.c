//
//  Chipset.c
//  Omega2
//
//  Created by Matt Parsons on 29/03/2022.
//

#include "Chipset.h"
#include "Memory.h"
#include "m68k.h"
#include "CIA.h"
#include "Beam.h"
#include "Scheduler.h"
#include "omega_probe.h"
#include "omega_host.h"

//remove when we have an internal memory allocation system
#include <stdlib.h>

Chipset_t* ChipsetState;

uint16_t ByteSwap16(uint16_t value){
    return (value >> 8) | (value << 8);
}


// 20
void DSKPTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF022];
    *p = value;
    return;
}

// 22
void DSKPTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF020];
    *p = value;
    return;
}





// 24
void DSKLEN(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF024];
    *p = value;
    return;
}

// 2A — VPOSW: bit0 = VPOS[8], bit15 = LOF write
void VPOSW(uint16_t value){
    // bit0 of value = VPOS[8] (MSB of 9-bit vertical counter)
    g_beam.v = (g_beam.v & 0xFF) | ((value & 0x01) << 8);
    g_beam.lof = (value >> 15) & 1;
    ChipsetState->VHPOS = beam_vhpos();
}

// 2C — VHPOSW: bits[15:8] = VPOS[7:0], bits[7:0] = HPOS[7:0]
void VHPOSW(uint16_t value){
    g_beam.v = (g_beam.v & 0x100) | ((value >> 8) & 0xFF);
    g_beam.h = value & 0xFF;
    ChipsetState->VHPOS = beam_vhpos();
}

// 30
void SERDAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF30];
    *p = value;
    return;
}

// 32
void SERPER(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF032];
    *p = value;
    return;
}

// 34
void POTGO(uint16_t value){
    ChipsetState->POTGOR = value;
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF016];
     *p = ByteSwap16(value);
}

// 36
void JOYTEST(uint16_t value){
    ChipsetState->POTGOR = value;
    uint16_t* JOY0DAT= (uint16_t*)&RAM24bit[0xDFF0A];
     *JOY0DAT = ByteSwap16(value);
    
    uint16_t* JOY1DAT= (uint16_t*)&RAM24bit[0xDFF0C];
     *JOY1DAT = ByteSwap16(value);
}

// 38
void STREQ(uint16_t value){
    ChipsetState->POTGOR = value;
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF038];
     *p = ByteSwap16(value);
    printf("Chipset: NOT IMPLEMENTED - Strobe for Horisontal sync with VB and EQU\n");
}

// 3A
void STRVBL(uint16_t value){
    ChipsetState->POTGOR = value;
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF03A];
     *p = ByteSwap16(value);
    printf("Chipset: NOT IMPLEMENTED - Strobe for Horisontal sync with VB\n");
}

// 3C
void STRHOR(uint16_t value){
    ChipsetState->POTGOR = value;
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF03A];
     *p = ByteSwap16(value);
    printf("Chipset: NOT IMPLEMENTED - Strobe for Horisontal sync\n");
}

// 3E
void STRLONG(uint16_t value){
    ChipsetState->POTGOR = value;
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF03E];
     *p = ByteSwap16(value);
    printf("Chipset: NOT IMPLEMENTED - Strobe for Identifiction of long horizontal line\n");
}


// 40
void BLTCON0(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF040];
    *p = value;
    return;
}

// 42
void BLTCON1(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF042];
    *p = value;
    return;
}


// 44
void BLTAFWM(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF044];
    *p = value;
    return;
}

// 46
void BLTALWM(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF046];
    *p = value;
    return;
}



// 48
void BLTCPTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF04A];
    *p = value;
    return;
}

// 4A
void BLTCPTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF048];
    *p = value;
    return;
}



// 4C
void BLTBPTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF04E];
    *p = value;
    return;
}

// 4E
void BLTBPTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF04C];
    *p = value;
    return;
}



// 50
void BLTAPTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF052];
    *p = value;
    return;
}

// 52
void BLTAPTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF050];
    *p = value;
    return;
}




// 54
void BLTDPTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF056];
    *p = value;
    return;
}

// 56
void BLTDPTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF054];
    *p = value;
    return;
}




// 58
void BLTSIZE(uint16_t value){
//Register not used Please see BLTSIZV and BLTSIZH
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF058];
    *p = value; //Save a copy here anyway...
    
    
    uint16_t* sizv = (uint16_t*) &RAM24bit[0xDFF05C];
    uint16_t* sizh = (uint16_t*) &RAM24bit[0xDFF05E];
    
    *sizv = value >> 6;
    *sizh = value & 0x3F;
    
    //Not sure why this is here? perhaps remove it? I had it in the old emulator so I'm keeping it for now
    if(*sizv == 0){
        *sizv=1024;
    }

    if(*sizh == 0){
        *sizh=64;
    }
    
    p = (uint16_t*) &RAM24bit[0xDFF002];
    *p = *p | 0x0040; // Set Big endian Blitter busy bit
    ChipsetState->DMACONR = ChipsetState->DMACONR | 0x4000; // Set Little endian Blitter busy bit
     
    return;
}

// 5A
void BLTCON0L(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF05A];
    *p = value;
    return;
}

// 5C
void BLTSIZV(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF05C];
    *p = value;
    return;
}

// 5E
void BLTSIZH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF05E];
    *p = value;
    
    
    p = (uint16_t*) &RAM24bit[0xDFF002];
    *p = *p | 0x40; // Set Big endian Blitter busy bit
    ChipsetState->DMACONR = ChipsetState->DMACONR | 0x4000; // Set Little endian Blitter busy bit
    
    return;
}

// 60
void BLTCMOD(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF060];
    *p = value;
    return;
}

// 62
void BLTBMOD(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF062];
    *p = value;
    return;
}

// 64
void BLTAMOD(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF064];
    *p = value;
    return;
}

// 66
void BLTDMOD(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF066];
    *p = value;
    return;
}


// 70
void BLTCDAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF070];
    *p = value;
    return;
}

// 72
void BLTBDAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF072];
    *p = value;
    return;
}

// 74
void BLTADAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF074];
    *p = value;
    return;
}


// 7E
void DSKSYNC(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF07E];
    *p = value;
    return;
}


void DecodeCopperList(int list){
    
    uint32_t address = 0;
    
    if(list == 2){
        address = *(uint32_t*) &RAM24bit[0xDFF084];
    }else{
        address = *(uint32_t*) &RAM24bit[0xDFF080];
    }
    
    if(address == 0){return;}
    
    
    
    for(int i = address; i<(address+512); i += 4){
        
        uint32_t instruction =  ByteSwap16(*(uint16_t*)&RAM24bit[i]);
        uint16_t value =  ByteSwap16(*(uint16_t*)&RAM24bit[i+2]);
        
        if(instruction & 1){
            
            if(value & 1){
                printf("Address: 0x%x - Skip 0x%04x (Mask: 0x%04x)\n",i,instruction, value);
            }else{
                printf("Address: 0x%x - Wait 0x%04x (Mask: 0x%04x)\n",i,instruction, value);
            }
                
        }else{
            
            if(instruction <0x80){
                printf("HALT\n");
                break;
            }
            
            if(instruction == 0x8A){
                printf("Address: 0x%x - JMP COP2\n",i);
                break;
            }else if(instruction == 0x88){
                printf("Address: 0x%x - JMP COP1\n",i);
                break;
            }else{
                instruction += 0xDFF000;
                //printf("Address: 0x%x - Move 0x%04x  -> 0x%x\n",i, value, instruction);
                printf("Address: 0x%x - Move 0x%04x -> %s\n",i, value, regNames[ (instruction & 0x1FF) >> 1 ]);
            }
        }
        

        
    }
    
    return;
    
}

// 80
void COP1LCH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF082];
    *p = value;
    probe_emit(EVT_CUSTOM_WRITE, 0x80 /*COP1LCH*/, value);
    return;
}

// 82
void COP1LCL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF080];
    *p = value;
    probe_emit(EVT_CUSTOM_WRITE, 0x82 /*COP1LCL*/, value);
    return;
}



// 84
void COP2LCH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF086];
    *p = value;
    return;
}

// 86
void COP2LCL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF084];
    *p = value;
    
    return;
}



// 88
// 3C — COPCON: bit 1 = CDANG (Copper dangerous access enable)
void COPCON(uint16_t value){
    ChipsetState->CopperCDANG = (value >> 1) & 1;
}

void COPJMP1(uint16_t value){
    //Load the Copper with COP1LC
    uint32_t* p = (uint32_t*)&RAM24bit[0xDFF080];
    ChipsetState->CopperPC = *p;
    ChipsetState->CopperState = 0;  //Reset the Copper.
    ChipsetState->copper_wake_cycle = 0;
    probe_emit(EVT_CUSTOM_WRITE, 0x88 /*COPJMP1*/, ChipsetState->CopperPC);
}

// 8A
void COPJMP2(uint16_t value){
    //Load the Copper with COP2LC
    uint32_t* p = (uint32_t*)&RAM24bit[0xDFF084];
    ChipsetState->CopperPC = *p;
    ChipsetState->CopperState = 0;  //Reset the Copper.
    ChipsetState->copper_wake_cycle = 0;
}


// 8E
void DIWSTRT(uint16_t value){

    ChipsetState->HSTART = (value & 0xFF);
    
    if( (value & 0xFF00) < 0x1400){
        value &= 0xFF;
        value |= 0x1400;
    }
       
       
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF08E];
    *p = value;
}

// 90
void DIWSTOP(uint16_t value){
    
    //if the top bit is not set then add 256 to the value
    if(value & 0x8000){
        ChipsetState->VSTOP = (value & 0xFF00);
    }else{
        ChipsetState->VSTOP = (value & 0xFF00) | 0x10000;
    }

    ChipsetState->HSTOP = ((value & 0xFF) | 0x100) - 88;
    
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF090];
    *p = value;
}


// 92
void DFFSTRT(uint16_t value){
    
    //The display can't be generated before this point
    if( value < 0x18){
        value = 0x18;
    }
    
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF092];
    *p = value & 0xFC;
}

// 94
void DFFSTOP(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF094];
    *p = value & 0xFC;
}

// 96
void DMACON(uint16_t value){
    if(value & 32768){
        ChipsetState->DMACONR = ChipsetState->DMACONR | (value & 0x7FF);
    }else{
        ChipsetState->DMACONR = ChipsetState->DMACONR ^ ((value & 0x7FF) & ChipsetState->DMACONR);
    }

    //Save big endian copy in RAM for the CPU to read
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF002];
    *p = ByteSwap16(ChipsetState->DMACONR);
    probe_emit(EVT_CUSTOM_WRITE, 0x96 /*DMACON*/, ChipsetState->DMACONR);
    return;
}

// 9A
void INTENA(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF01C];

    if(value & 32768){
        ChipsetState->INTENAR = ChipsetState->INTENAR | (value & 32767);
    }else{
        ChipsetState->INTENAR = ChipsetState->INTENAR ^ (value & ChipsetState->INTENAR);
    }

    //Save big endian copy in RAM for the CPU to read
    *p = ByteSwap16(ChipsetState->INTENAR);

    // Also call CheckInterrupts so pending INTREQR bits fire immediately when enabled
    CheckInterrupts();
    return;
}


void CheckInterrupts(){

    //Generate interrupts only if INTENA Master bit is set
    if( !(ChipsetState->INTENAR & 0x4000) ){
        m68k_set_irq(0);
        return;
    }

    // Strictly intreq & intena — matches vAmiga Paula::interruptLevel().
    // No special-casing for VBL or PORTS: if the OS didn't enable them, they don't fire.
    uint16_t intMask = ChipsetState->INTREQR & ChipsetState->INTENAR;

    // Edge-trigger: emit probe only when IRQ level changes.
    // m68k_set_irq() is always called (including 0) so Musashi sees the correct line state.
    static unsigned last_irq_level = 0;
    unsigned level = 0;
    if (intMask != 0) {
        level =
            (intMask & 8192) ? 6 :
            (intMask & 4096) ? 5 :
            (intMask & 2048) ? 5 :
            (intMask & 1024) ? 4 :
            (intMask &  512) ? 4 :
            (intMask &  256) ? 4 :
            (intMask &  128) ? 4 :
            (intMask &   64) ? 3 :
            (intMask &   32) ? 3 :
            (intMask &   16) ? 3 :
            (intMask &    8) ? 2 :
            (intMask &    4) ? 1 :
            (intMask &    2) ? 1 : 1;
    }
    if (level != last_irq_level) {
        probe_emit(EVT_INTR_FIRE, level, ChipsetState->INTREQR);
        last_irq_level = level;
    }
    m68k_set_irq(level);
}

// 9C
void INTREQ(uint16_t value){
    if(value & 32768){
        ChipsetState->INTREQR = ChipsetState->INTREQR | (value & 32767);
    }else{
        ChipsetState->INTREQR = ChipsetState->INTREQR ^ (value & ChipsetState->INTREQR);
    }

    //Save big endian copy in RAM for the CPU to read
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF01E];
    *p = ByteSwap16(ChipsetState->INTREQR);

    CheckInterrupts();
    return;
}

// 9E
void ADKCON(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF09E];
    *p = value;
}




// A0
void AUD0LCH(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0A2];
    *p = value;
    return;
}

// A2
void AUD0LCL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0A0];
    *p = value;
    return;
}

// A4
void AUD0LEN(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0A4];
    *p = value;
    return;
}

// A6
void AUD0PER(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0A6];
    *p = value;
    return;
}

// A8
void AUD0VOL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0A8];
    *p = value;
    return;
}


// AA
void AUD0DAT(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0AA];
    *p = value;
    return;
}



// ---------------------------------------------------------------------------
// Audio channels 1-3 — LCH/LCL/LEN/PER/VOL/DAT write handlers
// H/L swap convention: writing LCH stores at base+2, LCL at base+0
// so *(uint32_t*)&chipram[base] = (LCH<<16)|LCL = correct 32-bit pointer
// ---------------------------------------------------------------------------

// Channel 1 (base 0xDFF0B0)
void AUD1LCH(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0B2] = value; }
void AUD1LCL(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0B0] = value; }
void AUD1LEN(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0B4] = value; }
void AUD1PER(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0B6] = value; }
void AUD1VOL(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0B8] = value; }
void AUD1DAT(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0BA] = value; }

// Channel 2 (base 0xDFF0C0)
void AUD2LCH(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0C2] = value; }
void AUD2LCL(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0C0] = value; }
void AUD2LEN(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0C4] = value; }
void AUD2PER(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0C6] = value; }
void AUD2VOL(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0C8] = value; }
void AUD2DAT(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0CA] = value; }

// Channel 3 (base 0xDFF0D0)
void AUD3LCH(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0D2] = value; }
void AUD3LCL(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0D0] = value; }
void AUD3LEN(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0D4] = value; }
void AUD3PER(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0D6] = value; }
void AUD3VOL(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0D8] = value; }
void AUD3DAT(uint16_t value){ *(uint16_t*)&RAM24bit[0xDFF0DA] = value; }


// E0
void BPL1PTH(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0E2];
    *p = value;
    return;
}

// E2
void BPL1PTL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0E0];
    *p = value;
    return;
}

// E4
void BPL2PTH(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0E6];
    *p = value;
    return;
}

// E6
void BPL2PTL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0E4];
    *p = value;
    return;
}

// E8
void BPL3PTH(uint16_t value){
    
   // value &= 0x7; //Top three bits only
    
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0EA];
    *p = value;
    return;
}

// EA
void BPL3PTL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0E8];
    *p = value;
    return;
}


// EC
void BPL4PTH(uint16_t value){
    
   // value &= 0x7; //Top three bits only
    
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0EE];
    *p = value;
    return;
}

// EE
void BPL4PTL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0EC];
    *p = value;
    return;
}

// F0
void BPL5PTH(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0F2];
    *p = value;
}

// F2
void BPL5PTL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0F0];
    *p = value;
}


// F4
void BPL6PTH(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0F6];
    *p = value;
}

// F6
void BPL6PTL(uint16_t value){
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF0F4];
    *p = value;
}


// 100
void BPLCON0(uint16_t value){
    ChipsetState->hires = value >> 15;
    ChipsetState->planeCount = (value >> 12) & 0x7;

    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF100];
    *p = value;
    probe_emit(EVT_CUSTOM_WRITE, 0x100 /*BPLCON0*/, value);
    return;
}

// 102
void BPLCON1(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF102];
    *p = value;
    return;
}

// 104
void BPLCON2(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF104];
    *p = value;
    return;
}

// 106
void BPLCON3(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF106];
    *p = value;
    return;
}

//108
void BPL1MOD(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF108];
    *p = value;
    return;
}

//10A
void BPL2MOD(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF10A];
    *p = value;
    return;
}

// 10C
void UNKNOWN10C(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF10C];
    *p = value;
    return;
}

// 110
void BPL1DAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF110];
    *p = value;
    return;
}

// 112
void BPL2DAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF112];
    *p = value;
    return;
}

// 114
void BPL3DAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF114];
    *p = value;
    return;
}

// 116
void BPL4DAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF116];
    *p = value;
    return;
}


// 118
void BPL5DAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF118];
    *p = value;
    return;
}

// 11A
void BPL6DAT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF11A];
    *p = value;
    return;
}

// 120
void SPR0PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF122];
    *p = value;
    return;
}

// 122
void SPR0PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF120];
    *p = value;
    
    //Calculate Vertical Position of Sprite
    uint32_t* data   = (uint32_t*) &ChipsetState->chipram[0xDFF120];
    uint16_t* datst = (uint16_t*) &ChipsetState->chipram[*data];
    
    ChipsetState->sprite[0].VPOS =  ((datst[0] & 0xFF) << 8)   |  ((datst[1] & 0x400) << 6);
    
    if(ChipsetState->sprite[0].VPOS < 0x1600){
        ChipsetState->sprite[0].VPOS = 79874; //Sprite will never activate
    }
    
    ChipsetState->sprite[0].data = &datst[2];
    
    ChipsetState->sprite[0].stop = ((((datst[1] &0xFF)  | ((datst[1] &0x200) >> 1)) ) << 8);
    
    return;
}


// 124
void SPR1PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF126];
    *p = value;
    return;
}

// 126
void SPR1PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF124];
    *p = value;
    return;
}


// 128
void SPR2PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF12A];
    *p = value;
    return;
}

// 12A
void SPR2PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF128];
    *p = value;
    return;
}

// 12C
void SPR3PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF12E];
    *p = value;
    return;
}

// 12E
void SPR3PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF12C];
    *p = value;
    return;
}


// 130
void SPR4PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF132];
    *p = value;
    return;
}

// 132
void SPR4PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF130];
    *p = value;
    return;
}


// 134
void SPR5PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF136];
    *p = value;
    return;
}

// 136
void SPR5PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF134];
    *p = value;
    return;
}

// 138
void SPR6PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF13A];
    *p = value;
    return;
}

// 13A
void SPR6PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF138];
    *p = value;
    return;
}


// 13C
void SPR7PTH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF13E];
    *p = value;
    return;
}

// 13E
void SPR7PTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF13C];
    *p = value;
    return;
}

// 140
void SPR0POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF140];
    *p = value;
    return;
}

// 142
void SPR0CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF142];
    *p = value;
    return;
}

// 148
void SPR1POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF148];
    *p = value;
    return;
}

// 14A
void SPR1CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF14A];
    *p = value;
    return;
}

// 150
void SPR2POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF150];
    *p = value;
    return;
}

// 152
void SPR2CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF152];
    *p = value;
    return;
}

// 158
void SPR3POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF158];
    *p = value;
    return;
}

// 15A
void SPR3CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF15A];
    *p = value;
    return;
}

// 160
void SPR4POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF160];
    *p = value;
    return;
}

// 162
void SPR4CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF162];
    *p = value;
    return;
}


void SPR4DATA(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF164];
    *p = value;
    return;
}

// 168
void SPR5POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF168];
    *p = value;
    return;
}

// 16A
void SPR5CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF16A];
    *p = value;
    return;
}

// 16E
void SPR5DATB(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF16E];
    *p = value;
    return;
}

// 170
void SPR6POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF170];
    *p = value;
    return;
}

// 172
void SPR6CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF172];
    *p = value;
    return;
}

// 178
void SPR7POS(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF178];
    *p = value;
    return;
}

// 17A
void SPR7CTL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF17A];
    *p = value;
    return;
}


uint32_t RGB12(uint16_t color){
    
    /*
    uint32_t b = value & 0xF;
    uint32_t g = (value & 0xF0) >> 4;
    uint32_t r = (value & 0xF00) >> 8;
    
    r = (r << 4) + r;
    g = (g << 4) + g;
    b = (b << 4) + b;
    
    return ((r << 16) | (g << 8) | b ) | 0xFF000000; //0xFF000000 = no opacity in the ARGB buffer
     */
    
    // My original Optimised OCS2ARGB function
    uint32_t value  =  ((color <<12)& 0xF00000) | ((color <<8)&0xF000) | ((color << 4)&0xF0); // high nybble.
    value |= (value >> 4);// low nybble
    value  = value | 0xFF000000; //opaque alpha
    return value;
    
}

// 180
void COLOR00(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF180];
    
    //Debugging check if the value has changed!
    if(*p != value){
       // printf("Colour Value 00 Changed to:%x\n",value);
       // ChipsetState->VBL = 1;
    }
    
    *p = value;
    
    ChipsetState->Colour[0] = RGB12(value);
    return;
}

// 182
void COLOR01(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF182];
    *p = value;
                                        
    ChipsetState->Colour[1] = RGB12(value);
    return;
}

// 184
void COLOR02(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF184];
    *p = value;
    
    ChipsetState->Colour[2] = RGB12(value);
    return;
}

// 186
void COLOR03(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF186];
    *p = value;
    
    ChipsetState->Colour[3] = RGB12(value);
    return;
}

// 188
void COLOR04(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF188];
    *p = value;
    
    ChipsetState->Colour[4] = RGB12(value);
    return;
}

// 18A
void COLOR05(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF18A];
    *p = value;
    
    ChipsetState->Colour[5] = RGB12(value);
    return;
}

// 18C
void COLOR06(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF18C];
    *p = value;
    
    ChipsetState->Colour[6] = RGB12(value);
    return;
}


// 18E
void COLOR07(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF18E];
    *p = value;
    
    ChipsetState->Colour[7] = RGB12(value);
    return;
}


// 190
void COLOR08(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF190];
    *p = value;
    
    ChipsetState->Colour[8] = RGB12(value);
    return;
}

// 192
void COLOR09(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF192];
    *p = value;
    
    ChipsetState->Colour[9] = RGB12(value);
    return;
}

// 194
void COLOR10(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF194];
    *p = value;
    
    ChipsetState->Colour[10] = RGB12(value);
    return;
}

// 196
void COLOR11(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF196];
    *p = value;
    
    ChipsetState->Colour[11] = RGB12(value);
    return;
}

// 198
void COLOR12(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF198];
    *p = value;
    
    ChipsetState->Colour[12] = RGB12(value);
    return;
}

// 19A
void COLOR13(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF19A];
    *p = value;
    
    ChipsetState->Colour[13] = RGB12(value);
    return;
}

// 19C
void COLOR14(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF19C];
    *p = value;
    
    ChipsetState->Colour[14] = RGB12(value);
    return;
}

// 19E
void COLOR15(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF19E];
    *p = value;
    
    ChipsetState->Colour[15] = RGB12(value);
    return;
}


// 1A0
void COLOR16(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1A0];
    *p = value;
    
    ChipsetState->Colour[16] = RGB12(value);
    return;
}

// 1A2
void COLOR17(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1A2];
    *p = value;
    
    ChipsetState->Colour[17] = RGB12(value);
    return;
}


// 1A4
void COLOR18(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1A4];
    *p = value;
    
    ChipsetState->Colour[18] = RGB12(value);
    return;
}

// 1A6
void COLOR19(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1A6];
    *p = value;
    
    ChipsetState->Colour[19] = RGB12(value);
    return;
}

// 1A8
void COLOR20(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1A8];
    *p = value;
    
    ChipsetState->Colour[20] = RGB12(value);
    return;
}


// 1AA
void COLOR21(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1AA];
    *p = value;
    
    ChipsetState->Colour[21] = RGB12(value);
    return;
}

// 1AC
void COLOR22(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1AC];
    *p = value;
    
    ChipsetState->Colour[22] = RGB12(value);
    return;
}

// 1AE
void COLOR23(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1AE];
    *p = value;
    
    ChipsetState->Colour[23] = RGB12(value);
    return;
}

// 1B0
void COLOR24(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1B0];
    *p = value;
    
    ChipsetState->Colour[24] = RGB12(value);
    return;
}

// 1B2
void COLOR25(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1B2];
    *p = value;
    
    ChipsetState->Colour[25] = RGB12(value);
    return;
}

// 1B4
void COLOR26(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1B4];
    *p = value;
    
    ChipsetState->Colour[26] = RGB12(value);
    return;
}

// 1B6
void COLOR27(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1B6];
    *p = value;
    
    ChipsetState->Colour[27] = RGB12(value);
    return;
}

// 1B8
void COLOR28(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1B8];
    *p = value;
    
    ChipsetState->Colour[28] = RGB12(value);
    return;
}

// 1BA
void COLOR29(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1BA];
    *p = value;
    
    ChipsetState->Colour[29] = RGB12(value);
    return;
}

// 1BC
void COLOR30(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1BC];
    *p = value;
    
    ChipsetState->Colour[30] = RGB12(value);
    return;
}

// 1BE
void COLOR31(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1BE];
    *p = value;
    
    ChipsetState->Colour[31] = RGB12(value);
    return;
}

// 1C8
void VTOTAL(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1C8];
    *p = value;
    return;
}

// 1CC
void VBSTRT(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1CC];
    *p = value;
    return;
}

// 1CE
void VBSTOP(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1CE];
    *p = value;
    return;
}

// 1E4
void DIWHIGH(uint16_t value){
    uint16_t* p = (uint16_t*) &RAM24bit[0xDFF1E4];
    *p = value;
    return;
}

// 1FE
void NO_OP(uint16_t value){
    return;
}

void NO_OP_32(uint32_t value){
    return;
}

void WriteToReadOnlyRegister16(uint16_t value){
    printf("Why?!?\n");
    return;
}

void NotImplemented16(uint16_t value){
    printf("Need to implmented this 16bit function\n");
}

void InitChipset(void* chipram, void* memory){
    
    ChipsetState = memory;//calloc(1, sizeof(Chipset_t));

    
    //Clear the Registers
    uint8_t* temp = chipram;
    for(int i=0xDFF000; i<0xDFF1FF; ++i){
       temp[i] = 0;
    }
    
    
    if(ChipsetState == NULL){
        printf("Chipset Mem Alloc Fail\n");
    }
    
    ChipsetState->chipram = chipram;
    
    for(int i = 32; i < 512; ++i){
        ChipsetState->WriteWord[i] = NotImplemented16;
    }
    
    for(int i = 0; i< 32; ++i){
        ChipsetState->WriteWord[i] = WriteToReadOnlyRegister16;
    }
    
    //Implemented Registers
    ChipsetState->WriteWord[0x00] = NO_OP;  // Copper tries to randomlly write here
    
    ChipsetState->WriteWord[0x20] = DSKPTH;
    ChipsetState->WriteWord[0x22] = DSKPTL;

    
    ChipsetState->WriteWord[0x24] = DSKLEN;
    
    ChipsetState->WriteWord[0x2A] = VPOSW;
    ChipsetState->WriteWord[0x2C] = VHPOSW;
    
    ChipsetState->WriteWord[0x30] = SERDAT;
    
    ChipsetState->WriteWord[0x32] = SERPER;
    ChipsetState->WriteWord[0x34] = POTGO;
    
    ChipsetState->WriteWord[0x36] = JOYTEST;
    
    ChipsetState->WriteWord[0x38] = STREQ;
    ChipsetState->WriteWord[0x3A] = STRVBL;
    ChipsetState->WriteWord[0x3C] = COPCON;
    ChipsetState->WriteWord[0x3E] = STRLONG;
    
    ChipsetState->WriteWord[0x40] = BLTCON0;
    ChipsetState->WriteWord[0x42] = BLTCON1;
    
    ChipsetState->WriteWord[0x44] = BLTAFWM;
    ChipsetState->WriteWord[0x46] = BLTALWM;

    
    ChipsetState->WriteWord[0x48] = BLTCPTH;
    ChipsetState->WriteWord[0x4A] = BLTCPTL;
    
    ChipsetState->WriteWord[0x4C] = BLTBPTH;
    ChipsetState->WriteWord[0x4E] = BLTBPTL;

    
    ChipsetState->WriteWord[0x50] = BLTAPTH;
    ChipsetState->WriteWord[0x52] = BLTAPTL;
    
    ChipsetState->WriteWord[0x54] = BLTDPTH;
    ChipsetState->WriteWord[0x56] = BLTDPTL;
    
    ChipsetState->WriteWord[0x58] = BLTSIZE;
    
    ChipsetState->WriteWord[0x5A] = BLTCON0L;
    ChipsetState->WriteWord[0x5C] = BLTSIZV;
    ChipsetState->WriteWord[0x5E] = BLTSIZH;
    
    ChipsetState->WriteWord[0x60] = BLTCMOD;
    ChipsetState->WriteWord[0x62] = BLTBMOD;
    ChipsetState->WriteWord[0x64] = BLTAMOD;
    ChipsetState->WriteWord[0x66] = BLTDMOD;
    
    ChipsetState->WriteWord[0x70] = BLTCDAT;
    ChipsetState->WriteWord[0x72] = BLTBDAT;
    ChipsetState->WriteWord[0x74] = BLTADAT;
    
    ChipsetState->WriteWord[0x7E] = DSKSYNC;
    
    ChipsetState->WriteWord[0x80] = COP1LCH;
    ChipsetState->WriteWord[0x82] = COP1LCL;
    
    ChipsetState->WriteWord[0x84] = COP2LCH;
    ChipsetState->WriteWord[0x86] = COP2LCL;
    
    ChipsetState->WriteWord[0x88] = COPJMP1;
    ChipsetState->WriteWord[0x8A] = COPJMP2;
    
    ChipsetState->WriteWord[0x8E] = DIWSTRT;
    ChipsetState->WriteWord[0x90] = DIWSTOP;
    ChipsetState->WriteWord[0x92] = DFFSTRT;
    ChipsetState->WriteWord[0x94] = DFFSTOP;
    
    ChipsetState->WriteWord[0x96] = DMACON;
    ChipsetState->WriteWord[0x9A] = INTENA;
    ChipsetState->WriteWord[0x9C] = INTREQ;
    
    ChipsetState->WriteWord[0x9E] = ADKCON;
    
    
    ChipsetState->WriteWord[0xA0] = AUD0LCH;
    ChipsetState->WriteWord[0xA2] = AUD0LCL;
    ChipsetState->WriteWord[0xA4] = AUD0LEN;
    ChipsetState->WriteWord[0xA6] = AUD0PER;
    ChipsetState->WriteWord[0xA8] = AUD0VOL;
    ChipsetState->WriteWord[0xAA] = AUD0DAT;
    
    // Channel 1
    ChipsetState->WriteWord[0xB0] = AUD1LCH;
    ChipsetState->WriteWord[0xB2] = AUD1LCL;
    ChipsetState->WriteWord[0xB4] = AUD1LEN;
    ChipsetState->WriteWord[0xB6] = AUD1PER;
    ChipsetState->WriteWord[0xB8] = AUD1VOL;
    ChipsetState->WriteWord[0xBA] = AUD1DAT;

    // Channel 2
    ChipsetState->WriteWord[0xC0] = AUD2LCH;
    ChipsetState->WriteWord[0xC2] = AUD2LCL;
    ChipsetState->WriteWord[0xC4] = AUD2LEN;
    ChipsetState->WriteWord[0xC6] = AUD2PER;
    ChipsetState->WriteWord[0xC8] = AUD2VOL;
    ChipsetState->WriteWord[0xCA] = AUD2DAT;

    // Channel 3
    ChipsetState->WriteWord[0xD0] = AUD3LCH;
    ChipsetState->WriteWord[0xD2] = AUD3LCL;
    ChipsetState->WriteWord[0xD4] = AUD3LEN;
    ChipsetState->WriteWord[0xD6] = AUD3PER;
    ChipsetState->WriteWord[0xD8] = AUD3VOL;
    ChipsetState->WriteWord[0xDA] = AUD3DAT;
    
    
    ChipsetState->WriteWord[0xE0] = BPL1PTH;
    ChipsetState->WriteWord[0xE2] = BPL1PTL;
    ChipsetState->WriteWord[0xE4] = BPL2PTH;
    ChipsetState->WriteWord[0xE6] = BPL2PTL;
    ChipsetState->WriteWord[0xE8] = BPL3PTH;
    ChipsetState->WriteWord[0xEA] = BPL3PTL;
    ChipsetState->WriteWord[0xEC] = BPL4PTH;
    ChipsetState->WriteWord[0xEE] = BPL4PTL;
    ChipsetState->WriteWord[0xF0] = BPL5PTH;
    ChipsetState->WriteWord[0xF2] = BPL5PTL;
    ChipsetState->WriteWord[0xF4] = BPL6PTH;
    ChipsetState->WriteWord[0xF6] = BPL6PTL;
    

    
    ChipsetState->WriteWord[0x100] = BPLCON0;
    ChipsetState->WriteWord[0x102] = BPLCON1;
    ChipsetState->WriteWord[0x104] = BPLCON2;
    ChipsetState->WriteWord[0x106] = BPLCON3;
    
    ChipsetState->WriteWord[0x108] = BPL1MOD;
    ChipsetState->WriteWord[0x10A] = BPL2MOD;
    
    ChipsetState->WriteWord[0x10C] = UNKNOWN10C;
    
    ChipsetState->WriteWord[0x110] = BPL1DAT;
    ChipsetState->WriteWord[0x112] = BPL2DAT;
    ChipsetState->WriteWord[0x114] = BPL3DAT;
    ChipsetState->WriteWord[0x116] = BPL4DAT;
    ChipsetState->WriteWord[0x118] = BPL5DAT;
    ChipsetState->WriteWord[0x11A] = BPL6DAT;
    
    ChipsetState->WriteWord[0x120] = SPR0PTH;
    ChipsetState->WriteWord[0x122] = SPR0PTL;
    ChipsetState->WriteWord[0x124] = SPR1PTH;
    ChipsetState->WriteWord[0x126] = SPR1PTL;
    ChipsetState->WriteWord[0x128] = SPR2PTH;
    ChipsetState->WriteWord[0x12A] = SPR2PTL;
    ChipsetState->WriteWord[0x12C] = SPR3PTH;
    ChipsetState->WriteWord[0x12E] = SPR3PTL;
    ChipsetState->WriteWord[0x130] = SPR4PTH;
    ChipsetState->WriteWord[0x132] = SPR4PTL;
    ChipsetState->WriteWord[0x134] = SPR5PTH;
    ChipsetState->WriteWord[0x136] = SPR5PTL;
    ChipsetState->WriteWord[0x138] = SPR6PTH;
    ChipsetState->WriteWord[0x13A] = SPR6PTL;
    ChipsetState->WriteWord[0x13C] = SPR7PTH;
    ChipsetState->WriteWord[0x13E] = SPR7PTL;
    
    ChipsetState->WriteWord[0x140] = SPR0POS;
    ChipsetState->WriteWord[0x142] = SPR0CTL;
    ChipsetState->WriteWord[0x148] = SPR1POS;
    ChipsetState->WriteWord[0x14A] = SPR1CTL;
    ChipsetState->WriteWord[0x150] = SPR2POS;
    ChipsetState->WriteWord[0x152] = SPR2CTL;
    ChipsetState->WriteWord[0x158] = SPR3POS;
    ChipsetState->WriteWord[0x15A] = SPR3CTL;
    ChipsetState->WriteWord[0x160] = SPR4POS;
    ChipsetState->WriteWord[0x162] = SPR4CTL;
    ChipsetState->WriteWord[0x164] = SPR4DATA;
    
    ChipsetState->WriteWord[0x168] = SPR5POS;
    ChipsetState->WriteWord[0x16A] = SPR5CTL;
    
    ChipsetState->WriteWord[0x16E] = SPR5DATB;
    
    ChipsetState->WriteWord[0x170] = SPR6POS;
    ChipsetState->WriteWord[0x172] = SPR6CTL;
    ChipsetState->WriteWord[0x178] = SPR7POS;
    ChipsetState->WriteWord[0x17A] = SPR7CTL;
    
    
    ChipsetState->WriteWord[0x180] = COLOR00;
    ChipsetState->WriteWord[0x182] = COLOR01;
    ChipsetState->WriteWord[0x184] = COLOR02;
    ChipsetState->WriteWord[0x186] = COLOR03;
    ChipsetState->WriteWord[0x188] = COLOR04;
    ChipsetState->WriteWord[0x18A] = COLOR05;
    ChipsetState->WriteWord[0x18C] = COLOR06;
    ChipsetState->WriteWord[0x18E] = COLOR07;
    ChipsetState->WriteWord[0x190] = COLOR08;
    ChipsetState->WriteWord[0x192] = COLOR09;
    ChipsetState->WriteWord[0x194] = COLOR10;
    ChipsetState->WriteWord[0x196] = COLOR11;
    ChipsetState->WriteWord[0x198] = COLOR12;
    ChipsetState->WriteWord[0x19A] = COLOR13;
    ChipsetState->WriteWord[0x19C] = COLOR14;
    ChipsetState->WriteWord[0x19E] = COLOR15;
    ChipsetState->WriteWord[0x1A0] = COLOR16;
    ChipsetState->WriteWord[0x1A2] = COLOR17;
    ChipsetState->WriteWord[0x1A4] = COLOR18;
    ChipsetState->WriteWord[0x1A6] = COLOR19;
    ChipsetState->WriteWord[0x1A8] = COLOR20;
    ChipsetState->WriteWord[0x1AA] = COLOR21;
    ChipsetState->WriteWord[0x1AC] = COLOR22;
    ChipsetState->WriteWord[0x1AE] = COLOR23;
    ChipsetState->WriteWord[0x1B0] = COLOR24;
    ChipsetState->WriteWord[0x1B2] = COLOR25;
    ChipsetState->WriteWord[0x1B4] = COLOR26;
    ChipsetState->WriteWord[0x1B6] = COLOR27;
    ChipsetState->WriteWord[0x1B8] = COLOR28;
    ChipsetState->WriteWord[0x1BA] = COLOR29;
    ChipsetState->WriteWord[0x1BC] = COLOR30;
    ChipsetState->WriteWord[0x1BE] = COLOR31;
    

    
    ChipsetState->WriteWord[0x1C8] = VTOTAL;
    ChipsetState->WriteWord[0x1CC] = VBSTRT;
    ChipsetState->WriteWord[0x1CE] = VBSTOP;
    ChipsetState->WriteWord[0x1DC] = NO_OP;
    ChipsetState->WriteWord[0x1FC] = NO_OP;

    ChipsetState->WriteWord[0x1E4] = DIWHIGH;
    ChipsetState->WriteWord[0x1FE] = NO_OP;
    ChipsetState->WriteWord[0x200] = NO_OP;
    
    
    //Set the display to first possible pixel
    DFFSTRT(0x18);
    ChipsetState->bitplaneFetchActive = 0;
    
    // DeniseID (0xDFF07C): OCS=junk, ECS Denise 8373=0x00FC
#if defined(CHIPSET_ECS)
    RAM24bit[0xDFF07C] = 0x00;
    RAM24bit[0xDFF07D] = 0xFC;
#else
    //Put some junk in the DeniseID Reg so the software don't think this is ECS/OCS
    RAM24bit[0xDFF07C] = 0xFF;
    RAM24bit[0xDFF07D] = 0xFF;
#endif
    
    //Init Some Registers
    ChipsetState->DMACONR = 0;
    RAM24bit[0xDFF002] = 0;
    RAM24bit[0xDFF003] = 0;
    
    RAM24bit[0xDFF004] = 0;
    RAM24bit[0xDFF005] = 0;
    
    ChipsetState->INTENAR = 0;
    INTENA(0x7FFF);
    
    BLTCON0(0x0);
    BLTCON1(0x0);
    BLTAFWM(0x0);
    BLTALWM(0x0);
    
    BLTAPTH(0x0);
    BLTAPTL(0x0);
    BLTBPTH(0x0);
    BLTBPTL(0x0);
    BLTCPTH(0x0);
    BLTCPTL(0x0);
    BLTDPTH(0x0);
    BLTDPTL(0x0);
    
    BLTAMOD(0x0);
    BLTBMOD(0x0);
    BLTCMOD(0x0);
    BLTDMOD(0x0);
    
    BLTCDAT(0x0);
    BLTBDAT(0x0);
    BLTADAT(0x0);
    
    BPL1PTH(0x0);
    BPL1PTL(0x0);
    BPL2PTH(0x0);
    BPL2PTL(0x0);
    BPL3PTH(0x0);
    BPL3PTL(0x0);
    BPL4PTH(0x0);
    BPL4PTL(0x0);
    BPL5PTH(0x0);
    BPL5PTL(0x0);
    

    //For OS 1.x
    DSKSYNC(0x4489);
    VTOTAL(262);
    
    //Set the copper to wait until it is reset
    
    COP1LCH(0x0);
    COP1LCL(0x0);
    COP2LCH(0x0);
    COP2LCL(0x0);
    ChipsetState->CopperPC         = 0x0;
    ChipsetState->CopperIR1        = 0xFFFF;
    ChipsetState->CopperIR2        = 0xFFFE;
    ChipsetState->CopperState      = 3;
    ChipsetState->copper_wake_cycle = 0;  // wake immediately on first eval
    
    RAM24bit[0xDFF100] = 0x0;
    RAM24bit[0xDFF101] = 0x0;
    
    //Lazy way to set the Right Mouse Button to up
    RAM24bit[0xDFF016]  = 0xFF;
    RAM24bit[0xDFF017]  = 0xFF;
    
    //Setup the Serial port to show that the buffer is empty
    RAM24bit[0xDFF018]  = 0x38;
    
    ChipsetState->DMACycles = 0;
    ChipsetState->DMAFreq = 3579545;    // NTSC
    //ChipsetState->DMAFreq = 3546895;    // PAL
    
    ChipsetState->DisplayPositionAdjust = (22 * 800) + 40; //Centre the display
    
    for(int i=0;i<8;++i){
        ChipsetState->sprite[i].VPOS = 79874; //Sprite will never active
    }
    
    
}


void WriteChipsetByte(unsigned int address, unsigned int value){
    
    //Writing to a single byte in a chipset register needs to be
    //swapped as the High and low bytes are swapped in Little Endian.
    
    uint16_t* registerValue = (uint16_t*)&RAM24bit[address];
    uint16_t newValue;
    
    //swap which byte is being written to
    if(address & 1){
        newValue = (*registerValue & 0xFF00) | value;
    }else{
        newValue = (*registerValue & 0xFF) | (value << 8);
    }
    
    address = address & 510;    //Mask out top bits and the odd bit.
    ChipsetState->WriteWord[address](newValue);
}

void WriteChipsetWord(unsigned int address, unsigned int value){
    
    address = address & 511;
    ChipsetState->WriteWord[address](value);
}

void WriteChipsetLong(unsigned int address, unsigned int value){
    
    
    address = address & 511;
    
    ChipsetState->WriteWord[address](value >> 16);
    ChipsetState->WriteWord[address+2](value & 65535);

}


// ---------------------------------------------------------------------------
// SLOT_VBL handler — fires via the scheduler at the DMA cycle the beam wraps.
// Decouples VBL interrupt dispatch from the per-cycle beam tracking loop.
// ---------------------------------------------------------------------------
static void sched_vbl_handler(void) {
    CIAATOD();                                              // CIA-A frame counter
    ChipsetState->WriteWord[0x9C](0x8020);                 // VBL interrupt (INTREQ bit 5)
    probe_emit(EVT_VBL, m68k_get_reg(NULL, M68K_REG_PC), ChipsetState->DMACONR);
    ChipsetState->WriteWord[0x88](0);                      // Copper: restart from COP1LC
    ChipsetState->VBL = 1;                                 // notify host: frame complete

    // Debug: AROS boot progress
    static uint32_t s_vbl = 0;
    static uint32_t s_prev_pc = 0;
    s_vbl++;
    uint32_t pc = m68k_get_reg(NULL, M68K_REG_PC);

    // While in the AROS ROM checksum loop (F80112-F8011A), log counter every 20 VBLs
    if(pc >= 0xF80110 && pc <= 0xF8011E) {
        if(s_vbl % 20 == 0) {
            omega_log_hex("AROS chk D0(ctr)", m68k_get_reg(NULL, M68K_REG_D0));
            omega_log_hex("AROS chk D1(sum)", m68k_get_reg(NULL, M68K_REG_D1));
        }
    }
    // Detect first VBL where CPU has left the checksum range
    else if(s_prev_pc >= 0xF80110 && s_prev_pc <= 0xF8011E) {
        omega_log_hex("AROS chk DONE pc", pc);
        omega_log_hex("AROS chk D1(sum)", m68k_get_reg(NULL, M68K_REG_D1));
    }
    // After checksum: one-time register dump at scan function entry (FE9800-FE980E)
    else if(pc >= 0xFE9800 && pc <= 0xFE980E) {
        static int s_scan_dumped = 0;
        if(!s_scan_dumped) {
            s_scan_dumped = 1;
            omega_log_hex("SCAN D0(start)", m68k_get_reg(NULL, M68K_REG_D0));
            omega_log_hex("SCAN D4(end?)",  m68k_get_reg(NULL, M68K_REG_D4));
            omega_log_hex("SCAN A3(frame)", m68k_get_reg(NULL, M68K_REG_A3));
            omega_log_hex("SCAN A4(callee)",m68k_get_reg(NULL, M68K_REG_A4));
            omega_log_hex("SCAN SR",        m68k_get_reg(NULL, M68K_REG_SR));
            omega_log_hex("SCAN SP",        m68k_get_reg(NULL, M68K_REG_SP));
        }
    }
    // After checksum: log PC + INTENA every 10 VBLs to track boot progress
    else if(s_vbl % 10 == 0) {
        omega_log_hex("AROS boot pc",    pc);
        omega_log_hex("AROS INTENAR",    ChipsetState->INTENAR);
        omega_log_hex("AROS INTREQR",    ChipsetState->INTREQR);
        omega_log_hex("AROS DMACONR",    ChipsetState->DMACONR);
    }

    s_prev_pc = pc;
}

uint32_t IncrementVHPOS(void){

    ChipsetState->VBL = 0;

    g_beam.h++;

    // End of line: h reached the line boundary (227 short / 228 long)
    if (g_beam.h >= beam_hcnt()) {

        CIABTOD(); // increment CIA-B horizontal line counter

        beam_eol(); // h=0, v++, toggle lol

        // End of frame: v exceeded the last valid line
        if (g_beam.v > beam_vmax()) {

            beam_eof(); // v=0, frame++, toggle lof

            ChipsetState->bitplaneFetchActive = 0;

            // Schedule VBL event at delta=0: fires in this same sched_advance()
            // pass, after SLOT_DMA (slot 1 < slot 4), giving correct ordering.
            sched_schedule(SLOT_VBL, 0, sched_vbl_handler);
        }
    }

    // Keep ChipsetState->VHPOS in sync for code that reads it directly
    ChipsetState->VHPOS = beam_vhpos();

    // Write CPU-visible beam registers (big-endian in chip RAM)
    uint16_t vposr = beam_vposr();
#if defined(CHIPSET_ECS)
    // ECS Super Agnus PAL 1MB (8372A) — chip ID in bits[14:8] of VPOSR
    vposr = (vposr & 0x8001) | 0x2200;
#endif
    uint16_t* p = (uint16_t*)&RAM24bit[0xDFF004];
    *p = ByteSwap16(vposr);

    p = (uint16_t*)&RAM24bit[0xDFF006];
    *p = ByteSwap16(beam_vhposr());

    return 0;
}




