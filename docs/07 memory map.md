MC68k-64 Architecture Specification
Document 07 — Physical Memory Map

Status: Draft
Version: 0.3

1. Overview

The MC68k-64 physical address space is 64-bit, linear, and big-endian. All resources — RAM, ROM, chipset, expansion devices, and service interfaces — exist within a single unified address space.

There are no separate I/O spaces.

The map is organized around two principles:

Low region: boot and RAM — stable and predictable
High region: compatibility, expansion, and system control — fixed bases

The architecture supports both:

Legacy compatibility (cycle-sensitive MMIO regions)
Service-oriented execution (Service Gates and shared memory)
2. Top-Level Memory Map
Address                        Region                    Size
─────────────────────────────────────────────────────────────
0x0000_0000_0000_0000          Boot ROM / Vectors        64KB minimum
0x0000_0000_0001_0000          RAM (physical window)     64GB
0x0000_000F_FFFF_FFFF          End of RAM window         —
0x0000_0010_0000_0000          Reserved / future         —
  ...
0x0000_00FF_0000_0000          Chipset                   16MB
0x0000_00FF_0100_0000          Zorro CFG                 16MB
0x0000_00FF_0200_0000          Zorro board space         512MB
0x0000_00FF_2200_0000          System MMIO               256MB
0x0000_00FF_3200_0000          Reserved                  —
  ...
0x0000_00FF_FFFF_FFFF          End of implemented space  —

Addresses above this range are reserved and raise an Access Fault.

3. Boot Region
Base:     0x0000_0000_0000_0000
Size:     64KB minimum
Type:     ROM (architectural)
3.1 Boot Semantics

The boot region contains:

Initial exception vector table
Reset entry point
Minimal bootstrap environment
3.2 Orchestrated Boot Model

The boot region is architecturally defined as ROM, but may be provided by a system orchestrator.

The orchestrator is responsible for:

Initializing CPU state (PC, SSP)
Providing vector table contents
Preparing memory and service environment

After initialization:

VBR is relocated to RAM
Boot region becomes read-only and inert
4. RAM
Base:     0x0000_0000_0001_0000
Window:   64GB
Type:     RAM (cacheable, coherent)

The RAM window is an architectural reservation.

Implementations populate this window according to available physical memory.

Unimplemented addresses raise an Access Fault.

RAM is used for:

Program execution
OS and application memory
Service Gate shared buffers
DMA operations
5. Reserved Region
Base:     0x0000_0010_0000_0000
End:      0x0000_00FE_FFFF_FFFF
Type:     Reserved

Reserved for:

MMU expansion
Extended physical memory
Future architectural regions

Access raises an Access Fault.

6. Compatibility and Service Region
0x0000_00FF_0000_0000  ┌─────────────────────────────┐
                       │  Chipset (Legacy MMIO)       │
0x0000_00FF_0100_0000  ├─────────────────────────────┤
                       │  Zorro CFG Window            │
0x0000_00FF_0200_0000  ├─────────────────────────────┤
                       │  Zorro Board Space           │
0x0000_00FF_2200_0000  ├─────────────────────────────┤
                       │  System MMIO                 │
0x0000_00FF_3200_0000  ├─────────────────────────────┤
                       │  Reserved                    │
0x0000_00FF_FFFF_FFFF  └─────────────────────────────┘
6.1 Chipset Region (Legacy MMIO)
Base:     0x0000_00FF_0000_0000
Size:     16MB
Type:     MMIO (legacy-compatible)

Represents compatibility hardware.

Characteristics:

Fixed layout compatible with classical systems
Non-cacheable
Strong side effects

Accesses may be:

Direct hardware access
Trap-based emulation
Hybrid implementations

The orchestrator may alias this region into legacy 32-bit addresses (e.g., $DFF000).

6.2 Zorro CFG Window
Base:     0x0000_00FF_0100_0000
Size:     16MB
Type:     MMIO

Used for device and service discovery.

6.2.1 Unified Device Model

Both:

Physical expansion devices
Virtual Service Gates

are enumerated through this window.

Each device/service provides:

Identity
Capability information
Resource requirements

After configuration:

CFG window becomes inactive
Devices respond in assigned regions
6.3 Zorro Board Space
Base:     0x0000_00FF_0200_0000
Size:     512MB
Type:     Device Memory / MMIO hybrid

Used for:

Expansion devices
Service Gate shared memory
DMA buffers
Zero-copy communication

Characteristics:

Assigned dynamically
May contain large buffers
Accessible by CPU and DMA
7. System MMIO Region
Base:     0x0000_00FF_2200_0000
Size:     256MB
Type:     MMIO (system control)

Contains intrinsic system devices and fixed-base control interfaces.

7.1 Device Map
Address                      Device
────────────────────────────────────────
0x0000_00FF_2200_0000        Timer
0x0000_00FF_2201_0000        IRQ Controller
0x0000_00FF_2202_0000        DMA Controller
0x0000_00FF_2203_0000        Debug Console
  ...
0x0000_00FF_2210_0000        Coprocessor Block
  ...
0x0000_00FF_2300_0000        Expansion Region

Each device has a minimum size of 64KB.

7.2 Core System Devices
Timer

Provides system timing and periodic interrupts.

IRQ Controller

Routes interrupt sources to CPU interrupt levels.

DMA Controller

Handles memory transfers using physical addresses.

Debug Console

Provides early boot diagnostics.

7.3 Coprocessor Block
Base:     0x0000_00FF_2210_0000
Type:     MMIO

Reserved for platform-integrated accelerators.

Examples:

Audio synthesis engines
DSP
Crypto engines
Compute accelerators

These are fixed-base architectural components.

7.4 Service Gate Integration

Service Gates integrate with the memory map as follows:

Control interface → MMIO region
Data interface → Board Space

Service Gates are indistinguishable from hardware devices at the architectural level.

They may be implemented by:

Software services on other cores
Firmware
Hardware accelerators
7.5 SYSTEM_MMIO_EXPANSION
Base:     0x0000_00FF_2300_0000
End:      0x0000_00FF_31FF_FFFF
Type:     Reserved

Reserved for future system-level devices.

8. Memory Attributes
Region	Cacheable	Writable	Notes
Boot ROM	No	No	Architectural
RAM	Yes	Yes	Coherent
Reserved	—	—	Access Fault
Chipset	No	Yes	Side effects
Zorro CFG	No	Yes	Discovery
Board Space	Mixed	Yes	Shared memory
System MMIO	No	Yes	Control
9. Interaction Model

Behavior is determined by address:

RAM → data access
MMIO → control and signaling
Board Space → shared data exchange

Service interaction model:

MMIO writes → control / doorbell
Shared memory → data transfer
Interrupts → notification
10. Architectural Notes
Services and hardware share the same abstraction
The orchestrator may virtualize any region
Legacy and modern models coexist
11. Decisions Deferred
Topic	Status
Zorro autoconfig protocol details	Deferred
Chipset extended registers	Deferred
Coprocessor register standard	Deferred
Service Gate MMIO allocation policy	Deferred
Board Space allocation strategy	Deferred
Interrupt routing model	Deferred
Security / isolation model	Deferred
MMU enforcement and permissions	Deferred
Multi-core affinity for services	Deferred

End of document 07-memory-map.md