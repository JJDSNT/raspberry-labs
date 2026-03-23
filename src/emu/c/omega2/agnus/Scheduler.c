// Scheduler.c — slot-based event scheduler for Omega2

#include <stddef.h>
#include "Scheduler.h"

Scheduler_t g_sched;

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

static void update_trigger(void) {
    uint64_t min = SCHED_NEVER;
    for (int i = 0; i < SLOT_COUNT; i++) {
        if (g_sched.slot[i].triggerCycle < min)
            min = g_sched.slot[i].triggerCycle;
    }
    g_sched.nextTrigger = min;
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

void sched_init(void) {
    g_sched.clock = 0;
    for (int i = 0; i < SLOT_COUNT; i++) {
        g_sched.slot[i].triggerCycle = SCHED_NEVER;
        g_sched.slot[i].handler      = NULL;
    }
    g_sched.nextTrigger = SCHED_NEVER;
}

void sched_schedule(SlotID_t id, uint64_t delta, SlotHandler_t handler) {
    g_sched.slot[id].triggerCycle = g_sched.clock + delta;
    g_sched.slot[id].handler      = handler;
    // Fast path: only recompute min if new trigger is earlier
    if (g_sched.slot[id].triggerCycle < g_sched.nextTrigger)
        g_sched.nextTrigger = g_sched.slot[id].triggerCycle;
}

void sched_cancel(SlotID_t id) {
    g_sched.slot[id].triggerCycle = SCHED_NEVER;
    g_sched.slot[id].handler      = NULL;
    update_trigger();
}

void sched_advance(void) {
    g_sched.clock++;

    // Fast path: no events due this cycle
    if (g_sched.clock < g_sched.nextTrigger)
        return;

    // Fire all due slots in slot-ID order (lower ID = higher priority)
    for (int i = 0; i < SLOT_COUNT; i++) {
        if (g_sched.slot[i].triggerCycle <= g_sched.clock) {
            g_sched.slot[i].triggerCycle = SCHED_NEVER;
            if (g_sched.slot[i].handler)
                g_sched.slot[i].handler();
        }
    }

    update_trigger();
}

void sched_advance_n(uint32_t n) {
    for (uint32_t i = 0; i < n; i++)
        sched_advance();
}
