# MC68k-64 Architecture Specification
## Document 00 — Index and Overview

**Status:** Draft  
**Version:** 0.1  

---

## 1. Project Vision

The MC68k-64 is a 64-bit virtual CPU inspired by the Motorola 68k series, designed as the foundation of a modernized Amiga platform. The long-term goal is to support a port of AROS without the self-imposed limitations of existing ports — specifically with MMU and SMP support designed in from the start.

The architecture synthesizes three sources:

| Source | Contribution |
|--------|-------------|
| Motorola 68k / Musashi | Register organization (D/A banks), CCR, exception stack frames, vector table, assembly syntax and feeling |
| RISC-V | CSR model, trap/exception flow, privilege levels, foundation for MMU and atomics |
| PiStorm / emu68 | Architecture for dedicated-core execution on modern ARM SoCs |

The binary encoding is new and not compatible with any existing 68k implementation. The assembly syntax and programming model are 68k-like by design.

### 1.1 Execution Target

The MC68k-64 targets **ARM64 (AArch64) in big-endian mode** as its primary execution host. This is a deliberate and load-bearing decision — not a convenience choice.

ARM64-BE host and MC68k-64 guest share the same byte order. This eliminates the byte-swap layer that would otherwise exist at every memory access boundary, and is the prerequisite for the long-term virtualization path.

### 1.2 Virtualization Path

The project follows a deliberate progression from emulation toward virtualization:

```
Phase 1 — Interpreted emulator (current)
  ARM64-BE host running Rust interpreter
  MC68k-64 instructions decoded and executed one at a time
  Correct behavior is the only goal
  UEFI bootable on ARM64 hardware

Phase 2 — JIT compilation
  MC68k-64 instruction blocks translated to ARM64 sequences
  Performance approaches native
  Same correctness guarantees as interpreter

Phase 3 — Thin virtualization
  Indirection layers progressively removed
  MC68k-64 MMU mapped to ARM64 MMU directly
  MC68k-64 exceptions mapped to ARM64 exception model
  Memory is shared, not copied

Phase 4 — Native virtualization (long term)
  MC68k-64 as ISA of a hypervisor layer
  AROS runs as a near-native guest on ARM64-BE substrate
  Hardware resources exposed directly through the virtualization boundary
```

This path is enabled by the architectural alignment between MC68k-64 and ARM64-BE:

| MC68k-64 | ARM64-BE | Alignment |
|----------|----------|-----------|
| Big-endian throughout | BE data mode | Zero byte-swap overhead |
| User / Supervisor | EL0 / EL1 | Direct privilege mapping |
| CSRs (EPC, CAUSE, VBR...) | System registers | Structural equivalence |
| Exception stack frame | AArch64 exception model | Compatible entry/exit model |
| DMA in physical addresses | Stage 2 translation bypass | Clean IOMMU boundary |
| 3-level MMU (Sv39-like) | AArch64 stage 1 translation | Direct mapping candidate |

---

## 2. Design Principles

1. **Inspired by the Amiga, not bound by it.** Compatibility is not a goal. Familiarity and elegance are.
2. **Everything is memory.** No separate I/O space. Devices are accessed by address.
3. **The address decides, not the instruction.** No memory-type bits in instructions.
4. **Big-endian throughout.** RAM, instructions, MMIO, DMA. No exceptions.
5. **Strict alignment.** Misaligned access faults. No silent correction.
6. **Fixed 32-bit instruction base.** Extensions are explicit and counted in the base word.
7. **Fewer formats, more uniformity.** The decoder is simple. Complexity lives in the assembler.
8. **Reserve bits rather than fill them.** Future extensions should not require breaking changes.
9. **Correct before fast.** JIT, superscalar, and pipeline optimizations are future concerns.
10. **ARM64-BE as substrate.** The host architecture is not incidental — it is the virtualization target.
11. **UEFI bootable.** The emulator is a first-class UEFI application on ARM64 hardware.

---

## 3. Document Index

| Document | Title | Status |
|----------|-------|--------|
| 00-overview.md | Index and Overview (this document) | Draft |
| 01-registers.md | Register File | Draft |
| 02-memory-model.md | Memory Model | Draft |
| 03-instruction-encoding.md | Instruction Encoding | Draft |
| 04-instruction-set.md | Instruction Set Reference | Not started |
| 05-exception-model.md | Exception and Interrupt Model | Draft |
| 06-privilege-model.md | Privilege and Protection Model | Draft |
| 07-memory-map.md | Physical Memory Map | Draft |
| 08-future.md | Future Extensions and Architectural Intent | Draft |

---

## 4. Quick Reference

### Register summary

```
Integer:   D0–D7  (data, 64-bit)     A0–A7  (address, 64-bit)
           A7 = USP (user) or SSP (supervisor) per SR.S

Control:   PC     SR     USP    SSP
Exception: EPC    ECAUSE EADDR  VBR
Future:    HARTID PTBR   ASID

Float:     FP0–FP15 (IEEE-754 double)   FCSR
```

### SR bit layout

```
bit 15 = T  (trace)
bit 13 = S  (supervisor)
bits 10:8 = I2:I1:I0  (interrupt mask, levels 0–7)
bit 4 = X  (extend)
bit 3 = N  (negative)
bit 2 = Z  (zero)
bit 1 = V  (overflow)
bit 0 = C  (carry)
bits 31:16 reserved
```

### Instruction encoding

```
bits 31:30 = EXT   (0=none, 1=+1 word, 2=+2 words)
bits 29:26 = CLASS (0000=ALU, 0001=MOVE, 0010=BRANCH,
                    0011=SYSTEM, 0100=IMMED, 0101=SHIFT,
                    0110=FLOAT)
bits 25:24 = SIZE  (00=byte, 01=word, 10=long, 11=quad)
bits 23:0  = PAYLOAD (class-specific)
```

### EA modes

```
000 = Dn          001 = An
010 = (An)        011 = (An)+
100 = -(An)       101 = d(An)
110 = d(An,Xn)    111 = special (imm/abs/pcrel)
```

---

## 5. Target Platform

All target platforms run ARM64 in big-endian mode. UEFI is the boot mechanism.

**Development platform — Raspberry Pi 3B:**

| Core | Role |
|------|------|
| Core 0–1 | MC68k-64 interpreter (Rust) |
| Core 2 | Chipset emulator |
| Core 3 | Munt MT-32 or PPC emulator |

**Reference platform — Radxa Orion (RK3588):**

| Core | Type | Role |
|------|------|------|
| Core 0–1 | Cortex-A76 | MC68k-64 interpreter / JIT |
| Core 2 | Cortex-A55 | Chipset emulator |
| Core 3 | Cortex-A55 | Munt MT-32 synthesizer |
| Core 4 | Cortex-A55 | PPC emulator (optional) |
| Core 5–7 | Cortex-A55 | AROS SMP (future) |
| NPU | RK3588 | ML accelerator coprocessor |

The emulator is designed for correctness first. JIT and virtualization are future phases. See document 08 for the full progression.

---

## 6. Relationship to Existing Projects

| Project | Relationship |
|---------|-------------|
| MC64000 (IntuitionAmiga) | Proof of concept for 68k-64bit. Reference for ISA ideas. Not used as code base — different endianness, missing supervisor model, different encoding goals |
| Musashi | Reference for execution style, exception stack, vector organization |
| RISC-V | Source for CSR model, trap flow, privilege architecture |
| emu68 | Source of the dedicated-core execution concept |
| AROS | Target operating system for the platform |

---

*End of document 00-overview.md*