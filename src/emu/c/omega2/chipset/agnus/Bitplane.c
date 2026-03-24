//
//  Bitplane.c
//  Omega2
//
//  Created by Matt Parsons on 28/04/2022.
//

#include "Bitplane.h"


void P2C1(uint8_t* buffer, uint16_t source){
    
    source = source << 8 | source >> 8;
    
    for(int i = 15; i >= 0; --i){
        buffer[i] = source & 1;
        source = source >> 1;
    }
    
    
}

void P2C2(uint8_t* buffer, uint16_t source){
    
    source = source << 8 | source >> 8;
    
    for(int i = 15; i >= 0; --i){
        buffer[i] = (buffer[i] & 0xFD) | (source & 1) << 1;
        source = source >> 1;
    }

}

void P2C3(uint8_t* buffer,uint16_t source){
    
    source = source << 8 | source >> 8;
    
    for(int i = 15; i >= 0; --i){
        buffer[i] = (buffer[i] & 0xFB) | (source & 1) << 2;
        source = source >> 1;
    }

}

void P2C4(uint8_t* buffer,uint16_t source){
        
    source = source << 8 | source >> 8;
    
    for(int i = 15; i >= 0; --i){
        buffer[i] = (buffer[i] & 0xF7) | (source & 1) << 3;
        source = source >> 1;
    }
}

void P2C5(uint8_t* buffer,uint16_t source){
            
    source = source << 8 | source >> 8;
    
    for(int i = 15; i >= 0; --i){
        buffer[i] = (buffer[i] & 0xEF) | (source & 1) << 4;
        source = source >> 1;
    }

}

void P2C6(uint8_t* buffer,uint16_t source){
            
    source = source << 8 | source >> 8;
    
    for(int i = 15; i >= 0; --i){
        buffer[i] = (buffer[i] & 0xDF) | (source & 1) << 5;
        source = source >> 1;
    }

}


