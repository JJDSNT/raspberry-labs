# MC68k-64 Architecture Specification
## Document 01 — Register File

**Status:** Draft  
**Version:** 0.1  

---

## 1. Overview

The MC68k-64 provides a register file organized into four groups:

- General-purpose integer registers (GPRs) — 16 × 64-bit
- Control registers — PC, SR, USP, SSP
- Exception CSRs — EPC, ECAUSE, EADDR, VBR
- System CSRs — HARTID, PTBR, ASID (reserved for future use)
- Floating-point registers (FPRs) — 16 × 64-bit + FCSR

---

## 2. General-Purpose Registers (GPRs)

The MC68k-64 has 16 general-purpose 64-bit registers, organized into two conceptual banks: data registers (D0–D7) and address registers (A0–A7).

This separation is **conventional, not architectural**. Any register may be used for any purpose. The distinction exists to preserve the 68k programming model and to aid readability of assembly code.

| Register | Alias | Convention |
|----------|-------|------------|
| D0 | — | Data / return value |
| D1 | — | Data / argument 1 |
| D2 | — | Data / argument 2 |
| D3 | — | Data / argument 3 |
| D4 | — | Data / callee-saved |
| D5 | — | Data / callee-saved |
| D6 | — | Data / callee-saved |
| D7 | — | Data / callee-saved |
| A0 | — | Address / pointer |
| A1 | — | Address / pointer |
| A2 | — | Address / pointer |
| A3 | — | Address / pointer |
| A4 | — | Address / pointer |
| A5 | FP | Frame pointer (convention) |
| A6 | LR | Link register (convention) |
| A7 | SP | Stack pointer — mirrors USP or SSP per privilege mode |

### 2.1 Register Width and Sub-register Access

All GPRs are 64-bit. Instructions operating on sub-register sizes (byte, word, long) access the low-order bits of the register. Upper bits are zero-extended on load unless the instruction specifies sign-extension.

| Size suffix | Bits accessed |
|-------------|--------------|
| `.b` | bits 7:0 |
| `.w` | bits 15:0 |
| `.l` | bits 31:0 |
| `.q` | bits 63:0 |

### 2.2 Stack Pointer (A7)

A7 is the architectural stack pointer. Its physical source depends on the current privilege mode:

- In **user mode** (SR.S = 0): A7 reads and writes USP
- In **supervisor mode** (SR.S = 1): A7 reads and writes SSP

USP and SSP are independent registers. A transition between modes does not modify either register's value — A7 simply redirects to the appropriate one.

---

## 3. Control Registers

### 3.1 Program Counter (PC)

- 64-bit register
- Holds the address of the currently executing instruction
- Updated by all instructions; directly writable only by branch, jump, and exception return instructions
- Always aligned to a 4-byte boundary (instruction fetch alignment)

### 3.2 Status Register (SR)

- 32-bit register
- Contains both system control fields (upper byte) and condition codes (lower byte)
- Bits 31:16 are reserved and must be zero

```
 31      16 15 14 13 12 11 10  9  8  7  6  5  4  3  2  1  0
┌──────────┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐
│  (res.)  │T │rs│S │rs│rs│I2│I1│I0│rs│rs│rs│X │N │Z │V │C │
└──────────┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
```

| Bit(s) | Name | Description |
|--------|------|-------------|
| 31:16 | — | Reserved, must be zero |
| 15 | T | Trace mode. When set, a trace exception is generated after each instruction completes |
| 14 | — | Reserved |
| 13 | S | Supervisor mode. 1 = supervisor, 0 = user. Controls A7 routing and access to privileged instructions |
| 12 | — | Reserved |
| 11 | — | Reserved |
| 10:8 | I2:I1:I0 | Interrupt priority mask. Interrupts at or below this level are masked. Level 7 is always accepted (non-maskable) |
| 7:5 | — | Reserved |
| 4 | X | Extend. Set by arithmetic operations that produce a carry/borrow. Used by extended-precision arithmetic (ADDX, SUBX) |
| 3 | N | Negative. Set when the result MSB is 1 |
| 2 | Z | Zero. Set when the result is zero |
| 1 | V | Overflow. Set when the result overflows the signed range |
| 0 | C | Carry. Set when an unsigned carry or borrow occurs |

**Notes:**

