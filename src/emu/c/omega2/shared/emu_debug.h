// emu_debug.h — emulator debug views: DMA, Copper, Memory
//
// All output goes through omega_host_log (UART serial).
// Call from the frame-22 dump hook in omega_glue.c or on demand.

#pragma once
#include <stdint.h>

// Dump DMA channel enable/disable state and all DMA pointer registers.
void emu_debug_dma(void);

// Decode and print up to `max_insn` Copper instructions starting from
// the current CopperPC.  Shows MOVE target/value, WAIT position/mask,
// and the current CopperState and copper_wake_cycle.
void emu_debug_copper(uint32_t max_insn);

// Hex + ASCII dump of `len` bytes starting at Amiga address `addr`.
// Prints 16 bytes per line via omega_host_log.
void emu_debug_mem(uint32_t addr, uint32_t len);
