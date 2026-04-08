# MC68k-64 Architecture Specification
## Document 04 — Instruction Set Reference

**Status:** Draft  
**Version:** 0.1  

---

## Conventions

### Operand class names

| Class | Meaning |
|-------|---------|
| `data` | Data register direct: D0–D7 |
| `addr` | Address register direct: A0–A7 |
| `mem` | Memory: (An), (An)+, -(An), d(An), d(An,Xn) |
| `imm` | Immediate literal: #n |
| `abs` | Absolute address: (addr).w / (addr).l |
| `pcrel` | PC-relative: d(PC) |

### CCR table notation

| Symbol | Meaning |
|--------|---------|
| `*` | Updated by result |
| `0` | Forced to zero |
| `—` | Preserved unchanged |
| `R` | Restored from exception frame (RTE only) |

### Size suffixes

| Suffix | Width |
|--------|-------|
| `.b` | 8-bit byte |
| `.w` | 16-bit word |
| `.l` | 32-bit long |
| `.q` | 64-bit quad |

### Data register write policy

When a data register (Dn) is the destination of any operation, the following rule applies universally across all instruction families:

| Size | Bits written | Upper bits |
|------|-------------|------------|
| `.b` | 7:0 | 63:8 preserved |
| `.w` | 15:0 | 63:16 preserved |
| `.l` | 31:0 | 63:32 cleared to zero |
| `.q` | 63:0 | — full write |

This rule applies to MOVE, all ALU operations, shifts, and any other instruction with a data register destination. It is not repeated in individual instruction entries unless there is a specific exception.

Address register (An) destination behavior is defined per instruction where applicable.

---

Unless stated otherwise, any effective address mode valid for a source operand is also valid for a destination operand, with the following universal restrictions:

- `imm` is never valid as a destination
- `pcrel` is never valid as a destination
- `addr` direct is not valid for `.b` size operations

Instruction-specific restrictions are listed under each entry.

---

## Group 1 — Data Movement

---

### MOVE — Move data

**Description:** Copy source operand to destination operand.

**Syntax:** `MOVE.{size} <src>, <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Source: `data, addr, mem, imm, abs, pcrel`
- Destination: `data, addr, mem, abs`

**Restrictions:**
- `MOVE.B An, <dst>` is invalid — address registers have no byte-accessible form
- `MOVE.{any} <src>, An` is valid but `MOVE.B <src>, An` is invalid for the same reason
- For loading an immediate value into an address register, use `LEA` or `MOVEI` (see MOVEQ entry)
- `MOVE.{size} <src>, SR` and `MOVE.{size} SR, <dst>` are privileged system forms documented in Group 6 (System)
- `MOVE.{size} <src>, CCR` and `MOVE.{size} CCR, <dst>` are user-accessible forms documented in Group 6 (System)

**Operation:**
```
dst ← src
```

Sub-register behavior when destination is a data register:

```
MOVE.Q D0, D1   ; D1[63:0]  ← D0[63:0]                      full write
MOVE.L D0, D1   ; D1[31:0]  ← D0[31:0],  D1[63:32] ← 0      zero-extend
MOVE.W D0, D1   ; D1[15:0]  ← D0[15:0],  D1[63:16] unchanged preserve
MOVE.B D0, D1   ; D1[7:0]   ← D0[7:0],   D1[63:8]  unchanged preserve
```

Rule: `.l` zeroes the upper 32 bits. `.b` and `.w` preserve upper bits. `.q` writes the full register.

When destination is a memory location, only the bytes corresponding to size are written.

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| 0 | 0 | * | * | — |

Z and N are set based on the value written, at the specified size.

**Exceptions:**
- Address Fault — effective address violates alignment for the specified size
- Access Fault — effective address falls in a reserved or invalid region
- Privilege Violation — destination or source is SR (use system forms)

**Encoding class:** Class 1 (MOVE)

**Example:**
```asm
MOVE.L  D0, (A1)        ; write low 32 bits of D0 to memory at A1
MOVE.Q  D2, D3          ; copy full 64-bit value from D2 to D3
MOVE.W  #$FF00, D0      ; load immediate word into D0[15:0]
MOVE.L  (A0)+, D1       ; load long from memory at A0, post-increment A0 by 4
```

**Notes:**
- `MOVE An, <dst>` is valid for `.w`, `.l`, `.q` — the full address register value is the source
- When source is `An` and size is `.w`, the 16-bit value is sign-extended before being used as the source value for CCR computation, but only 16 bits are written to a memory destination

---

### MOVEQ — Move quick (short immediate load)

**Description:** Load a small signed immediate value into a data register, sign-extended to 64 bits.

**Syntax:** `MOVEQ #<imm8>, <dst>`

**Sizes:** Always quad (64-bit result). No size suffix used.

**Operand classes:**
- Source: 8-bit signed immediate literal (−128 to +127)
- Destination: `data`

**Restrictions:**
- Destination must be a data register (D0–D7)
- Immediate value is limited to 8 bits signed (−128 to +127)
- For larger immediates, use `MOVE.{size} #imm, Dn` (assembler selects IMMED class encoding)

**Operation:**
```
dst ← SignExtend64(imm8)
```

The 8-bit immediate is sign-extended to the full 64-bit register width. The entire register is written.

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| 0 | 0 | * | * | — |

Z and N are set based on the full 64-bit result.

