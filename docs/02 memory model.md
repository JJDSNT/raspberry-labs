# MC68k-64 Architecture Specification  
## Document 02 — Memory Model  

**Status:** Stable Draft  
**Version:** 1.0  

---

## 1. Overview

The MC68k-64 defines a unified, linear, 64-bit address space. All resources — RAM, ROM, MMIO, device registers, and service interfaces — exist within this single space.

There are no separate I/O address spaces and no special I/O instructions.

This is a von Neumann architecture: instructions and data share the same address space and access mechanisms.

The architecture supports both:

- **Legacy-compatible execution** through cycle-sensitive MMIO regions  
- **Service-oriented execution** through memory-mapped Service Gates  

---

## 2. Address Space

### 2.1 Architectural Address Width

All architectural addresses are 64-bit.

Registers capable of holding addresses include:

- General-purpose registers used as pointers  
- PC (Program Counter)  
- USP / SSP (User/Supervisor Stack Pointers)  
- EPC (Exception Program Counter)  
- EADDR (Fault Address Register)  
- VBR (Vector Base Register)  

---

### 2.2 Implemented Address Range

Implementations are not required to provide the full 64-bit physical space.

A compliant implementation must:

- Accept and generate 64-bit addresses architecturally  
- Raise an address fault for accesses outside implemented ranges  
- Require upper bits of unimplemented ranges to be zero  

---

### 2.3 Address Space Model

    0x0000_0000_0000_0000  ┌─────────────────────┐
                           │  RAM                 │
                           │  (system memory)     │
                           ├─────────────────────┤
                           │  Reserved / future   │
                           ├─────────────────────┤
                           │  Device memory       │
                           │  (shared buffers,    │
                           │   DMA-coherent)      │
                           ├─────────────────────┤
                           │  MMIO               │
                           │  (chipset, Zorro,    │
                           │   service gates)     │
                           ├─────────────────────┤
    0xFFFF_FFFF_FFFF_FFFF  │  Boot / firmware     │
                           └─────────────────────┘

Exact region boundaries are defined in **Document 07 — Physical Memory Map**.

---

## 3. Byte Order

The MC68k-64 is **strictly big-endian**.

This applies to:

- RAM contents  
- Instruction encoding  
- MMIO registers  
- DMA transfers  
- Shared memory structures (including Service Gates)  

Example (32-bit value at address A):

    Address A+0 → most significant byte  
    Address A+1 → next byte  
    Address A+2 → next byte  
    Address A+3 → least significant byte  

---

## 4. Memory Access Alignment

Strict alignment is enforced.

| Access size | Required alignment |
|-------------|-------------------|
| Byte (`.b`) | Any address |
| Word (`.w`) | 2-byte boundary |
| Long (`.l`) | 4-byte boundary |
| Quad (`.q`) | 8-byte boundary |

Violations raise an alignment fault.

No hardware correction or split access is performed.

## 4.1 Atomicity and Coherence

- Aligned accesses whose size is less than or equal to the native data path width (64-bit in v1.0) are guaranteed to be atomic.

- Writes to ROM regions must raise an Access Fault.
  The system orchestrator may optionally emulate legacy behavior by intercepting and ignoring such writes.

- MMIO accesses are strongly ordered and must not be reordered relative to each other.

---

## 5. Region Types

Region type is determined solely by the physical address.

| Type | Description |
|------|-------------|
| RAM | Cacheable, coherent system memory |
| ROM / Boot | Read-only memory |
| MMIO | Control interfaces with side effects |
| Device memory | Shared or DMA-coherent memory |
| Reserved | Access fault |
| Future translated | Reserved for MMU |

---

### 5.1 MMIO Subclassification

- **Legacy MMIO**  
  Cycle-sensitive regions (e.g., classic chipset compatibility)

- **Service MMIO**  
  Control interfaces for Service Gates  

Both obey identical access rules.

---

### 5.2 Device Memory Usage

Device memory is used for:

- DMA buffers  
- Inter-core shared memory  
- Command/event queues  
- Zero-copy service communication  

---

### 5.3 Legacy Address Aliasing

To support 32-bit software, the architecture allows mapping of 64-bit regions into the lower 4GB address space.

The system supervisor (orchestrator) is responsible for:

- Providing canonical legacy mappings (e.g., `$DFF000`)  
- Maintaining behavioral compatibility  
- Translating between legacy and native address layouts  

---

## 6. Instruction Fetch

Instruction fetch is a memory read from PC.

Constraints:

- PC must be 4-byte aligned  
- Fetch from MMIO or Device Memory raises a fault  
- Fetch from Reserved regions raises a fault  

---

## 7. Memory Access Types

| Type | Description |
|------|-------------|
| Instruction fetch | PC-driven read |
| Data read/write | Load/store operations |
| DMA read/write | Device-initiated access |
| Device read/write | MMIO access |

---

## 8. DMA and Physical Addressing

DMA operates exclusively on physical addresses.

- No participation in virtual translation  
- OS/supervisor manages address translation (future MMU)  

---

## 9. MMIO Semantics

MMIO accesses have side effects.

Rules:

- No reordering within a thread  
- Never cached  
- Access size is significant  
- Alignment rules apply  

---

### 9.1 Extended Side Effects

MMIO operations may trigger:

- Interrupt generation  
- Inter-core signaling (e.g., doorbells)  
- Queue publication/consumption  
- Activation of asynchronous services  

---

## 10. Virtual Memory and Orchestration (Future)

The MMU is not part of v1.0.

The architecture assumes a system supervisor capable of:

- **Transparent Emulation**  
  Intercepting accesses to provide compatibility or services  

- **Address Mapping**  
  Implementing legacy aliasing and memory layout translation  

Future MMU integration will include:

- Virtual → physical translation  
- Address spaces via ASID  
- Page fault handling  

---

## 11. Design Principles

1. **Everything is memory**  
2. **The address decides**  
3. **Strict alignment and endianness**  
4. **DMA is physical**  
5. **Asynchronous service offload**  
6. **Synchronous control, asynchronous execution**  

---

## 12. Decisions Deferred

| Topic | Status | Document |
|------|--------|----------|
| Exact region boundaries | Deferred | 07 |
| MMU format and paging | Deferred | 08 |
| IOMMU model | Deferred | 08 |
| Cache coherence model | Deferred | 08 |
| Atomic memory operations | Deferred | 08 |
| Formal memory ordering model | Deferred | Future |
| Service Gate memory ordering profiles | Deferred | 09 |

---

*End of document 02-memory-model.md*