- Bits 12, 11, 14 are reserved and read as zero. Writing them has no effect.
- The CCR (Condition Code Register) refers informally to bits 4:0 (X, N, Z, V, C).
- Bits 31:16 are reserved for future architectural extensions. Software must not rely on their value.
- T = 1 causes a trace exception (vector TBD) after every completed instruction. Used for single-step debugging.
- The interrupt mask I2:I1:I0 encodes levels 0–7. An incoming interrupt at level N is accepted only if N > current mask value.

### 3.3 User Stack Pointer (USP)

- 64-bit register
- Holds the stack pointer value for user-mode execution
- Not directly accessible from user mode via move instructions (privileged access only)
- Readable and writable from supervisor mode via `MOVE USP, An` / `MOVE An, USP`

### 3.4 Supervisor Stack Pointer (SSP)

- 64-bit register
- Holds the stack pointer value for supervisor-mode execution
- Active as A7 whenever SR.S = 1
- Exception entry uses SSP for the exception stack frame

---

## 4. Exception CSRs

These registers are written by the processor during exception processing and read by exception handlers.

| Register | Width | Description |
|----------|-------|-------------|
| EPC | 64-bit | Exception Program Counter. Address of the instruction that caused the exception, or the return address after RTE |
| ECAUSE | 32-bit | Exception cause code. Identifies the type of exception |
| EADDR | 64-bit | Exception address. For memory faults, holds the faulting effective address |
| VBR | 64-bit | Vector Base Register. Base address of the exception vector table |

### 4.1 ECAUSE encoding

The upper 1 bit distinguishes interrupts from synchronous exceptions:

```
bit 31    = 1 → interrupt
bit 31    = 0 → synchronous exception
bits 30:0 = cause code
```

Cause codes are defined in document 05 (Exception Model).

---

## 5. System CSRs (Future)

These registers are reserved for future architectural extensions. Their presence is architectural — implementations that do not yet support the associated features must return zero on read and ignore writes.

| Register | Width | Future use |
|----------|-------|------------|
| HARTID | 64-bit | Hardware thread identifier. Unique per CPU core in an SMP system |
| PTBR | 64-bit | Page Table Base Register. Physical address of the root page table |
| ASID | 16-bit | Address Space Identifier. Used by the MMU to avoid TLB flushes on context switch |

---

## 6. Floating-Point Registers (FPRs)

The MC68k-64 provides 16 floating-point registers and one FP control/status register.

| Register | Convention |
|----------|------------|
| FP0–FP7 | Scratch / FP argument and return values |
| FP8–FP15 | Callee-saved |
| FCSR | FP control and status |

### 6.1 FPR Width and Format

All FPRs hold 64-bit IEEE 754 double-precision values. Single-precision (32-bit) operands are converted to double on load and stored as double internally.

The architecture does not support 80-bit extended precision.

### 6.2 FCSR — Floating-Point Control and Status Register

32-bit register. Controls rounding mode and records floating-point exception flags.

```
 31       8  7   5  4   0
┌──────────┬──────┬──────┐
│ reserved │  RM  │FLAGS │
└──────────┴──────┴──────┘
```

| Bits | Name | Description |
|------|------|-------------|
| 31:8 | — | Reserved |
| 7:5 | RM | Rounding mode: 000=RN (nearest), 001=RZ (zero), 010=RP (plus inf), 011=RM (minus inf) |
| 4 | NV | Invalid operation flag |
| 3 | DZ | Division by zero flag |
| 2 | OF | Overflow flag |
| 1 | UF | Underflow flag |
| 0 | NX | Inexact result flag |

Flags are sticky — they accumulate until explicitly cleared by software.

---

## 7. CSR Access Instructions

System and exception CSRs are accessed via dedicated instructions in the system instruction class (CLASS=0011). General-purpose registers are used as intermediaries:

```asm
MOVE.Q  EPC, D0       ; read EPC into D0
MOVE.Q  D0, VBR       ; write D0 into VBR
MOVE.L  ECAUSE, D1    ; read cause code
```

Access to EPC, ECAUSE, EADDR, VBR, USP, SSP, PTBR, ASID, HARTID is privileged. Attempting access from user mode raises a privilege violation exception.

FCSR is accessible from both user and supervisor mode.

---

## 8. Decisions Deferred

| Topic | Status | Document |
|-------|--------|----------|
| ABI calling convention (which regs are caller/callee saved) | Deferred | TBD |
| Full ECAUSE code table | Deferred | 05-exception-model.md |
| PTBR / ASID format and MMU page table layout | Deferred | 08-future.md |
| SMP HARTID assignment and IPI mechanism | Deferred | 08-future.md |

---

*End of document 01-registers.md*