void BitplaneExecuteOLD(Chipset_t* ChipsetState){
    
    static int fetchcount;
    
    if( ChipsetState->DMACONR & 0x100){
    
        uint16_t DEBUGVPOS = ChipsetState->VHPOS >> 8;
        uint16_t DEBUGHPOS = ChipsetState->VHPOS & 0xFF;
        
        uint16_t* DIWSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF08E];
        uint16_t* DFFSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF092];
        uint16_t* DFFSTOP = (uint16_t*) &ChipsetState->chipram[0xDFF094];
        uint16_t FetchStop = *DFFSTOP;
        
        
        
        //Check the vertical position of the beam
        if( (ChipsetState->VHPOS & 0x1FF00) == (*DIWSTRT & 0xFF00) ){
            ChipsetState->bitplaneFetchActive = 1;
            
        }
        
        
        
        
        if( (ChipsetState->VHPOS & 0x1FF00) == (ChipsetState->VSTOP) ){
            ChipsetState->bitplaneFetchActive = 0;
            for(int i = 0; i < 16; ++i){
                ChipsetState->PixelBuffer[i] = 0;
            }
        }
        
        
        
        
        if(ChipsetState->bitplaneFetchActive == 1){
        uint16_t* BPLCON0 = (uint16_t*) &ChipsetState->chipram[0xDFF100];
        uint32_t hires = *BPLCON0 >> 15;
        uint32_t planeCount = (*BPLCON0 >> 12) & 0x7;
        
        if(hires){
            //FetchStop += 8;     // Normal KS1.3
            FetchStop += 4;   //To get KS3.1 boot screen working
            
        }
        

        
        
        // Check the horizontal position of the beam
        ChipsetState->bitplaneFetchCountdown -= 1;
        
        if( (ChipsetState->VHPOS & 0xFF) == *DFFSTRT){
            ChipsetState->bitplaneFetchCountdown = 0; // Start Countdown
            
            int VPOS = ((ChipsetState->VHPOS & 0x1FF00) >> 8);
            ChipsetState->FrameBufferLine = &ChipsetState->frameBuffer[800*VPOS - ChipsetState->DisplayPositionAdjust];
            ChipsetState->FrameBufferLineIndex = (*DIWSTRT & 0xFF);
            
        }
                    
        //Fetch the bitplane data into the pixelbuffer
        if(ChipsetState->bitplaneFetchCountdown == 0){
        
            
            //Do we need to stop the fetch
            if( (ChipsetState->VHPOS & 0xFF) > 0xD7 || (ChipsetState->VHPOS & 0xFF) > FetchStop){
                ChipsetState->needsRedraw = 1;
                ChipsetState->bitplaneFetchCountdown = -1; // End Countdown
                fetchcount = 0;
                int16_t* BPLMOD1 = (int16_t*)&ChipsetState->chipram[0xDFF108];
                int16_t* BPLMOD2 = (int16_t*)&ChipsetState->chipram[0xDFF10A];
                
                //need to add the Bitplane modulus if there is one.
                if(*BPLMOD1 !=0){
                    
                    uint32_t* BP;
                    uint16_t* Data;
                
                    //Bitplane1
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];  // get BLP1PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD1;
                    
                    //Bitplane2
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];  // get BLP2PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD2;
                    
                    //Bitplane3
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];  // get BLP3PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD1;
                    
                    //Bitplane4
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];  // get BLP4PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD2;
                    
                    //Bitplane5
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];  // get BLP5PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD1;
                    
                    //Bitplane6
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];  // get BLP6PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD2;
                    
                }
                return;
            }
            

            
            
            fetchcount += 1;
            
                if(planeCount > 0){
                    
                    
                    uint32_t* BP;
                    uint16_t* Data;
                    
                    //Bitplane1
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];  // get BLP1PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += 2;                                                       // increment register
                    P2C1(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                
            
                    if(planeCount > 1){
                    //Bitplane2
                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];  // get BLP2PT register
                        Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                        *BP += 2;                                                       // increment register
                        P2C2(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                        
                        if(planeCount > 2){
                            //Bitplane3
                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];  // get BLP3PT register
                            Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                            *BP += 2;                                                       // increment register
                            P2C3(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                            
                            if(planeCount > 3){
                                //Bitplane4
                                BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];  // get BLP4PT register
                                Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                *BP += 2;                                                       // increment register
                                P2C4(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                
                                if(planeCount > 4){
                                    //Bitplane5
                                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];  // get BLP5PT register
                                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                    *BP += 2;                                                       // increment register
                                    P2C5(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                    
                                    if(planeCount > 5){
                                        //Bitplane6
                                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];  // get BLP6PT register
                                        Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                        *BP += 2;                                                       // increment register
                                        P2C6(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                        
                                    }
                                }
                            }
                        }
                    }
                    
                    
                    //Flush buffer to screen
    
                    if(hires){
                        ChipsetState->bitplaneFetchCountdown = 4;
                        for(int i = 0; i < 16; ++i){
                            ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+i] = ChipsetState->Colour[ChipsetState->PixelBuffer[i]];
                        }
                        ChipsetState->FrameBufferLineIndex += 16;
                        
                    }else{
                        
                        
                        
                        ChipsetState->bitplaneFetchCountdown = 8;
                        //Flush buffer to screen
                        
                        if(planeCount == 6){
                            
                            for(int i = 0; i < 16; i += 1){
                                
                                uint32_t colour = ChipsetState->Colour[ChipsetState->PixelBuffer[i] & 0x1F];
                            
                                if(ChipsetState->PixelBuffer[i] > 31){
                                    colour = colour >> 1;
                                    colour = colour & 0xFF7F7F7F;
                                }
                            
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)] = colour;
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)+1] = colour;
                            }
                            
                        }else{
                        
                        
                            for(int i = 0; i < 16; i += 1){
                        
                                uint32_t colour = ChipsetState->Colour[ChipsetState->PixelBuffer[i]];
                        
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)] = colour;
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)+1] = colour;
                            }
                        }
                        
                        
                        ChipsetState->FrameBufferLineIndex += 32;
                        
                    }
                    
                    

                    
                    
                    

                    
                }
            
            
            
            
            }
        

        
    }
        
        

    
    }


}


