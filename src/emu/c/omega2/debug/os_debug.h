// os_debug.h — AmigaOS structure reader for bare-metal debugging
//
// Inspired by vAmiga OSDebugger (Dirk W. Hoffmann, MPL 2.0)
// Ported to C99 / bare-metal: no stdlib, reads directly from RAM24bit.
//
// Entry point: os_debug_dump()
//   Reads ExecBase from address 4, then walks LibList + TaskReady/TaskWait.
//   Output goes to serial via omega_host_log_str().

#pragma once
#include <stdint.h>

// Call once per "interesting" frame (e.g. frame 22) to dump OS state.
void os_debug_dump(void);
