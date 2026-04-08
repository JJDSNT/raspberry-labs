# MC68k-64 Architecture Specification
## Document 05 — Exception and Interrupt Model

**Status:** Draft  
**Version:** 0.1  

---

## 1. Principles

The MC68k-64 uses a unified exception model covering synchronous faults, explicit traps, trace, and asynchronous interrupts.

- Single model for all exception types
- User / Supervisor privilege modes
- A7 is the architectural stack pointer — routes to USP or SSP per SR.S
- T simple (single-step trace, no T2)
- 7 IRQ levels (I2:I1:I0 in SR)
- SR 32-bit with classical layout preserved in low 16 bits
- VECTOR, CAUSE, EPC and EADDR explicit in the exception frame
- IRQ is only accepted at instruction boundaries
- Trace fires after instruction commit, before next instruction
- Synchronous fault aborts instruction commit

---

## 2. Exception Classes

### Class A — Reset / Fatal

| Name | Description |
|------|-------------|
| Reset | Power-on or external reset. Full machine state reset |
| Fatal | Reserved instruction encoding, machine check, or vector fetch fault |

### Class B — Synchronous Faults

Faults abort the faulting instruction. The instruction does not commit. EPC points to the faulting instruction.

| Name | Description |
|------|-------------|
| Illegal Instruction | Instruction encoding is undefined or reserved |
| Privilege Violation | Privileged instruction executed in user mode |
| Divide by Zero | Integer divide or modulo with zero divisor |
| Address Fault | PC or EA violates alignment rules |
| Access Fault | Memory access to reserved or invalid region |
| Page Fault | MMU translation failure (reserved, future) |

### Class C — Synchronous Traps

Traps are explicit and controlled. The instruction commits. EPC points to the next instruction.

| Name | Description |
|------|-------------|
| Trap #n | Software trap instruction. Controlled entry to supervisor |
| Trace | Single-step trace. Fires after each instruction when SR.T=1 |

### Class D — Asynchronous

Accepted only at instruction boundaries. EPC points to the next instruction to execute.

| Name | Description |
|------|-------------|
| Timer IRQ | Periodic timer interrupt |
| External IRQ | External interrupt at level 1–7 |
| IPI | Inter-processor interrupt (reserved, future) |

---

## 3. Vector Table

### 3.1 Vector Base Register (VBR)

VBR holds the 64-bit physical base address of the vector table. Initialized to zero at reset. Writable only from supervisor mode.

### 3.2 Vector Stride

Each vector entry is a 64-bit handler address. Stride is 8 bytes.

```
handler_address = *(uint64_t *)(VBR + vector_number * 8)
```

### 3.3 Vector Table

| Vector | Number | Class |
|--------|--------|-------|
| Reset | 0 | A |
| Fatal | 1 | A |
| Illegal Instruction | 2 | B |
| Privilege Violation | 3 | B |
| Divide by Zero | 4 | B |
| Address Fault | 5 | B |
| Access Fault | 6 | B |
| Page Fault | 7 | B (future) |
| Trace | 8 | C |
| Trap #0 | 9 | C |
| Trap #1 | 10 | C |
| Trap #2 | 11 | C |
| Trap #3 | 12 | C |
| Timer IRQ | 13 | D |
| External IRQ level 1 | 14 | D |
| External IRQ level 2 | 15 | D |
| External IRQ level 3 | 16 | D |
| External IRQ level 4 | 17 | D |
| External IRQ level 5 | 18 | D |
| External IRQ level 6 | 19 | D |
| External IRQ level 7 | 20 | D |
| IPI | 21 | D (future) |
| 22–63 | — | Reserved |

### 3.4 Vector Zero / Uninitialized Vector

If a resolved vector entry contains zero, the CPU does not jump to address zero. Instead, it escalates to a Fatal exception (vector 1). If the Fatal vector is also zero, the machine enters halt state. This behavior is defined and mandatory — there is no undefined behavior on uninitialized vectors.