void BitplaneExecuteHires(Chipset_t* ChipsetState){
    
    static int fetchCount = 0;
    
    //uint16_t DEBUGVPOS = ChipsetState->VHPOS >> 8;
    //uint16_t DEBUGHPOS = ChipsetState->VHPOS & 0xFF;
    
    //Do we need to process any Display?
    if( (ChipsetState->VHPOS & 0x1FF00) == (ChipsetState->VSTOP) ){
        ChipsetState->bitplaneFetchActive = 0;
        return;
    }
    
    
    uint16_t* DIWSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF08E];
    uint16_t* DIWSTOP = (uint16_t*) &ChipsetState->chipram[0xDFF090];
    uint16_t* DFFSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF092];
    uint16_t* DFFSTOP = (uint16_t*) &ChipsetState->chipram[0xDFF094];
    
    
    //Check the vertical position of the beam
    if( (ChipsetState->VHPOS & 0x1FF00) == (*DIWSTRT & 0xFF00) ){
        ChipsetState->bitplaneFetchActive = 1;
        ChipsetState->displayCountdown = -1;
    }
    
    if(ChipsetState->bitplaneFetchActive){

        ChipsetState->bitplaneFetchCountdown -= 1;
        
        if( (ChipsetState->VHPOS & 0xFF) == (*DFFSTRT)){
            ChipsetState->bitplaneFetchCountdown = 0; // Start Countdown
            fetchCount = 0;
            int VPOS = ((ChipsetState->VHPOS & 0x1FF00) >> 8);
            ChipsetState->FrameBufferLine = &ChipsetState->frameBuffer[
                ChipsetState->frameBufferStride * VPOS + (ChipsetState->HSTART << 1)];
            ChipsetState->FrameBufferLineIndex = 0;
            ChipsetState->needsRedraw = 1;
            
        }
    

    
        
        //Fetch the bitplane data into the pixelbuffer
        if(ChipsetState->bitplaneFetchCountdown == 0){
        
            if(ChipsetState->planeCount > 0){
                fetchCount +=1;
                
                uint32_t* BP;
                uint16_t* Data;
                
                //Bitplane1
                BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];  // get BLP1PT register
                Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                *BP += 2;                                                       // increment register
                P2C1(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
            
        
                if(ChipsetState->planeCount > 1){
                //Bitplane2
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];  // get BLP2PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += 2;                                                       // increment register
                    P2C2(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                    
                    if(ChipsetState->planeCount > 2){
                        //Bitplane3
                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];  // get BLP3PT register
                        Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                        *BP += 2;                                                       // increment register
                        P2C3(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                        
                        if(ChipsetState->planeCount > 3){
                            //Bitplane4
                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];
                            Data = (uint16_t*)&ChipsetState->chipram[*BP];
                            *BP += 2;
                            P2C4(ChipsetState->PixelBuffer, *Data);

                            if(ChipsetState->planeCount > 4){
                                //Bitplane5
                                BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];
                                Data = (uint16_t*)&ChipsetState->chipram[*BP];
                                *BP += 2;
                                P2C5(ChipsetState->PixelBuffer, *Data);

                                if(ChipsetState->planeCount > 5){
                                    //Bitplane6
                                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];
                                    Data = (uint16_t*)&ChipsetState->chipram[*BP];
                                    *BP += 2;
                                    P2C6(ChipsetState->PixelBuffer, *Data);
                                }
                            }
                        }
                    }
                }
                
                
  
                ChipsetState->bitplaneFetchCountdown = 4;

            }
        

                
            
            //Do we need to stop the fetch
            if( ((ChipsetState->VHPOS & 0xFF) > 0xD8) || ((ChipsetState->VHPOS & 0xFF) > (*DFFSTOP) && !(fetchCount & 0x1)) ){
                ChipsetState->bitplaneFetchCountdown = -1;
                
                int16_t* BPLMOD1 = (int16_t*)&ChipsetState->chipram[0xDFF108];
                int16_t* BPLMOD2 = (int16_t*)&ChipsetState->chipram[0xDFF10A];
                
                if(*BPLMOD1 != 0 || *BPLMOD2 != 0){
                    uint32_t* BP;

                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];
                    *BP += *BPLMOD1;

                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];
                    *BP += *BPLMOD2;

                    if(ChipsetState->planeCount > 2){
                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];
                        *BP += *BPLMOD1;

                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];
                        *BP += *BPLMOD2;

                        if(ChipsetState->planeCount > 4){
                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];
                            *BP += *BPLMOD1;

                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];
                            *BP += *BPLMOD2;
                        }
                    }
                }

            }
            
        
        }
        
        
        
        // Display output — Denise (hires path, no HAM/EHB in hires mode).
        if(ChipsetState->planeCount > 0){
            ChipsetState->displayCountdown -= 1;
            int HPOS;

            uint16_t* BPLCON1 = (uint16_t*) &ChipsetState->chipram[0xDFF102];
            int shift = *BPLCON1 & 0xF;

            for(int i = 0; i < 2; ++i){
                HPOS = (((ChipsetState->VHPOS & 0xFF) << 1) | i);

                if( HPOS == ((*DIWSTRT & 0xFF) - 4 - shift)){
                    ChipsetState->displayCountdown = 0;
                }

                if( HPOS == ((*DIWSTOP & 0xFF) | 0x100) - 8 - shift){
                    ChipsetState->displayCountdown = -1;
                }

                if(ChipsetState->displayCountdown == 0){
                    ChipsetState->displayCountdown = 4;
                    for(int j = 0; j < 16; ++j){
                        ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex + j] =
                            ChipsetState->Colour[ChipsetState->PixelBuffer[j] & 0x1Fu];
                    }
                    ChipsetState->FrameBufferLineIndex += 16;
                }
            }
        }

    }


}





