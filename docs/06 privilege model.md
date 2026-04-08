# MC68k-64 Architecture Specification
## Document 06 — Privilege Model

**Status:** Draft  
**Version:** 0.1  

---

## 1. Overview

The MC68k-64 implements a two-level privilege model: **User** and **Supervisor**. No additional privilege levels (hypervisor, machine mode, interrupt mode) exist in v0. The architecture reserves space in the SR for future privilege extensions.

---

## 2. Privilege Levels

| Level | SR.S | Description |
|-------|------|-------------|
| User | 0 | Normal application execution. Restricted access to control state |
| Supervisor | 1 | Kernel, exception handlers, device drivers. Full access to machine state |

The current privilege level is determined solely by SR bit 13 (S). There is no separate mode register.

---

## 3. Stack Pointer Routing

A7 is the architectural stack pointer. Its physical source is determined by SR.S:

```
SR.S = 0  →  A7 references USP (User Stack Pointer)
SR.S = 1  →  A7 references SSP (Supervisor Stack Pointer)
```

The switch is automatic and immediate on any change to SR.S. USP and SSP are independent registers — switching mode does not modify either register's value.

---

## 4. Privilege Rules

### 4.1 User Mode — Permitted

- All integer and floating-point computation instructions
- Memory load and store to accessible regions
- `MOVE CCR, Dn` — read condition codes (bits 4:0 of SR)
- `MOVE Dn, CCR` — write condition codes (bits 4:0 of SR)
- Instructions that update CCR as part of their normal semantics
- `TRAP #n` — controlled entry to supervisor via exception mechanism
- `JSR`, `RTS`, `BSR` — subroutine call and return

### 4.2 User Mode — Not Permitted

The following operations are privileged. Attempting any of them from user mode raises a **Privilege Violation** exception (vector 3):

| Operation | Reason |
|-----------|--------|
| `MOVE SR, Dn` | Reads full SR including S, T, I_mask |
| `MOVE Dn, SR` | Writes full SR |
| `MOVE An, USP` | Access to user stack pointer from outside |
| `MOVE USP, An` | Access to user stack pointer from outside |
| Read or write any CSR (VBR, EPC, ECAUSE, EADDR, FCSR, HARTID, PTBR, ASID) | System control state |
| `RTE` | Exception return |
| `STOP` | Halt processor |
| `RESET` | Assert reset to external devices |
| Modify SR.T | Trace control is supervisor-only |
| Modify SR.I (I2:I1:I0) | Interrupt mask control |
| Access to supervisor-only MMIO regions | Device protection (enforced by MMU when present) |

### 4.3 SR vs CCR Distinction

The SR is partitioned into two accessible views:

```
SR  (32-bit, supervisor only):
  All fields — T, S, I2:I1:I0, X, N, Z, V, C, reserved

CCR (bits 4:0 of SR, user accessible):
  X, N, Z, V, C
```

| Instruction | User | Supervisor |
|-------------|------|------------|
| `MOVE CCR, Dn` | Permitted | Permitted |
| `MOVE Dn, CCR` | Permitted | Permitted |
| `MOVE SR, Dn` | Privilege Violation | Permitted |
| `MOVE Dn, SR` | Privilege Violation | Permitted |

Writing CCR from user mode modifies only bits 4:0. Bits 5:31 of SR are unaffected. This allows user code to manipulate condition flags without exposing system control state.

---

## 5. Privilege Transitions

### 5.1 User → Supervisor

The only path from user to supervisor is through the exception mechanism. There is no direct instruction to enter supervisor mode from user mode.

Triggers:
- Any synchronous fault (illegal instruction, privilege violation, divide by zero, address fault, access fault)
- Explicit `TRAP #n`
- Trace exception (SR.T=1)
- Accepted IRQ

On entry, the processor sets SR.S=1 and routes A7 to SSP. The full entry sequence is defined in document 05 (Exception Model), section 7.

### 5.2 Supervisor → User

The only path from supervisor to user mode is `RTE` restoring an SR_old with S=0.

```
RTE sequence:
  new_SR = mem[SSP + 0]
  new_PC = mem[SSP + 8]
  SSP = SSP + 32
  SR = new_SR
  PC = new_PC
  if SR.S == 0: A7 now references USP
```

The mode transition happens implicitly through SR restoration. The supervisor cannot directly set SR.S=0 other than via RTE — this ensures the stack frame is always properly consumed before returning to user mode.

### 5.3 Supervisor → Supervisor

When an exception occurs while already in supervisor mode (SR.S=1), the processor remains in supervisor mode. A7 continues to reference SSP. The exception frame is pushed onto the supervisor stack, potentially nesting handlers.

### 5.4 Transition Summary

| From | To | Mechanism |
|------|----|-----------|
| User | Supervisor | Any exception (fault, trap, IRQ, trace) |
| Supervisor | User | RTE restoring SR with S=0 |
| Supervisor | Supervisor | Any exception (nested) |
| User | User | Not possible directly |

---

## 6. Privilege Violation

A Privilege Violation exception (vector 3, CAUSE=0x00000003) is raised when:

- A privileged instruction is executed in user mode
- A user-mode access targets a supervisor-only CSR
- `RTE` is executed in user mode
- A write to SR is attempted from user mode
- A read of SR (full) is attempted from user mode

On Privilege Violation:

- The faulting instruction does not commit
- EPC points to the faulting instruction
- VECTOR = 3, CAUSE = PrivViolation
- EADDR = zero
- Exception entry proceeds normally (document 05, section 7)

The handler (typically the OS) may terminate the offending process or deliver a signal.

---

## 7. Trace and Interrupt Control from User Mode

SR.T and SR.I (I2:I1:I0) are part of the full SR and are therefore supervisor-only.

**Trace (SR.T):**
- User code cannot arm or disarm trace directly
- The supervisor can enable trace for a user process by setting T=1 in the SR_old frame before executing RTE
- This is the correct mechanism for implementing ptrace-style debugging

**Interrupt mask (SR.I):**
- User code cannot modify the interrupt priority mask
- The mask is saved and restored automatically by the exception frame mechanism

---

## 8. MMIO Protection

In v0, without an active MMU, MMIO protection is advisory — there is no hardware enforcement preventing user-mode code from accessing MMIO regions by address.

The architectural intent is:

- Supervisor-only MMIO regions exist conceptually from v0
- When the MMU is active (future), page table entries will enforce supervisor-only access
- Software (the OS) is responsible for not mapping supervisor MMIO into user address spaces

This means v0 operates in a trusted environment where user code is expected to be well-behaved or the system accepts the risk. This is consistent with the bring-up phase and with the AROS model for early development.

---

## 9. Future Privilege Extensions

The architecture reserves space for additional privilege levels without breaking v0 software:

- SR bits 31:16 are reserved for future use
- A future hypervisor or machine mode could be encoded in reserved SR bits or in a dedicated CSR
- v0 software that does not touch reserved bits will remain compatible

No commitment is made to any specific future privilege extension in this version of the specification.

---

## 10. Decisions Deferred

| Topic | Status | Document |
|-------|--------|----------|
| MMU-enforced MMIO protection | Deferred | 08-future.md |
| ptrace / debug interface for user processes | Deferred | TBD |
| Hypervisor / machine mode | Deferred | 08-future.md |
| FCSR privilege split (control vs status) | Deferred | TBD |

---

*End of document 06-privilege-model.md*