**Exceptions:**
- None. MOVEQ operates on registers only.

**Encoding class:** Class 4 (IMMED) — short immediate form

**Example:**
```asm
MOVEQ   #0,  D0     ; clear D0 (fast register zero)
MOVEQ   #1,  D1     ; load 1 into D1
MOVEQ   #-1, D2     ; load 0xFFFFFFFFFFFFFFFF into D2
```

**Notes:**
- `MOVEQ` is an assembler-visible short immediate load form encoded through the IMMED class. It is retained as a first-class mnemonic for readability and source familiarity, but does not require a dedicated architectural instruction class.
- The assembler will automatically select the MOVEQ encoding when the destination is `Dn` and the immediate fits in 8 bits signed. For larger values, it falls back to the general IMMED class encoding with extension words.
- Because MOVEQ always produces a 64-bit result, it is the preferred way to zero a register (`MOVEQ #0, Dn`) or load small constants in performance-sensitive code.

---

### LEA — Load effective address

**Description:** Compute an effective address and load it into an address register. Does not access memory.

**Syntax:** `LEA <ea>, <dst>`

**Sizes:** Always quad (64-bit address). No size suffix used.

**Operand classes:**
- Source: `mem, abs, pcrel` — any EA that computes an address (not `data`, `addr`, or `imm`)
- Destination: `addr` — address registers only (A0–A7)

**Restrictions:**
- Source must be a computable effective address — register direct and immediate are not valid sources
- Destination must be an address register
- LEA does not perform a memory read — it computes the address only

**Operation:**
```
dst ← EffectiveAddress(src)
```

The effective address of the source operand is computed and written into the destination address register. No memory access occurs.

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

LEA has no effect on any condition code.

**Exceptions:**
- Address Fault — if the computed effective address is misaligned in a way that would fault on access (implementation note: LEA itself does not access memory, but some implementations may validate the address)

**Encoding class:** Class 1 (MOVE)

**Example:**
```asm
LEA     (A0), A1            ; A1 ← address currently in A0
LEA     16(A0), A1          ; A1 ← A0 + 16
LEA     8(A0, D0.L*4), A1   ; A1 ← A0 + D0*4 + 8
LEA     label(PC), A0       ; A0 ← PC-relative address of label
```

**Notes:**
- `LEA label(PC), An` is the standard way to load the address of a data structure or code label in position-independent code
- `LEA #imm, An` is not valid — use `MOVEQ` or `MOVE.Q #imm, Dn` then transfer, or use `LEA imm.l, An` with an absolute address form
- LEA is the correct way to load a pointer constant into an address register, as it sets no condition codes and makes the intent explicit

---

### MOVEM — Move multiple registers

**Description:** Transfer a set of registers to or from consecutive memory locations.

**Syntax:**
```
MOVEM.{size} <reglist>, <dst>   ; registers to memory
MOVEM.{size} <src>, <reglist>   ; memory to registers
```

**Sizes:** `w, l, q`

**Operand classes:**
- Memory operand: `mem, abs` — register direct and immediate are not valid
- Register list: any combination of D0–D7 and A0–A7, specified as a bitmap or assembler list syntax

**Restrictions:**
- The memory operand must be a memory EA, not a register
- For the pre-decrement form `-(An)`, only register-to-memory direction is valid
- For the post-increment form `(An)+`, only memory-to-register direction is valid
- `.b` size is not supported

**Operation:**

Register order is always fixed and independent of transfer direction:

```
D0, D1, D2, D3, D4, D5, D6, D7, A0, A1, A2, A3, A4, A5, A6, A7
```

This order is invariant — it does not change based on whether the transfer is to or from memory.

**Registers to memory** (e.g. `MOVEM.L <reglist>, (A0)`):
```
addr ← EffectiveAddress(dst)
for each register R in reglist, in fixed order D0→A7:
    mem[addr] ← R[size]
    addr ← addr + sizeof(size)
```

**Memory to registers** (e.g. `MOVEM.L (A0), <reglist>`):
```
addr ← EffectiveAddress(src)
for each register R in reglist, in fixed order D0→A7:
    R ← SignExtend64(mem[addr][size])    ; for .w and .l, sign-extend to 64 bits
    addr ← addr + sizeof(size)
```

**Pre-decrement form** (`MOVEM.{size} <reglist>, -(An)`):
```
for each register R in reglist, in reverse order A7→D0:
    An ← An - sizeof(size)
    mem[An] ← R[size]
```

The pre-decrement form uses reverse order so that the frame can be restored with the post-increment form in the natural forward order.

**Post-increment form** (`MOVEM.{size} <src>, <reglist>` where src is `(An)+`):
```
for each register R in reglist, in fixed order D0→A7:
    R ← SignExtend64(mem[An][size])
    An ← An + sizeof(size)
```

**Base register in register list:** If the address register used as the memory base is also included in the register list, the behavior is fully defined:

- The effective address is computed from the **original value** of the base register before any transfer begins
- If the base register is in the list, the **original value** is the one transferred to or from memory
- The post-increment or pre-decrement update of the base register occurs normally after all transfers complete

This means the transferred value and the addressing base are always the pre-transfer value, and the final register value reflects the EA update. There is no undefined behavior.

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

MOVEM has no effect on any condition code.

**Exceptions:**
- Address Fault — any memory access violates alignment for the specified size
- Access Fault — any memory access falls in a reserved or invalid region

