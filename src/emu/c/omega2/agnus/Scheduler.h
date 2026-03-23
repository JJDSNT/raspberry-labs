// Scheduler.h — slot-based event scheduler for Omega2
//
// Inspired by vAmiga's Agnus scheduler: fixed array of slots (one per
// chipset subsystem), O(1) dispatch per DMA cycle.
//
// Slots fire in slot-ID order when multiple slots share the same cycle.
// All handlers are void(void) and use the global ChipsetState.

#ifndef Scheduler_h
#define Scheduler_h

#include <stdint.h>

// ---------------------------------------------------------------------------
// Slot IDs — one per chipset subsystem, fired in order when same cycle
// ---------------------------------------------------------------------------
typedef enum {
    SLOT_CIA   = 0,   // CIA-A + CIA-B timers + Floppy  (E-clock, every 5 DMA)
    SLOT_DMA   = 1,   // Copper, Blitter, Bitplane, Sprite DMA  (every DMA cycle)
    SLOT_IRQ   = 2,   // Deferred interrupt check
    SLOT_AUDIO = 3,   // Paula audio DMA — 4 channels, fires every AUD_SAMPLE_PERIOD
    SLOT_VBL   = 4,   // VBL event — scheduled by IncrementVHPOS on frame wrap
    SLOT_COUNT
} SlotID_t;

#define SCHED_NEVER  UINT64_MAX

typedef void (*SlotHandler_t)(void);

typedef struct {
    uint64_t      triggerCycle;   // absolute DMA cycle when slot fires
    SlotHandler_t handler;
} Slot_t;

typedef struct {
    uint64_t clock;               // master DMA cycle counter (advances 1 per DMA slot)
    Slot_t   slot[SLOT_COUNT];
    uint64_t nextTrigger;         // cached min of all triggerCycles (fast early exit)
} Scheduler_t;

// Global scheduler — defined in Scheduler.c, extern here
extern Scheduler_t g_sched;

// CPU cycles per DMA cycle (Amiga: 7.09 MHz CPU / 3.55 MHz DMA = 2)
#define CPU_CYCLES_PER_DMA  2

// E-clock period in DMA cycles (E = CPU/10 = DMA/5)
#define ECLOCK_PERIOD       5

// Batch size: DMA cycles processed per main-loop iteration
// Trade-off: larger = less call overhead, coarser granularity
#define SCHED_BATCH         8

void sched_init(void);
void sched_schedule(SlotID_t id, uint64_t delta, SlotHandler_t handler);
void sched_cancel(SlotID_t id);
void sched_advance(void);             // advance exactly one DMA cycle
void sched_advance_n(uint32_t n);     // advance n DMA cycles

static inline uint64_t sched_clock(void) { return g_sched.clock; }

#endif /* Scheduler_h */
