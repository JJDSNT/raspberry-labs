# MC68k-64 Architecture Specification
## Document 08 — Future Extensions and Architectural Intent

**Status:** Draft  
**Version:** 0.1  

---

## 1. Purpose

This document is not a list of deferred decisions. It is a statement of architectural intent — what is coming, how it fits into what already exists, and which decisions made today were made deliberately with future extensions in mind.

Each section answers three questions:

1. What is the chosen direction?
2. Which current decisions were already made because of this?
3. What remains to be defined?

---

## 2. Memory Management Unit (MMU)

### 2.1 Chosen Direction

The MC68k-64 MMU is a **page-based virtual memory system** with the following characteristics:

- Page size: **4KB base** (standard, universally supported by OS allocators and device drivers)
- Translation levels: **3 levels** (Sv39-like model)
- Virtual address space: **39-bit effective** per process (512GB)
- Physical address space: defined by implementation, up to the 64GB RAM window plus MMIO

The 3-level model is a deliberate version choice, not a conceptual limitation. Future extensions may add a fourth level to reach a 48-bit virtual space (256TB) without altering the semantics of the existing three levels.

```
Virtual address (39-bit canonical):
  bits 38:30  →  L1 index (9 bits, 512 entries)
  bits 29:21  →  L2 index (9 bits, 512 entries)
  bits 20:12  →  L3 index (9 bits, 512 entries)
  bits 11:0   →  page offset (12 bits, 4KB)
```

Formal statement: *The MMU v1 defines a 3-level translation model with a 39-bit effective virtual address space. Future revisions may extend to 4 levels without altering existing semantics.*

### 2.2 Decisions Already Made for This

The following v0 decisions were taken deliberately to accommodate the MMU:

| Decision | Where defined | Why it matters for MMU |
|----------|--------------|----------------------|
| PTBR reserved | 01-registers | Root of page table — ready to use |
| ASID reserved | 01-registers | Avoids full TLB flush on context switch |
| HARTID reserved | 01-registers | Per-core TLB management in SMP |
| DMA uses physical addresses | 02-memory-model | DMA bypasses MMU cleanly — no IOMMU needed in v1 |
| Page fault vector (7) reserved | 05-exception-model | Structured fault delivery already in place |
| EADDR in exception frame | 05-exception-model | Carries the faulting virtual address |
| Supervisor-only MMIO advisory in v0 | 06-privilege-model | MMU will enforce this properly in v1 |

### 2.3 Page Table Entry Format (Intent)

Each page table entry (PTE) is 64-bit. Minimum protection bits:

| Bit | Name | Description |
|-----|------|-------------|
| 0 | V | Valid |
| 1 | R | Read permission |
| 2 | W | Write permission |
| 3 | X | Execute permission |
| 4 | U | User accessible (0 = supervisor only) |
| 5 | G | Global (not flushed on ASID change) — reserved for v1 |
| 6 | A | Accessed (set by hardware on access) — reserved for v1 |
| 7 | D | Dirty (set by hardware on write) — reserved for v1 |

Upper bits carry the physical page number. Exact format to be defined in the MMU specification.

### 2.4 Page Fault Model

Page faults are already structurally present in the exception model. When the MMU raises a page fault:

- Vector 7 is taken
- EADDR receives the faulting virtual address
- CAUSE distinguishes fault type:

| CAUSE sub-code | Condition |
|----------------|-----------|
| Page not present | PTE.V = 0 |
| Privilege violation | User access to supervisor-only page |
| Write fault | Write to read-only page |
| Execute fault | Fetch from non-executable page |

### 2.5 MMIO Protection

In v0, supervisor-only MMIO protection is advisory. With the MMU active, protection is enforced through page table attributes:

- Supervisor-only MMIO regions are mapped with U=0
- User processes cannot access MMIO regions without an explicit OS mapping decision
- The OS retains full control over what it exposes to user space

DMA always operates on physical addresses. The OS is responsible for ensuring DMA descriptors contain valid physical addresses when virtual addressing is active.

### 2.6 What Remains to Be Defined

- Complete PTE format with physical page number field layout
- TLB invalidation instructions (`TLBI` or equivalent)
- SFENCE / memory barrier instructions for TLB coherence in SMP
- Behavior of ASID=0 (global mappings)
- Handling of misaligned virtual addresses above bit 38

---

## 3. Atomic Memory Operations

### 3.1 Chosen Direction