**Encoding class:** Class 1 (MOVE)

**Example:**
```asm
MOVEM.L D0-D7/A0-A6, -(SP)     ; push all working registers onto stack
MOVEM.L (SP)+, D0-D7/A0-A6     ; pop all working registers from stack
MOVEM.Q D0/D2/A1, (A5)         ; store three specific registers to memory at A5
```

**Notes:**
- The pre-decrement / post-increment pair is the standard register save/restore idiom for subroutine prologue and epilogue
- The fixed register order (D0→A7 forward, A7→D0 reverse for pre-decrement) is a deliberate simplification from the 68k classical model, which had direction-dependent ordering quirks. The MC68k-64 order is always predictable.
- When loading with `.w` or `.l`, values are sign-extended to 64 bits on load into registers. Use `.q` to preserve full 64-bit register state.
- If the base address register appears in the register list, the original pre-transfer value is used for both addressing and the transfer itself. The EA update (post-increment or pre-decrement) occurs after all transfers complete. This behavior is fully defined — there is no undefined case.

---

*End of Group 1 — Data Movement*

---

## Group 2 — Integer ALU

---

### ADD — Add

**Description:** Add source operand to destination operand and store the result.

**Syntax:** `ADD.{size} <src>, <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Source: `data, addr, mem, imm, abs, pcrel`
- Destination: `data, mem, abs`

**Restrictions:**
- `addr` is not valid as destination — for address arithmetic use `LEA` or `ADDA` (future)
- When source is `addr`, size must be `.w` or `.l` or `.q` — no `.b` on address registers
- `imm` is not valid as destination

**Operation:**
```
dst ← dst + src
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| * | * | * | * | * |

- C: set if unsigned carry out of the MSB
- V: set if signed overflow (result exceeds signed range of size)
- Z: set if result is zero
- N: set if result MSB is 1
- X: set equal to C

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 0 (ALU)

**Example:**
```asm
ADD.L   D0, D1          ; D1 ← D1 + D0 (32-bit, upper 32 cleared)
ADD.Q   D0, D1          ; D1 ← D1 + D0 (64-bit)
ADD.W   #4, D0          ; D0[15:0] ← D0[15:0] + 4
ADD.L   (A0), D2        ; D2 ← D2 + mem[A0] (32-bit)
```

**Notes:**
- For 64-bit address arithmetic, use `.q`
- Extended-precision addition (chained carry via X flag) will be provided by `ADDX` in a future extension group

---

### SUB — Subtract

**Description:** Subtract source operand from destination operand and store the result.

**Syntax:** `SUB.{size} <src>, <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Source: `data, addr, mem, imm, abs, pcrel`
- Destination: `data, mem, abs`

**Restrictions:**
- Same as ADD

**Operation:**
```
dst ← dst - src
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| * | * | * | * | * |

- C: set if unsigned borrow (dst < src unsigned)
- V: set if signed overflow
- Z: set if result is zero
- N: set if result MSB is 1
- X: set equal to C

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 0 (ALU)

**Example:**
```asm
SUB.L   D0, D1          ; D1 ← D1 - D0 (32-bit)
SUB.Q   #1, D0          ; D0 ← D0 - 1 (64-bit)
SUB.W   (A0)+, D2       ; D2[15:0] ← D2[15:0] - mem[A0], A0 += 2
```

**Notes:**
- Extended-precision subtraction via X flag will be provided by `SUBX` in a future extension group

---

### AND — Bitwise AND

**Description:** Perform bitwise AND of source and destination operands.

**Syntax:** `AND.{size} <src>, <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Source: `data, mem, imm, abs, pcrel`
- Destination: `data, mem, abs`

**Restrictions:**
- `addr` is not valid as source or destination for AND

**Operation:**
```
dst ← dst AND src
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| 0 | 0 | * | * | — |

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 0 (ALU)

**Example:**
```asm
AND.L   #$FF, D0        ; isolate low byte of D0[31:0], upper 32 cleared
AND.Q   D1, D0          ; 64-bit bitwise AND
AND.B   #%11110000, D2  ; mask high nibble of byte
```

---

### OR — Bitwise OR

**Description:** Perform bitwise OR of source and destination operands.

**Syntax:** `OR.{size} <src>, <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Source: `data, mem, imm, abs, pcrel`
- Destination: `data, mem, abs`

**Restrictions:**
- `addr` is not valid as source or destination for OR

**Operation:**
```
dst ← dst OR src
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| 0 | 0 | * | * | — |

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 0 (ALU)

**Example:**
```asm
OR.L    #$80000000, D0  ; set sign bit of D0[31:0]
OR.Q    D1, D0          ; 64-bit bitwise OR
```

---

### EOR — Bitwise Exclusive OR

**Description:** Perform bitwise exclusive OR of source and destination operands.

**Syntax:** `EOR.{size} <src>, <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Source: `data, mem, imm, abs, pcrel`
- Destination: `data, mem, abs`

**Restrictions:**
- `addr` is not valid as source or destination for EOR

**Operation:**
```
dst ← dst EOR src
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| 0 | 0 | * | * | — |

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 0 (ALU)

**Example:**
```asm
EOR.L   D0, D1          ; toggle bits in D1[31:0] where D0 is set
EOR.Q   #-1, D0         ; invert all bits of D0 (equivalent to NOT.Q)
EOR.B   #1, (A0)        ; toggle bit 0 of byte at A0
EOR.L   (A1), D2        ; XOR memory long into D2
```