---

## 4. Exception Priority

Priority from highest to lowest:

| Priority | Condition |
|----------|-----------|
| 1 | Reset / fatal machine conditions |
| 2 | Synchronous fault of the current instruction |
| 3 | Trace (fires after instruction commit) |
| 4 | Pending IRQ accepted at boundary |
| 5 | Normal execution |

**Rules:**

- A synchronous fault on the current instruction takes priority over any pending IRQ
- Trace fires after instruction commit and before the next instruction begins
- If SR.T=1 and an IRQ is pending after an instruction completes, trace fires first. The IRQ is accepted only after the trace handler executes RTE
- IRQ is never accepted mid-instruction

---

## 5. Exception Timing

### 5.1 Synchronous Fault

When an instruction generates a fault (illegal, privilege, divide by zero, address, access, page):

- The instruction does not commit — no registers or memory are modified
- EPC is set to the address of the faulting instruction
- EADDR is set to the faulting effective address (zero if not applicable)
- The exception handler is entered

### 5.2 Explicit Trap

When a `TRAP #n` instruction executes:

- The instruction commits
- EPC is set to the address of the **next** instruction (return address)
- EADDR is set to zero
- The exception handler for Trap #n is entered

### 5.3 Trace

When SR.T=1, after every instruction that completes:

- The instruction has committed
- EPC is set to the address of the **next** instruction
- EADDR is set to zero
- The trace exception handler is entered
- SR.T is cleared on entry (as with all exceptions) — the handler does not trace itself unless it explicitly sets T=1 again

### 5.4 IRQ

An IRQ of level L is accepted at an instruction boundary if:

- Interrupts are not globally masked
- L > I_mask (current value of SR bits 10:8)

On acceptance:

- EPC is set to the address of the next instruction to execute
- EADDR is set to zero
- SR.I is updated to L (see section 6)
- The handler for External IRQ level L is entered

---

## 6. IRQ Masking

The SR field I2:I1:I0 (bits 10:8) holds the current interrupt priority mask.

**Acceptance rule:** An IRQ at level L is accepted only if L > current mask value.

**On IRQ entry:** The mask is updated to L (the level of the accepted IRQ). This means:

- IRQs at level L and below are blocked for the duration of the handler
- IRQs at level L+1 through 7 can preempt the handler
- Level 7 is always accepted regardless of mask (non-maskable)

**Example:**

```
Current mask = 2
IRQ level 3 arrives → accepted (3 > 2)
  mask updated to 3
  IRQ levels 1, 2, 3 now blocked
  IRQ levels 4, 5, 6, 7 can preempt
```

The old mask value is preserved in SR_old on the exception frame and restored by RTE.

---

## 7. Exception Entry Sequence

The following sequence is executed atomically by the processor on any exception:

**Step 1 — Capture state:**
```
old_SR   = SR
old_PC   = PC (faulting instruction) or next_PC (trap/trace/IRQ)
ECAUSE   = cause code
EADDR    = faulting address or zero
```

**Step 2 — Switch to supervisor (if not already):**
```
if SR.S == 0:
    save USP
    A7 now references SSP
```

**Step 3 — Push exception frame onto supervisor stack:**
```
SSP = SSP - 32
mem[SSP +  0] = old_SR   (32-bit)
mem[SSP +  4] = VECTOR   (32-bit)
mem[SSP +  8] = old_PC   (64-bit)
mem[SSP + 16] = ECAUSE   (32-bit)
mem[SSP + 20] = reserved (32-bit, written as zero)
mem[SSP + 24] = EADDR    (64-bit)
```

**Step 4 — Update SR:**
```
SR.S = 1          (enter supervisor)
SR.T = 0          (disable trace — handler does not trace itself)
SR.I = L          (if IRQ: set mask to accepted level; otherwise preserve)
```

**Step 5 — Fetch and jump to handler:**
```
PC = *(uint64_t *)(VBR + vector_number * 8)
if PC == 0: escalate to Fatal
```