void BitplaneExecuteLores(Chipset_t* ChipsetState){
    
    static int fetchCount = 0;
    
   // uint16_t DEBUGVPOS = ChipsetState->VHPOS >> 8;
   // uint16_t DEBUGHPOS = ChipsetState->VHPOS & 0xFF;
    
    //Do we need to process any Display?
    if( (ChipsetState->VHPOS & 0x1FF00) == (ChipsetState->VSTOP) ){
        ChipsetState->bitplaneFetchActive = 0;
        return;
    }
    
    
    uint16_t* DIWSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF08E];
    uint16_t* DIWSTOP = (uint16_t*) &ChipsetState->chipram[0xDFF090];
    uint16_t* DFFSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF092];
    uint16_t* DFFSTOP = (uint16_t*) &ChipsetState->chipram[0xDFF094];
    
    
    //Check the vertical position of the beam
    if( (ChipsetState->VHPOS & 0x1FF00) == (*DIWSTRT & 0xFF00) ){
        ChipsetState->bitplaneFetchActive = 1;
        ChipsetState->displayCountdown = -1;
    }
    
    if(ChipsetState->bitplaneFetchActive){
        
        ChipsetState->bitplaneFetchCountdown -= 1;
        
        if( (ChipsetState->VHPOS & 0xFF) == (*DFFSTRT)){
            ChipsetState->bitplaneFetchCountdown = 0; // Start Countdown
            fetchCount = 0;
            int VPOS = ((ChipsetState->VHPOS & 0x1FF00) >> 8);
            ChipsetState->FrameBufferLine = &ChipsetState->frameBuffer[
                ChipsetState->frameBufferStride * VPOS + (ChipsetState->HSTART << 1)];
            ChipsetState->FrameBufferLineIndex = 0;
        }
    

        
    
        
        //Fetch the bitplane data into the pixelbuffer
        if(ChipsetState->bitplaneFetchCountdown == 0){
        
            if(ChipsetState->planeCount > 0){
                fetchCount +=1;
                
                uint32_t* BP;
                uint16_t* Data;
                
                //Bitplane1
                BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];  // get BLP1PT register
                Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                *BP += 2;                                                       // increment register
                P2C1(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
            
        
                if(ChipsetState->planeCount > 1){
                //Bitplane2
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];  // get BLP2PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += 2;                                                       // increment register
                    P2C2(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                    
                    if(ChipsetState->planeCount > 2){
                        //Bitplane3
                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];  // get BLP3PT register
                        Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                        *BP += 2;                                                       // increment register
                        P2C3(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky

                        if(ChipsetState->planeCount > 3){
                            //Bitplane4
                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];  // get BLP4PT register
                            Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                            *BP += 2;                                                       // increment register
                            P2C4(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                            
                            if(ChipsetState->planeCount > 4){
                                //Bitplane5
                                BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];  // get BLP5PT register
                                Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                *BP += 2;                                                       // increment register
                                P2C5(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                
                                if(ChipsetState->planeCount > 5){
                                    //Bitplane6
                                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];  // get BLP6PT register
                                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                    *BP += 2;                                                       // increment register
                                    P2C6(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                    
                                }
                            }
                        }
                    }
                }
                
                ChipsetState->bitplaneFetchCountdown = 8;
   
            }
        

                
            
            //Do we need to stop the fetch
            if( ((ChipsetState->VHPOS & 0xFF) > 0xD8) || ((ChipsetState->VHPOS & 0xFF) == (*DFFSTOP)) ){
                ChipsetState->bitplaneFetchCountdown = -1;
                
                int16_t* BPLMOD1 = (int16_t*)&ChipsetState->chipram[0xDFF108];
                int16_t* BPLMOD2 = (int16_t*)&ChipsetState->chipram[0xDFF10A];
                
                // Apply bitplane modulos (odd planes use BPLMOD1, even use BPLMOD2).
                // Check each modulo independently — mod2 may be non-zero even when mod1=0.
                if(*BPLMOD1 != 0 || *BPLMOD2 != 0){

                    uint32_t* BP;

                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];
                    *BP += *BPLMOD1;

                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];
                    *BP += *BPLMOD2;

                    if(ChipsetState->planeCount > 2){
                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];
                        *BP += *BPLMOD1;

                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];
                        *BP += *BPLMOD2;

                        if(ChipsetState->planeCount > 4){
                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];
                            *BP += *BPLMOD1;

                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];
                            *BP += *BPLMOD2;
                        }
                    }
                }

            }

            ChipsetState->needsRedraw = 1;
        
        }
        
        
        
        // Display output — Denise reads PixelBuffer and outputs to framebuffer.
        // Runs independently of the Agnus DMA fetch above (different window: DIW vs DDF).
        if(ChipsetState->planeCount > 0){
            ChipsetState->displayCountdown -= 1;
            int HPOS;

            uint16_t* BPLCON0_ptr = (uint16_t*) &ChipsetState->chipram[0xDFF100];
            uint16_t  bplcon0     = *BPLCON0_ptr;
            uint16_t* BPLCON1     = (uint16_t*) &ChipsetState->chipram[0xDFF102];
            int shift = *BPLCON1 & 0xF;

            // Special color modes (lores only; HAM is undefined in hires)
            int ham = ((bplcon0 >> 11) & 1) && (ChipsetState->planeCount == 6);
            int ehb = !ham && (ChipsetState->planeCount == 6);

            for(int i = 0; i < 2; ++i){
                HPOS = (((ChipsetState->VHPOS & 0xFF) << 1) | i);

                if( HPOS == ((*DIWSTRT & 0xFF) - 8 - shift)){
                    ChipsetState->displayCountdown = 0;
                    ChipsetState->ham_color = ChipsetState->Colour[0]; // reset HAM carry at display start
                }

                if( HPOS == ((*DIWSTOP & 0xFF) | 0x100) - 16 - shift){
                    ChipsetState->displayCountdown = -1;
                }

                if(ChipsetState->displayCountdown == 0){
                    ChipsetState->displayCountdown = 8;

                    for(int j = 0; j < 16; j++){
                        uint8_t  idx = ChipsetState->PixelBuffer[j];
                        uint32_t colour;

                        if(ham){
                            // HAM6: bits[5:4] = control, bits[3:0] = data
                            uint8_t ctrl = (idx >> 4) & 0x3u;
                            uint8_t data = idx & 0xFu;
                            switch(ctrl){
                                case 0: colour = ChipsetState->Colour[data]; break;
                                case 1: // hold, modify blue
                                    colour = (ChipsetState->ham_color & 0xFFFFFF00u)
                                           | ((uint32_t)data << 4) | (uint32_t)data;
                                    break;
                                case 2: // hold, modify red
                                    colour = (ChipsetState->ham_color & 0xFF00FFFFu)
                                           | ((uint32_t)data << 20) | ((uint32_t)data << 16);
                                    break;
                                default: // case 3: hold, modify green
                                    colour = (ChipsetState->ham_color & 0xFFFF00FFu)
                                           | ((uint32_t)data << 12) | ((uint32_t)data << 8);
                                    break;
                            }
                            ChipsetState->ham_color = colour;
                        } else if(ehb && (idx >= 32)){
                            // EHB: upper 32 colors are half-bright versions of lower 32
                            colour = (ChipsetState->Colour[idx & 0x1Fu] >> 1) & 0xFF7F7F7Fu;
                        } else {
                            colour = ChipsetState->Colour[idx & 0x1Fu];
                        }

                        ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex + (j << 1)]     = colour;
                        ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex + (j << 1) + 1] = colour;
                    }

                    ChipsetState->FrameBufferLineIndex += 32;
                }
            }
        }
    }


}