The MC68k-64 atomic primitive is **Load-Reserved / Store-Conditional (LR/SC)**, not Compare-And-Swap.

This is a deliberate foundational choice:

- LR/SC is simpler to implement correctly in both emulator and FPGA
- LR/SC avoids the ABA problem inherent in CAS semantics
- LR/SC composes cleanly — any synchronization primitive (spinlock, mutex, semaphore) can be built from it
- LR/SC aligns with the RISC-V and ARM LL/SC model, both proven in production SMP systems

Compare-And-Swap (CAS) may be provided later as a derived convenience instruction, but it is not the architectural foundation.

### 3.2 Why v0 Has No Atomics

The absence of atomic instructions in v0 is a deliberate phase decision, not an oversight:

- v0 is architecturally single-core from a coherence perspective
- The AROS kernel functions correctly without atomics in a uniprocessor environment
- Introducing atomics requires defining the memory model simultaneously — premature without SMP
- Bring-up, porting, and validation are simpler without atomics in the instruction set

This is not a deficit. It is correct sequencing.

### 3.3 Decisions Already Made for This

| Decision | Where defined | Why it matters for atomics |
|----------|--------------|--------------------------|
| Memory model is simple sequential in v0 | 02-memory-model | Clean baseline before relaxation |
| IPI vector (21) reserved | 05-exception-model | Atomics + IPI together enable SMP |
| Encoding classes 1000–1111 reserved | 03-instruction-encoding | Space for atomic instruction class |

### 3.4 LR/SC Semantics (Intent)

```asm
LR.L  D0, (A0)    ; load-reserved: read (A0), mark reservation
SC.L  D1, (A0)    ; store-conditional: write D1 to (A0) if reservation holds
                  ; D1 = 0 on success, D1 = 1 on failure
```

A reservation is broken by:
- Any store to the reserved address from any core
- Any exception or context switch
- Implementation-defined conditions (e.g. reservation granule conflict)

Software must always check the SC result and retry on failure.

### 3.5 Memory Model Intent

The future memory model will be:

- **Single-core (v0):** sequentially consistent — loads and stores appear in program order
- **Multi-core (SMP):** weakly ordered with explicit synchronization via atomics and fence instructions
- **Fence instructions** will be introduced alongside atomics to enforce ordering between memory operations

The model will be simple and explicit — not excessively relaxed. Software that uses LR/SC pairs and fences correctly will be portable across all implementations.

### 3.6 What Remains to Be Defined

- LR/SC instruction encoding (new CLASS or extension of existing CLASS)
- Fence instruction variants (store fence, load fence, full fence)
- Reservation granule size
- Interaction with DMA — whether DMA breaks reservations
- CAS as optional derived instruction

---

## 4. Symmetric Multi-Processing (SMP)

### 4.1 Chosen Direction

The MC68k-64 SMP model is **shared-memory multi-core** with architecturally-guaranteed coherence and implementation-defined coherence protocol.

Key properties:

- All cores share a single physical address space
- Memory coherence is an architectural guarantee — software does not need to know whether the implementation uses MESI, MOESI, or write-through
- Each core is identified by HARTID
- Inter-core communication uses IPI (Inter-Processor Interrupt)
- SMP requires atomics — SMP and atomics arrive together

### 4.2 Coherence Model

The ISA guarantees **observable coherence**: all cores observe a consistent view of shared memory when synchronization is performed correctly via atomics and fences.

The implementation is free to choose any coherence protocol (MESI, MOESI, write-through, etc.). This detail is invisible to the ISA.

Software correctness depends on:
- Using LR/SC pairs for shared state modification
- Using fence instructions to enforce ordering
- Not relying on implementation-specific cache behavior

### 4.3 HARTID

Each core has a unique, immutable HARTID readable via CSR access. HARTID is assigned by the implementation and documented in the platform BSP.

Uses:
- Per-core data structures (stack, scheduler state, TLB management)
- IPI targeting
- Debug identification

### 4.4 IPI Model

IPI delivery uses vector 21 (already reserved in the exception model). The mechanism for triggering an IPI on a remote core will be a write to an MMIO register in the IRQ controller block (`0x0000_00FF_2201_0000`).

Intended model:
```
Core A writes target HARTID + IPI type to IRQ controller MMIO
IRQ controller delivers interrupt to Core B
Core B takes vector 21, reads CAUSE for IPI type
Core B handles and returns via RTE
```

### 4.5 What the AROS Port Will Need

