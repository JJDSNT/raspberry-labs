//
//  DMA.h
//  Omega2
//
//  Created by Matt Parsons on 29/03/2022.
//

#ifndef DMA_h
#define DMA_h

#include <stdio.h>

// Legacy per-cycle DMA execution (called internally by the scheduler handler)
void DMAExecute(void* ChipState, uint32_t* framebuffer);

// Arm SLOT_CIA, SLOT_DMA, and SLOT_AUDIO in the scheduler; call once after sched_init()
void sched_dma_init(void);

#endif /* DMA_h */