int fetchlock = 0;

void BitplaneExecute(Chipset_t* ChipsetState){
    /*
    static int fetchcount;
    
    if( ChipsetState->DMACONR & 0x100){
    
        uint16_t DEBUGVPOS = ChipsetState->VHPOS >> 8;
        uint16_t DEBUGHPOS = ChipsetState->VHPOS & 0xFF;
        
        uint16_t* DIWSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF08E];
        uint16_t* DIWSTOP = (uint16_t*) &ChipsetState->chipram[0xDFF090];
        uint16_t* DFFSTRT = (uint16_t*) &ChipsetState->chipram[0xDFF092];
        uint16_t* DFFSTOP = (uint16_t*) &ChipsetState->chipram[0xDFF094];
        int fetchStop = *DFFSTOP+1;
        
        //Check the vertical position of the beam
        if( (ChipsetState->VHPOS & 0x1FF00) == (*DIWSTRT & 0xFF00) ){
            ChipsetState->bitplaneFetchActive = 1;
            
        }
        
    
        if( (ChipsetState->VHPOS & 0x1FF00) == (ChipsetState->VSTOP) ){
            ChipsetState->bitplaneFetchActive = 0;
            for(int i = 0; i < 16; ++i){
                ChipsetState->PixelBuffer[i] = 0;
            }
        }
        
        
        
        
        if(ChipsetState->bitplaneFetchActive == 1){
        uint16_t* BPLCON0 = (uint16_t*) &ChipsetState->chipram[0xDFF100];
        uint32_t hires = *BPLCON0 >> 15;
        uint32_t planeCount = (*BPLCON0 >> 12) & 0x7;
    

        
        
        // Check the horizontal position of the beam
        ChipsetState->bitplaneFetchCountdown -= 1;
        
        if( (ChipsetState->VHPOS & 0xFF) == *DFFSTRT){
            ChipsetState->bitplaneFetchCountdown = 0; // Start Countdown
            
            fetchlock = 0;
            int VPOS = ((ChipsetState->VHPOS & 0x1FF00) >> 8);
            ChipsetState->FrameBufferLine = &ChipsetState->frameBuffer[800*VPOS - ChipsetState->DisplayPositionAdjust];
            ChipsetState->FrameBufferLineIndex = (*DIWSTRT & 0xFF);
            
        }
                    
        //Fetch the bitplane data into the pixelbuffer
        if(ChipsetState->bitplaneFetchCountdown == 0){
            
            fetchlock = 1 - fetchlock;
            
            //Do we need to stop the fetch
            if( ((ChipsetState->VHPOS & 0xFF) > 0xD8) || (((ChipsetState->VHPOS & 0xFF) > fetchStop) && fetchlock) ){
                ChipsetState->needsRedraw = 1;
                ChipsetState->bitplaneFetchCountdown = -1; // End Countdown
                fetchcount = 0;
                int16_t* BPLMOD1 = (int16_t*)&ChipsetState->chipram[0xDFF108];
                int16_t* BPLMOD2 = (int16_t*)&ChipsetState->chipram[0xDFF10A];
                
                *BPLMOD1 = 0;
                *BPLMOD2 = 0;
                
                //need to add the Bitplane modulus if there is one.
                if(*BPLMOD1 !=0){
                    
                    uint32_t* BP;
                    uint16_t* Data;
                
                    //Bitplane1
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];  // get BLP1PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD1;
                    
                    //Bitplane2
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];  // get BLP2PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD2;
                    
                    //Bitplane3
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];  // get BLP3PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD1;
                    
                    //Bitplane4
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];  // get BLP4PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD2;
                    
                    //Bitplane5
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];  // get BLP5PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD1;
                    
                    //Bitplane6
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];  // get BLP6PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += *BPLMOD2;
                    
                }
                return;
            }
            

            
            
            fetchcount += 1;
            
                if(planeCount > 0){
                    
                    
                    uint32_t* BP;
                    uint16_t* Data;
                    
                    //Bitplane1
                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E0];  // get BLP1PT register
                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                    *BP += 2;                                                       // increment register
                    P2C1(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                
            
                    if(planeCount > 1){
                    //Bitplane2
                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E4];  // get BLP2PT register
                        Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                        *BP += 2;                                                       // increment register
                        P2C2(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                        
                        if(planeCount > 2){
                            //Bitplane3
                            BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0E8];  // get BLP3PT register
                            Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                            *BP += 2;                                                       // increment register
                            P2C3(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                            
                            if(planeCount > 3){
                                //Bitplane4
                                BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0EC];  // get BLP4PT register
                                Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                *BP += 2;                                                       // increment register
                                P2C4(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                
                                if(planeCount > 4){
                                    //Bitplane5
                                    BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F0];  // get BLP5PT register
                                    Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                    *BP += 2;                                                       // increment register
                                    P2C5(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                    
                                    if(planeCount > 5){
                                        //Bitplane6
                                        BP  = (uint32_t*) &ChipsetState->chipram[0xDFF0F4];  // get BLP6PT register
                                        Data = (uint16_t*)&ChipsetState->chipram[*BP];     // Get the data pointed to by register
                                        *BP += 2;                                                       // increment register
                                        P2C6(ChipsetState->PixelBuffer, *Data);    //Planar to Chunky
                                        
                                    }
                                }
                            }
                        }
                    }
                    
                    
                    //Flush buffer to screen
    
                    if(hires){
                        ChipsetState->bitplaneFetchCountdown = 4;
                        for(int i = 0; i < 16; ++i){
                            ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+i] = ChipsetState->Colour[ChipsetState->PixelBuffer[i]];
                        }
                        ChipsetState->FrameBufferLineIndex += 16;
                        
                    }else{
                        
                        
                        
                        ChipsetState->bitplaneFetchCountdown = 8;
                        fetchlock = 0;
                        //Flush buffer to screen
                        
                        if(planeCount == 6){
                            
                            for(int i = 0; i < 16; i += 1){
                                
                                uint32_t colour = ChipsetState->Colour[ChipsetState->PixelBuffer[i] & 0x1F];
                            
                                if(ChipsetState->PixelBuffer[i] > 31){
                                    colour = colour >> 1;
                                    colour = colour & 0xFF7F7F7F;
                                }
                            
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)] = colour;
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)+1] = colour;
                            }
                            
                        }else{
                        
                        
                            for(int i = 0; i < 16; i += 1){
                        
                                uint32_t colour = ChipsetState->Colour[ChipsetState->PixelBuffer[i]];
                        
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)] = colour;
                                ChipsetState->FrameBufferLine[ChipsetState->FrameBufferLineIndex+(i*2)+1] = colour;
                            }
                        }
                        
                        
                        ChipsetState->FrameBufferLineIndex += 32;
                        
                    }
                    
                    

                    
                    
                    

                    
                }
            
            
            
            
            }
        

        
    }
        
        

    
    }

*/
}