SMP support in AROS requires:

- Atomic spinlocks and mutexes (depends on LR/SC)
- SMP-aware scheduler with per-core run queues
- HARTID-aware per-core initialization
- IPI-based TLB shootdown (when MMU is active)
- IPI-based scheduler cross-calls

None of this requires changes to the ISA beyond atomics and IPI. The architectural foundations (HARTID, IPI vector, shared memory model) are already in place.

### 4.6 What Remains to Be Defined

- Maximum supported HARTID / core count
- IPI type encoding in CAUSE
- Boot protocol — how secondary cores are started
- Per-core reset behavior
- HARTID assignment convention

---

## 5. Heterogeneous Execution Model

### 5.1 Chosen Direction

The MC68k-64 platform supports a **heterogeneous execution model** in which additional processing components — software emulators, hardware coprocessors, or dedicated cores — are integrated as MMIO devices with access to shared memory and interrupt signaling.

This is not a special case. It is the natural consequence of the memory model: everything is memory, and devices are addresses.

Formal statement: *Any component that reads and writes memory and responds to MMIO addresses is a first-class citizen of the MC68k-64 platform, regardless of its physical implementation.*

### 5.2 Integration Contract

All heterogeneous components communicate with the MC68k-64 CPU through exactly three mechanisms:

| Mechanism | Purpose |
|-----------|---------|
| Shared memory | Bulk data transfer — audio buffers, framebuffers, command rings, shared data structures |
| MMIO | Control, status, configuration, command dispatch |
| IRQ | Asynchronous event notification — completion, error, data ready |

No other inter-domain communication mechanism is architecturally defined. This constraint is intentional — it keeps the model uniform and implementable in software, FPGA, or dedicated silicon.

### 5.3 Component Examples

**Chipset emulator (dedicated core):**
```
CPU writes to chipset MMIO region (0x0000_00FF_0000_0000)
Chipset core observes MMIO writes via shared memory region
Chipset core processes DMA, audio, video
Chipset core asserts IRQ via IRQ controller
CPU takes IRQ, handles, returns
```

**Munt MT-32 synthesizer (dedicated core):**
```
CPU writes MIDI commands to Munt MMIO block
Munt core receives commands, synthesizes audio
Munt core writes PCM samples to shared audio buffer
Munt core signals completion via IRQ (optional)
```

**emu68 / PPC coprocessor (dedicated core):**
```
PPC core accesses shared RAM region directly
PPC core uses defined shared data structures for AmigaOS interop
PPC core uses MMIO block for control/status
IRQ used for synchronization events
```

**NPU / ML accelerator (hardware, e.g. RK3588):**
```
CPU writes inference request to accelerator MMIO
CPU provides input tensor in shared memory (physical address)
Accelerator processes, writes result to output buffer
Accelerator signals completion via IRQ
```

### 5.4 Memory Ordering for Inter-Domain Communication

Communication between domains requires explicit ordering. The canonical pattern is:

```
1. CPU writes data to shared memory buffer
2. CPU issues memory fence (when available)
3. CPU writes MMIO "kick" register to notify device
4. Device reads buffer (guaranteed to see CPU's writes)
5. Device writes result to output buffer
6. Device asserts IRQ
7. CPU handles IRQ, reads result
```

In v0 without fence instructions, software must rely on the sequential consistency of the single-core model. When atomics and fences arrive (section 3), they provide the formal ordering guarantee for multi-core scenarios.

### 5.5 Platform Core Map (Radxa Orion / RK3588)

The reference heterogeneous deployment on the Radxa Orion:

| Core | Type | Role |
|------|------|------|
| A76 core 0–1 | MC68k-64 | Primary CPU — AROS execution |
| A55 core 2 | Software | Chipset emulator |
| A55 core 3 | Software | Munt MT-32 synthesizer |
| A55 core 4 | Software | PPC emulator (optional, emu68-style) |
| A55 cores 5–7 | MC68k-64 | SMP expansion (future) |
| RK3588 NPU | Hardware | ML accelerator coprocessor |

On the Raspberry Pi 3B (current development platform):

| Core | Role |
|------|------|
| Core 0–1 | MC68k-64 (Rust emulator) |
| Core 2 | Chipset emulator |
| Core 3 | Munt or PPC — one at a time |

### 5.6 What Remains to Be Defined