---

## 8. Exception Frame Layout

The exception frame is **fixed at 32 bytes** for all exception types. The frame is always fully written, including fields that are not applicable to the current exception type (written as zero).

```
SSP after entry →  offset  0  :  SR_old   (32-bit)  — SR before exception
                   offset  4  :  VECTOR   (32-bit)  — vector number taken
                   offset  8  :  PC_old   (64-bit)  — return address
                   offset 16  :  CAUSE    (32-bit)  — detailed cause code
                   offset 20  :  reserved (32-bit)  — written as zero
                   offset 24  :  EADDR    (64-bit)  — fault address or zero
SSP before entry → offset 32
```

**VECTOR vs CAUSE distinction:**

| Field | Meaning |
|-------|---------|
| VECTOR | Identifies which handler was invoked (index into vector table) |
| CAUSE | Identifies the detailed reason within that handler's domain |

These may overlap for simple exceptions but diverge for complex ones:

```
VECTOR = AddressFault  +  CAUSE = UnalignedLongWrite
VECTOR = ExternalIRQ   +  CAUSE = IRQ level 3, source device X
```

**EADDR when not applicable:** written as zero by hardware. Software must not rely on EADDR for exception types where it is not defined (trace, trap, IRQ).

---

## 9. RTE — Return from Exception

`RTE` is a privileged instruction. Executing RTE from user mode raises a Privilege Violation fault.

**Sequence:**

```
Step 1 — Read frame:
    new_SR = mem[SSP + 0]   (32-bit SR_old)
    new_PC = mem[SSP + 8]   (64-bit PC_old)

Step 2 — Discard frame:
    SSP = SSP + 32

Step 3 — Restore state:
    SR = new_SR
    PC = new_PC

Step 4 — Restore stack pointer routing:
    if new_SR.S == 0:
        A7 now references USP
```

VECTOR, CAUSE, reserved, and EADDR fields are read but ignored by the hardware during RTE. They exist for software use only.

---

## 10. CAUSE Code Table

CAUSE bit 31 distinguishes interrupts from synchronous exceptions:

```
bit 31 = 1  →  interrupt (asynchronous)
bit 31 = 0  →  synchronous exception
bits 30:0   →  cause code
```

| CAUSE | Value | Description |
|-------|-------|-------------|
| Reset | 0x00000000 | Reset condition |
| Fatal | 0x00000001 | Fatal / machine check |
| IllegalInstr | 0x00000002 | Undefined instruction encoding |
| PrivViolation | 0x00000003 | Privileged instruction in user mode |
| DivByZero | 0x00000004 | Integer divide by zero |
| AddrFault | 0x00000005 | Misaligned or invalid PC/EA |
| AccessFault | 0x00000006 | Access to reserved or invalid region |
| PageFault | 0x00000007 | MMU translation failure (future) |
| Trace | 0x00000008 | Single-step trace |
| Trap0–Trap3 | 0x00000009–0x0000000C | Explicit TRAP #n |
| TimerIRQ | 0x80000000 | Timer interrupt |
| ExternalIRQ | 0x80000001–0x80000007 | External IRQ level 1–7 |
| IPI | 0x80000008 | Inter-processor interrupt (future) |

Sub-cause detail (e.g. UnalignedLongWrite, IRQ source device) is carried in the reserved bits of the CAUSE field in future revisions, or in device-specific registers readable by the handler.

---

## 11. Decisions Deferred

| Topic | Status | Document |
|-------|--------|----------|
| Page fault CAUSE sub-codes | Deferred | 08-future.md |
| IPI delivery mechanism and CAUSE encoding | Deferred | 08-future.md |
| IRQ controller interface and source identification | Deferred | 07-memory-map.md |
| TRAP #n range extension beyond #3 | Deferred | 04-instruction-set.md |
| Nested exception depth limit | Deferred | TBD |
| Debug exception (DCSR) | Deferred | TBD |

---

*End of document 05-exception-model.md*