/*
if(*BPLCON0 & 0x800){
//HAM6
   
    for(int i = 0; i < 16; i += 1){
    
        uint32_t colour = ChipsetState->Colour[buffer[i]];
        
        int command = buffer[i] >> 5;
                
        switch(command){
            case 0:break;
            case 1:
                //Modify Blue
                colour = lastPixel & 0xFFFFFF00;
                uint8_t blue = (buffer[i] & 0xF);
                blue = (blue << 4) | blue;
                colour = colour | blue;
                break;
                
            case 2:
                //modify Red
                colour = lastPixel & 0xFF00FFFF;
                uint8_t red = (buffer[i] & 0xF);
                red = (red << 4) | red;
                colour = colour | (red << 16);
                break;
                
            case 3:
                //Modify Green
                colour = lastPixel & 0xFFFF00FF;;
                uint8_t green = (buffer[i] & 0xF);
                green = (green << 4) | green;
                colour = colour | (green << 8);
                break;
        }
                
        ChipsetState->frameBuffer[ChipsetState->framebufferIndex+(i*2)   - centre] = colour;
        ChipsetState->frameBuffer[ChipsetState->framebufferIndex+(i*2)+1 - centre] = colour;
        lastPixel = colour;
    }
    ChipsetState->needsRedraw = 1;
    

}else{
//EHB
    
    for(int i = 0; i < 16; i += 1){

        uint32_t colour = ChipsetState->Colour[buffer[i] & 0x1F];
    
            if(buffer[i] > 31){
                colour = colour >> 1;
                colour = colour & 0xFF7F7F7F;
            }
    
            ChipsetState->frameBuffer[ChipsetState->framebufferIndex+(i*2)   - centre] = colour;
            ChipsetState->frameBuffer[ChipsetState->framebufferIndex+(i*2)+1 - centre] = colour;
        }
        ChipsetState->needsRedraw = 1;
}
*/