- MMIO block assignments for each heterogeneous component (within coprocessor block at `0x0000_00FF_2210_0000`)
- Shared memory region conventions and ownership model
- Boot and initialization protocol for secondary cores
- IRQ source assignment in the IRQ controller for each component

---

## 6. JIT Compilation

### 6.1 Chosen Direction

JIT compilation is a future performance optimization. It requires no changes to the architectural specification — if the spec is correct, JIT is just an implementation.

### 6.2 Decisions Already Made for This

The fixed 32-bit instruction encoding was chosen with JIT in mind:

| Property | Benefit for JIT |
|----------|----------------|
| Fixed 32-bit base | Decoder never needs lookahead — translation units are regular |
| EXT count in bits 31:30 | Full instruction size known after reading 4 bytes |
| CLASS in bits 29:26 | Dispatch table indexed directly — no complex decode tree |
| No variable-length prefixes | No prefix accumulation state in the JIT frontend |
| Clean class separation | Each class maps to a small, independent translation routine |

A JIT implementation reads the EXT field, fetches 1–3 words, dispatches on CLASS, and emits host instructions. The regularity of the encoding makes this straightforward.

### 6.3 What a JIT Needs from the Spec

Nothing beyond what is already defined:

- Instruction semantics (document 04, when complete)
- Exception entry/exit model (document 05)
- Memory model (document 02)
- Privilege transitions (document 06)

A correct interpreter and a correct JIT produce identical observable behavior. The spec defines that behavior completely.

### 6.4 What Remains to Be Defined

- No architectural additions needed for basic JIT
- Potential future: self-modifying code semantics and instruction cache coherence
- Potential future: JIT hint instructions (branch prediction, prefetch)

---

## 7. Debug Extensions (DCSR)

### 7.1 Chosen Direction

v0 provides **T-bit trace** only — single-step by exception after each instruction. This is sufficient for initial bring-up and basic debugging.

A future Debug CSR (DCSR) will provide a richer debug interface without modifying the core exception model.

### 7.2 What T-Bit Trace Provides (v0)

- Single-step execution via SR.T=1
- Trace exception (vector 8) after each committed instruction
- EPC points to the next instruction — handler knows where execution will resume
- T is cleared on exception entry — handler does not trace itself
- Supervisor sets T=1 in SR_old before RTE to enable tracing of a user process

This is sufficient for:
- Kernel bring-up
- Basic debugger implementation
- Verifying exception model correctness

### 7.3 DCSR Intent

The future DCSR will provide:

| Feature | Description |
|---------|-------------|
| Hardware breakpoints | Stop execution when PC reaches a specified address, without modifying code |
| Hardware watchpoints | Stop execution when a specified memory address is read or written |
| Single-step control | Finer control than T-bit — step over, step into, step out |
| Debug halt/resume | External debug host can halt and resume execution |
| Debug cause | Identifies which breakpoint or watchpoint triggered |

DCSR access will be supervisor-only. User-mode debug support is mediated by the OS (ptrace-style interface).

### 7.4 Decisions Already Made for This

| Decision | Where defined | Relevance |
|----------|--------------|-----------|
| T-bit in SR | 01-registers | Foundation — DCSR extends this |
| T restricted to supervisor | 06-privilege-model | Debug control stays in kernel |
| SR bits 31:16 reserved | 01-registers | Space for future debug mode bits |
| Trace vector (8) reserved | 05-exception-model | Reused or extended for DCSR events |

### 7.5 What Remains to Be Defined

- DCSR register format and CSR address
- Breakpoint and watchpoint register count and format
- Debug exception vector (reuse vector 8 or new vector)
- Interaction between DCSR and SMP — per-core or global debug state

---

## 8. ARM64-BE Virtualization Path

### 8.1 Chosen Direction

The MC68k-64 is designed to run on **ARM64 (AArch64) in big-endian mode** and to progressively close the gap between emulation and native virtualization. UEFI is the boot mechanism on all target platforms.

This is not a convenience choice — it is an architectural commitment. The host endianness, privilege model, MMU structure, and exception model were all chosen with ARM64-BE alignment in mind.

### 8.2 Why ARM64-BE

| Property | Consequence |
|----------|-------------|
| Host and guest share big-endian byte order | Zero byte-swap overhead at memory boundaries |
| Eliminates swap layer permanently | Virtualization path has no endianness obstacle |
| ARM64 hardware is the substrate | Each phase of the progression runs on the same hardware |

### 8.3 UEFI Boot