---

### NOT — Bitwise complement

**Description:** Invert all bits of the destination operand.

**Syntax:** `NOT.{size} <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Destination: `data, mem, abs`

**Restrictions:**
- `addr` is not valid as destination
- NOT is a unary operation — no source operand

**Operation:**
```
dst ← NOT dst
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| 0 | 0 | * | * | — |

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 1 (MOVE/unary)

**Example:**
```asm
NOT.L   D0              ; invert D0[31:0], upper 32 cleared
NOT.Q   D1              ; invert all 64 bits of D1
NOT.B   (A0)            ; invert byte at memory address A0
```

---

### NEG — Negate (two's complement)

**Description:** Negate the destination operand (two's complement).

**Syntax:** `NEG.{size} <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Destination: `data, mem, abs`

**Restrictions:**
- `addr` is not valid as destination
- NEG is a unary operation — no source operand

**Operation:**
```
dst ← 0 - dst
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| * | * | * | * | * |

- C: set if result is non-zero (borrow from zero); cleared if result is zero
- V: set if destination was the minimum signed value (overflow on negation)
- Z: set if result is zero
- N: set if result MSB is 1
- X: set equal to C — always, without exception

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 1 (MOVE/unary)

**Example:**
```asm
NEG.L   D0              ; D0 ← -D0 (32-bit), upper 32 cleared
NEG.Q   D1              ; D1 ← -D1 (64-bit)
NEG.W   (A0)            ; negate word at memory address A0
```

**Notes:**
- X is always set equal to C, with no exception for the zero result case. This is a deliberate simplification from the 68k classical behavior where `NEG` of zero set C=0 but X=1 on some implementations.
- `NEG` of the minimum signed value (e.g. `$80000000` for `.l`) sets V=1 and produces the same value (overflow wraps).

---

*End of Group 2 — Integer ALU*

---

## Group 3 — Comparison and Test

---

### CMP — Compare

**Description:** Subtract source from destination and set condition codes. Neither operand is modified.

**Syntax:** `CMP.{size} <src>, <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Source: `data, addr, mem, imm, abs, pcrel`
- Destination: `data, addr, mem, abs`

**Restrictions:**
- `addr` operands (source or destination) are not valid for `.b` size
- `imm` is not valid as destination

**Operation:**
```
temp ← dst - src
CCR updated from temp
dst and src unchanged
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| * | * | * | * | — |

- C: set if unsigned borrow (dst < src unsigned)
- V: set if signed overflow
- Z: set if dst == src
- N: set if result MSB is 1
- X: always preserved — CMP never modifies X

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 0 (ALU) — writeback=0 variant

**Example:**
```asm
CMP.L   D0, D1          ; set CCR from D1 - D0, neither modified
CMP.Q   A0, A1          ; compare two address registers (64-bit)
CMP.W   #$FF, D0        ; compare D0[15:0] against 255
CMP.Q   (A0)+, D2       ; compare D2 against memory quad, advance A0
```

**Notes:**
- There is no `CMPA` instruction. Address registers participate in `CMP` directly for sizes `.w`, `.l`, and `.q`.
- The operand order follows the 68k convention: `CMP src, dst` computes `dst - src`. This means `CMP.L D0, D1` asks "is D1 related to D0?" and branches after CMP test the relationship of dst to src.
- After `CMP src, dst`: `BEQ` branches if dst==src, `BGT` if dst>src signed, `BHI` if dst>src unsigned.

---

### TST — Test

**Description:** Test an operand against zero and set condition codes.

**Syntax:** `TST.{size} <dst>`

**Sizes:** `b, w, l, q`

**Operand classes:**
- Destination: `data, mem, abs`

**Restrictions:**
- `addr` is not valid — use `CMP.{size} #0, An` to test an address register against zero
- `imm` is not valid
- TST is a unary operation — no source operand

**Operation:**
```
temp ← dst - 0
CCR updated from temp
dst unchanged
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| 0 | 0 | * | * | — |

- C and V are always cleared
- Z: set if dst is zero
- N: set if dst MSB is 1
- X: always preserved

**Exceptions:**
- Address Fault — memory operand violates alignment
- Access Fault — memory operand in invalid region

**Encoding class:** Class 0 (ALU) — writeback=0, zero-source variant

**Example:**
```asm
TST.L   D0              ; test D0[31:0] for zero or negative
TST.Q   D1              ; test full 64-bit D1
TST.B   (A0)            ; test byte at A0
BEQ     .zero           ; branch if result was zero
BMI     .negative       ; branch if result was negative
```

**Notes:**
- `TST` is the preferred way to test a data register or memory location against zero — more readable than `CMP #0, dst` and produces a more compact encoding.
- To test an address register against zero, use `CMP.Q #0, An` explicitly.

---

*End of Group 3 — Comparison and Test*

---

## Group 4 — Shifts

### Shift count rules (apply to all shift instructions)

Two count forms are valid for all shift instructions:

```asm
LSL.L   #3, D0      ; immediate count — small literal embedded in encoding
LSL.L   D1, D0      ; register count — count taken from low bits of D1
```

**Effective count:** the count is reduced modulo the operand size before the shift is applied.

| Size | Modulus | Effective count range |
|------|---------|----------------------|
| `.b` | mod 8 | 0–7 |
| `.w` | mod 16 | 0–15 |
| `.l` | mod 32 | 0–31 |
| `.q` | mod 64 | 0–63 |

This rule applies regardless of whether the count comes from an immediate or a register.

**Count zero behavior:** when the effective count is zero (after modulo reduction):
- Result equals the original operand
- N and Z are updated based on the original value
- C and X are preserved unchanged
- V is set to 0

**Scope in this document:** shift instructions in the core minimum operate on data registers only. Memory shift forms are reserved for a future extension.

---

### LSL — Logical Shift Left

**Description:** Shift destination left by count positions, filling vacated bits with zero.

**Syntax:**
```
LSL.{size} #<count>, <dst>
LSL.{size} <cnt>, <dst>
```

**Sizes:** `b, w, l, q`

**Operand classes:**
- Count: immediate literal, or `data` register
- Destination: `data`

**Operation:**
```
effective_count ← count mod size_in_bits
if effective_count == 0:
    C, X preserved; N, Z updated; V ← 0
else:
    C ← last bit shifted out (MSB side)
    dst ← dst << effective_count  (vacated bits = 0)
    X ← C
    N, Z updated from result
    V ← 0
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| § | 0 | * | * | § |

§ See count zero rules above.

**Encoding class:** Class 5 (SHIFT)

**Example:**
```asm
LSL.L   #1, D0      ; D0[31:0] ← D0[31:0] << 1, upper 32 cleared
LSL.Q   #3, D1      ; D1 ← D1 << 3
LSL.W   D2, D0      ; D0[15:0] ← D0[15:0] << (D2 mod 16)
```

**Notes:**
- LSL by 1 is equivalent to an unsigned multiply by 2
- The last bit shifted out of the MSB position is captured in C and X

---

### LSR — Logical Shift Right

**Description:** Shift destination right by count positions, filling vacated bits with zero.

**Syntax:**
```
LSR.{size} #<count>, <dst>
LSR.{size} <cnt>, <dst>
```

**Sizes:** `b, w, l, q`

**Operand classes:**
- Count: immediate literal, or `data` register
- Destination: `data`

**Operation:**
```
effective_count ← count mod size_in_bits
if effective_count == 0:
    C, X preserved; N, Z updated; V ← 0
else:
    C ← last bit shifted out (LSB side)
    dst ← dst >> effective_count  (vacated bits = 0, unsigned fill)
    X ← C
    N, Z updated from result
    V ← 0
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| § | 0 | * | * | § |

§ See count zero rules above.

**Encoding class:** Class 5 (SHIFT)

**Example:**
```asm
LSR.L   #1, D0      ; D0[31:0] ← D0[31:0] >> 1 (unsigned), upper 32 cleared
LSR.Q   #4, D1      ; D1 ← D1 >> 4 (unsigned)
LSR.W   D2, D0      ; D0[15:0] ← D0[15:0] >> (D2 mod 16)
```

**Notes:**
- LSR always fills with zero — result is always non-negative regardless of original sign
- LSR by 1 is equivalent to an unsigned divide by 2 (truncated)

---

### ASL — Arithmetic Shift Left

**Description:** Shift destination left by count positions, filling vacated bits with zero. Sets overflow if sign bit changes during shift.

**Syntax:**
```
ASL.{size} #<count>, <dst>
ASL.{size} <cnt>, <dst>
```

**Sizes:** `b, w, l, q`

**Operand classes:**
- Count: immediate literal, or `data` register
- Destination: `data`

**Operation:**
```
effective_count ← count mod size_in_bits
if effective_count == 0:
    C, X preserved; N, Z updated; V ← 0
else:
    C ← last bit shifted out (MSB side)
    V ← 1 if any bit shifted through the sign position differs from the original sign bit
    dst ← dst << effective_count  (vacated bits = 0)
    X ← C
    N, Z updated from result
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| § | * | * | * | § |

§ See count zero rules above. V is set if any bit shifted out of the sign position differs from the original sign — this detects signed overflow on left shift.

**Encoding class:** Class 5 (SHIFT)

**Example:**
```asm
ASL.L   #1, D0      ; D0 ← D0 * 2 (signed), V set if overflow
ASL.Q   #3, D1      ; D1 ← D1 * 8 (signed), V set if overflow
```

**Notes:**
- ASL and LSL produce the same bit pattern in the result. The difference is V: ASL detects signed overflow, LSL does not.
- Use ASL when operating on signed values and overflow detection matters. Use LSL for unsigned shifts or when overflow is not of interest.

---

### ASR — Arithmetic Shift Right

**Description:** Shift destination right by count positions, filling vacated bits with the original sign bit (sign extension).

**Syntax:**
```
ASR.{size} #<count>, <dst>
ASR.{size} <cnt>, <dst>
```

**Sizes:** `b, w, l, q`

**Operand classes:**
- Count: immediate literal, or `data` register
- Destination: `data`

**Operation:**
```
effective_count ← count mod size_in_bits
sign_bit ← dst[MSB]
if effective_count == 0:
    C, X preserved; N, Z updated; V ← 0
else:
    C ← last bit shifted out (LSB side)
    dst ← dst >> effective_count  (vacated bits = sign_bit)
    X ← C
    N, Z updated from result
    V ← 0
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| § | 0 | * | * | § |

§ See count zero rules above. V is always 0 — arithmetic right shift cannot produce signed overflow.

**Encoding class:** Class 5 (SHIFT)

**Example:**
```asm
ASR.L   #1, D0      ; D0 ← D0 / 2 (signed, truncated toward negative infinity)
ASR.Q   #3, D1      ; D1 ← D1 / 8 (signed)
ASR.W   D2, D0      ; D0[15:0] ← D0[15:0] >> (D2 mod 16), sign-filled
```

**Notes:**
- ASR by 1 is a signed divide by 2 that rounds toward negative infinity (not toward zero as C integer division does). For negative odd values, the results differ.
- Shifting a negative value repeatedly by ASR converges to -1 (all ones), not to zero.

---

*End of Group 4 — Shifts*

---

## Group 5 — Control Flow

### Branch displacement rule (applies to all relative branches)

For all relative control-flow instructions, the branch displacement is added to the address of the branch instruction itself — the address of the base 32-bit instruction word — not to the address of the following instruction.

```
target ← branch_pc + displacement
```

Consequences:
- `BRA #0` is an infinite loop — displacement zero branches to itself
- `BRA #4` branches to the next instruction (skips itself)
- The base address is always the branch instruction address, regardless of whether the displacement is inline (16-bit) or in an extension word (32-bit)

The assembler computes the correct displacement automatically from symbolic labels.

---

### BRA — Branch always

**Description:** Unconditional relative branch to a target address.

**Syntax:** `BRA <label>` or `BRA #<disp>`

**Sizes:** None — BRA has no size suffix. Displacement width is selected by the assembler.

**Operand classes:**
- Target: PC-relative displacement (16-bit inline or 32-bit extension word)

**Operation:**
```
PC ← branch_pc + displacement
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

**Exceptions:**
- Address Fault — target address is misaligned (not 4-byte aligned)
- Access Fault — target address is in an invalid region

**Encoding class:** Class 2 (BRANCH) — COND=T (always true), LINK=0

**Example:**
```asm
BRA     loop            ; branch to label "loop"
BRA     #0              ; infinite loop (branches to itself)
BRA     #4              ; skip to next instruction
```

---

### Bcc — Branch conditionally

**Description:** Branch to a target address if the specified condition is true.

**Syntax:** `B{cc} <label>` or `B{cc} #<disp>`

**Sizes:** None — no size suffix.

**Conditions:**

| Mnemonic | Condition | CCR test |
|----------|-----------|----------|
| `BEQ` | Equal | Z=1 |
| `BNE` | Not equal | Z=0 |
| `BCS` / `BLO` | Carry set / unsigned lower | C=1 |
| `BCC` / `BHS` | Carry clear / unsigned higher or same | C=0 |
| `BMI` | Minus (negative) | N=1 |
| `BPL` | Plus (non-negative) | N=0 |
| `BVS` | Overflow set | V=1 |
| `BVC` | Overflow clear | V=0 |
| `BHI` | Unsigned higher | C=0 and Z=0 |
| `BLS` | Unsigned lower or same | C=1 or Z=1 |
| `BGE` | Signed greater or equal | N=V |
| `BLT` | Signed less than | N≠V |
| `BGT` | Signed greater than | Z=0 and N=V |
| `BLE` | Signed less or equal | Z=1 or N≠V |

**Operation:**
```
if condition_true:
    PC ← branch_pc + displacement
else:
    PC ← branch_pc + instruction_size   ; fall through
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

**Exceptions:**
- Address Fault — target address misaligned (taken branch only)
- Access Fault — target address invalid (taken branch only)

**Encoding class:** Class 2 (BRANCH) — COND field selects condition, LINK=0

**Example:**
```asm
CMP.L   D0, D1
BEQ     .equal          ; branch if D1 == D0
BGT     .greater        ; branch if D1 > D0 (signed)
BHI     .higher         ; branch if D1 > D0 (unsigned)
BNE     .loop           ; loop while D1 != D0
```

**Notes:**
- `BRA` is the always-true form (`COND=T`). There is no `BF` (branch never) — it would be a NOP and is not useful.
- Condition `BLO` and `BCS` are aliases — same encoding. Similarly `BHS` and `BCC`. The assembler accepts both forms.

---

### BSR — Branch to subroutine

**Description:** Push the return address onto the supervisor or user stack, then branch to a target address.

**Syntax:** `BSR <label>` or `BSR #<disp>`

**Sizes:** None.

**Operation:**
```
A7 ← A7 - 8                ; push return address (8 bytes, 64-bit)
mem[A7] ← branch_pc + instruction_size   ; return address = instruction after BSR
PC ← branch_pc + displacement
```

The return address pushed is the address of the instruction immediately following the BSR — the natural point to resume after `RTS`.

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

**Exceptions:**
- Address Fault — target address or stack address misaligned
- Access Fault — target or stack address invalid

**Encoding class:** Class 2 (BRANCH) — COND=T, LINK=1

**Example:**
```asm
BSR     my_function     ; call my_function, return address on stack
BSR     #-4             ; call the previous instruction (unusual but defined)
```

**Notes:**
- BSR uses A7 as the stack pointer. In user mode A7=USP, in supervisor mode A7=SSP.
- The return address is always 64-bit (8 bytes), regardless of the address space actually in use. This keeps the stack frame consistent.
- `BSR` is the PC-relative subroutine call. For indirect calls through a pointer, use `JSR`.

---

### JMP — Jump

**Description:** Unconditional absolute or indirect jump to a target address.

**Syntax:** `JMP <ea>`

**Sizes:** None.

**Operand classes:**
- Target: `mem, abs, pcrel` — any EA that computes an address

**Restrictions:**
- `data`, `addr` direct, and `imm` are not valid as jump targets
- To jump to an address in a register, use `JMP (An)`

**Operation:**
```
PC ← EffectiveAddress(ea)
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

**Exceptions:**
- Address Fault — target address misaligned
- Access Fault — target address invalid

**Encoding class:** Class 2 (BRANCH)

**Example:**
```asm
JMP     (A0)            ; jump to address in A0
JMP     $1000.l         ; jump to absolute address
JMP     table(PC)       ; jump to PC-relative address
```

---

### JSR — Jump to subroutine

**Description:** Push the return address onto the stack, then jump to a target address.

**Syntax:** `JSR <ea>`

**Sizes:** None.

**Operand classes:**
- Target: `mem, abs, pcrel` — same as JMP

**Restrictions:**
- Same as JMP

**Operation:**
```
A7 ← A7 - 8
mem[A7] ← PC     ; PC here is the address of the instruction after JSR
PC ← EffectiveAddress(ea)
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

**Exceptions:**
- Address Fault — target or stack address misaligned
- Access Fault — target or stack address invalid

**Encoding class:** Class 2 (BRANCH) — LINK=1

**Example:**
```asm
JSR     (A0)            ; call subroutine at address in A0
JSR     func_table(A1,D0.L*8)  ; call via indexed function table
```

**Notes:**
- JSR is the indirect subroutine call. For PC-relative calls to known labels, use `BSR`.
- The return address pushed is always 64-bit (8 bytes).

---

### RTS — Return from subroutine

**Description:** Pop the return address from the stack and jump to it.

**Syntax:** `RTS`

**Sizes:** None. No operands.

**Operation:**
```
PC ← mem[A7]    ; pop 64-bit return address
A7 ← A7 + 8
```

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

**Exceptions:**
- Address Fault — stack address misaligned
- Access Fault — stack address invalid

**Encoding class:** Class 2 (BRANCH)

**Example:**
```asm
my_function:
    ; ... function body ...
    RTS             ; return to caller
```

**Notes:**
- RTS expects a 64-bit return address at the current stack pointer. Mismatched push/pop sizes will cause incorrect returns.
- RTS does not modify the CCR — the caller's condition codes are preserved across a subroutine call and return.

---

*End of Group 5 — Control Flow*

---

## Group 6 — System

---

### TRAP — Software trap

**Description:** Generate a synchronous trap exception, transferring control to the supervisor via the exception mechanism.

**Syntax:** `TRAP #<n>`

**Sizes:** None.

**Operand classes:**
- Vector number: immediate literal 0–3 (core minimum)

**Operation:**
```
VECTOR  ← 9 + n
ECAUSE  ← 0x00000009 + n
EADDR   ← 0
enter exception sequence (document 05, section 7)
EPC     ← address of instruction following TRAP
```

TRAP is a synchronous trap — the instruction commits before the exception is taken. EPC points to the instruction after the TRAP, the natural return address after `RTE`.

**CCR:**
| C | V | Z | N | X |
|---|---|---|---|---|
| — | — | — | — | — |

CCR is saved in the exception frame and restored by RTE.

**Exceptions:**
- TRAP initiates an exception by design. Privilege Violation does not apply — TRAP is valid from both user and supervisor mode. It is the controlled mechanism for user code to enter supervisor mode.

**Encoding class:** Class 3 (SYSTEM)

**Example:**
```asm
TRAP    #0              ; syscall entry — vector 9
TRAP    #1              ; alternative syscall or ABI-defined use
```

**Notes:**
- TRAP is the only instruction available to user-mode code for controlled entry to supervisor mode.
- The convention mapping TRAP numbers to OS services is defined by the AROS ABI, not by the architecture.
- Vectors 9–12 correspond to TRAP #0–#3. Additional TRAP vectors may be added in future extensions.

---

### RTE — Return from exception

**Description:** Restore processor state from the exception frame on the supervisor stack and return to the interrupted context.

**Syntax:** `RTE`

**Privilege:** Supervisor only. Executing RTE from user mode raises Privilege Violation.

**Operation:**
```
new_SR ← mem[SSP + 0]    ; SR_old from frame
new_PC ← mem[SSP + 8]    ; PC_old from frame
SSP    ← SSP + 32        ; discard 32-byte frame
SR     ← new_SR
PC     ← new_PC
if SR.S == 0:
    A7 now references USP
```

VECTOR (offset 4), CAUSE (offset 16), reserved (offset 20), and EADDR (offset 24) are read but ignored by hardware. Full frame layout in document 05, section 8.

**CCR:** Restored from SR_old. All bits reflect saved state.

**Exceptions:**
- Privilege Violation — if executed from user mode
- Address Fault — SSP misaligned
- Access Fault — SSP points to invalid region

**Encoding class:** Class 3 (SYSTEM)

**Example:**
```asm
my_handler:
    RTE             ; return to interrupted context
```

**Notes:**
- Do not use RTS from an exception handler — stack frames differ.
- The supervisor may modify SR_old in the frame before RTE to alter privilege, interrupt mask, or trace mode of the returning context.

---

### NOP — No operation

**Description:** Perform no operation. Advances PC only.

**Syntax:** `NOP`

**Operation:** `PC ← PC + 4`

**CCR:** All preserved. **Exceptions:** None.

**Encoding class:** Class 3 (SYSTEM)

---

### STOP — Stop and wait for interrupt

**Description:** Load an immediate value into SR and halt execution until an exception or interrupt is accepted.

**Syntax:** `STOP #<imm32>`

**Privilege:** Supervisor only. Executing STOP from user mode raises Privilege Violation.

**Operation:**
```
if SR.S == 0: raise Privilege Violation
SR ← imm32
enter stopped state — no further instructions execute
on accepted exception or IRQ:
    exit stopped state
    enter exception sequence normally
```

**CCR:** All bits loaded from immediate as part of new SR.

**Exceptions:**
- Privilege Violation — if executed from user mode
- Any accepted IRQ or exception exits stopped state and is handled normally

**Encoding class:** Class 3 (SYSTEM)

**Example:**
```asm
idle_loop:
    STOP    #$00002000      ; SR ← supervisor, IRQ mask=0, T=0, CCR=0
    BRA     idle_loop       ; IRQ wakes us, handler runs, then loop
```

**Notes:**
- `STOP` loads the **full 32-bit SR**. This is intentional — partial SR loads would be inconsistent on a 32-bit SR architecture.
- If the new SR masks all pending IRQs, the processor remains stopped until a level-7 IRQ, fatal exception, or reset. This is defined behavior, not undefined.
- After exiting stopped state, the processor takes the pending exception. EPC points to the STOP instruction itself — if the handler returns via RTE, STOP executes again, creating a natural wait loop.

---

### MOVE SR — Access status register (privileged)

**Description:** Read or write the full Status Register.

**Syntax:**
```
MOVE SR, <dst>      ; read SR → destination
MOVE <src>, SR      ; write source → SR
```

**Sizes:** `.l`

**Privilege:** Supervisor only. Either form from user mode raises Privilege Violation.

**Operand classes:**
- `MOVE SR, dst`: destination is `data, mem, abs`
- `MOVE src, SR`: source is `data, imm, mem, abs`

**Operation:**
```
MOVE SR, Dn:    Dn ← ZeroExtend64(SR)
MOVE Dn, SR:    SR ← Dn[31:0]
```

**CCR:**
- `MOVE SR, dst`: unchanged
- `MOVE src, SR`: all bits replaced by source

**Encoding class:** Class 3 (SYSTEM)

**Example:**
```asm
MOVE    SR, D0
OR.L    #$0700, D0      ; set IRQ mask to level 7
MOVE    D0, SR
```

---

### MOVE CCR — Access condition codes (user accessible)

**Description:** Read or write condition code bits of SR (bits 4:0).

**Syntax:**
```
MOVE CCR, <dst>     ; read CCR → destination
MOVE <src>, CCR     ; write source[4:0] → CCR
```

**Sizes:** `.w`

**Privilege:** User and supervisor.

**Operation:**
```
MOVE CCR, Dn:   Dn ← ZeroExtend64(SR[4:0])
MOVE Dn, CCR:   SR[4:0] ← Dn[4:0]    ; SR[31:5] unchanged
```

**CCR:**
- `MOVE CCR, dst`: unchanged
- `MOVE src, CCR`: C, V, Z, N, X updated from source bits 4:0

**Encoding class:** Class 3 (SYSTEM)

**Example:**
```asm
MOVE    CCR, D0         ; save CCR
; ... operations ...
MOVE    D0, CCR         ; restore CCR
```

---

*End of Group 6 — System*

---

## Appendix A — CCR Summary Table

```
Instruction    C    V    Z    N    X    Notes
──────────────────────────────────────────────────────────────────
MOVE           0    0    *    *    —
MOVEQ          0    0    *    *    —    always quad result
LEA            —    —    —    —    —
MOVEM          —    —    —    —    —
ADD            *    *    *    *    *
SUB            *    *    *    *    *
AND            0    0    *    *    —
OR             0    0    *    *    —
EOR            0    0    *    *    —
NOT            0    0    *    *    —
NEG            *    *    *    *    *    X=C always
CMP            *    *    *    *    —    X never modified
TST            0    0    *    *    —
LSL            §    0    *    *    §
LSR            §    0    *    *    §
ASL            §    *    *    *    §
ASR            §    0    *    *    §
BRA/Bcc        —    —    —    —    —
BSR            —    —    —    —    —
JMP/JSR        —    —    —    —    —
RTS            —    —    —    —    —
TRAP           —    —    —    —    —    saved/restored via frame
RTE            R    R    R    R    R    restored from SR_old
NOP            —    —    —    —    —
STOP           *    *    *    *    *    all bits from immediate
MOVE SR,dst    —    —    —    —    —
MOVE src,SR    *    *    *    *    *    all bits replaced
MOVE CCR,dst   —    —    —    —    —
MOVE src,CCR   *    *    *    *    *    bits 4:0 only

* = updated    0 = cleared    — = preserved
R = restored from frame    § = see shift count-zero rules
```

---

*End of document 04-instruction-set.md — core minimum*

*Instructions to be added in future revisions: ROL/ROR, MUL/DIV, BTST/BSET/BCLR/BCHG, ADDX/SUBX, DBcc, SCC, EXT, SWAP, CHK, and floating-point group.*