The MC68k-64 emulator is a **UEFI application** on ARM64 hardware. The boot sequence is:

```
1. ARM64 hardware powers on
2. UEFI firmware initializes hardware (runs in LE as per UEFI spec)
3. UEFI loads MC68k-64 emulator from storage
4. Emulator configures ARM64 to big-endian data mode
5. Emulator initializes VM memory, loads AROS image
6. Emulator transfers control to MC68k-64 boot ROM at address 0
7. AROS boots normally within the VM
```

UEFI provides:
- Hardware enumeration and initialization
- Storage access (loading the AROS image)
- A standard boot entry point across all ARM64 platforms

This makes the emulator portable across any ARM64 platform with UEFI — Pi 4/5, Radxa Orion, and any future ARM64 server or SBC.

### 8.4 Progression Phases

```
Phase 1 — Interpreted emulator (current)
  Rust interpreter on ARM64-BE
  MC68k-64 instructions decoded and executed one at a time
  UEFI bootable
  Correct behavior is the only goal

Phase 2 — JIT compilation
  MC68k-64 instruction blocks compiled to ARM64 sequences at runtime
  Performance approaches native execution
  Same memory model and exception semantics as interpreter
  No architectural changes required

Phase 3 — Thin virtualization
  Indirection layers progressively removed
  MC68k-64 MMU tables mapped directly to ARM64 stage-1 translation tables
  MC68k-64 exceptions mapped to AArch64 exception vectors
  Memory is shared between host and guest — no copy on access
  DMA uses ARM64 physical addresses directly

Phase 4 — Native virtualization (long term)
  MC68k-64 privilege model runs as EL1/EL0 guest under ARM64 hypervisor
  AROS runs at near-native performance
  Hardware resources (DMA, IRQ, MMIO) exposed through virtualization boundary
  Heterogeneous components (Munt, chipset, PPC) remain as MMIO devices
```

### 8.5 Architectural Alignment with ARM64

The following MC68k-64 design decisions map directly to ARM64-BE constructs, enabling the virtualization progression:

| MC68k-64 | ARM64-BE | Phase where mapping is exploited |
|----------|----------|----------------------------------|
| Big-endian throughout | SCTLR_EL1.E0E + EE | Phase 1 — zero overhead from day one |
| User / Supervisor | EL0 / EL1 | Phase 3 — direct privilege mapping |
| EPC, CAUSE, VBR | ELR_EL1, ESR_EL1, VBAR_EL1 | Phase 3 — system register mapping |
| Exception stack frame | AArch64 exception entry | Phase 3 — compatible entry/exit |
| DMA in physical addresses | Stage 2 bypass | Phase 3 — no IOMMU needed |
| 3-level MMU (Sv39-like) | AArch64 stage-1 (4KB, 3 levels) | Phase 3 — direct table reuse |
| HARTID | MPIDR_EL1 | Phase 4 — core identity |
| IPI via IRQ controller | GIC SGI | Phase 4 — inter-core signaling |

### 8.6 What Remains to Be Defined

- Detailed UEFI application structure and boot protocol
- ARM64 BE mode initialization sequence
- Memory ownership model during phase 3 transition
- Hypervisor interface for phase 4 (KVM/ARM or custom)
- Exception vector mapping between MC68k-64 and AArch64

---

## 9. Summary of Architectural Commitments

The following future extensions are architectural commitments — they will be implemented, and current decisions were made to accommodate them:

| Extension | Status | Key dependency |
|-----------|--------|----------------|
| MMU (3-level, 4KB pages) | Committed | PTBR, ASID, page fault vector already in place |
| LR/SC atomics | Committed | Memory model baseline established |
| SMP (shared memory, HARTID, IPI) | Committed | HARTID, IPI vector already reserved |
| Heterogeneous execution model | Committed | MMIO contract + shared memory already defined |
| JIT compilation | Committed | Encoding regularity already guaranteed |
| DCSR debug extension | Committed | T-bit foundation already in place |
| ARM64-BE virtualization path | Committed | Architectural alignment verified |
| UEFI boot | Committed | Target platforms all support UEFI |

The following are possibilities but not commitments:

| Extension | Notes |
|-----------|-------|
| 4-level MMU (48-bit virtual) | If 39-bit proves insufficient |
| IOMMU | If DMA isolation becomes necessary |
| Hypervisor mode | If virtualization becomes a platform goal |
| CAS instruction | Convenience derivation from LR/SC |

---

*End of document 08-future.md*