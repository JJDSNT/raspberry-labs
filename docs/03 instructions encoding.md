# MC68k-64 Architecture Specification
## Document 03 — Instruction Encoding

**Status:** Draft  
**Version:** 0.1  

---

## 1. Overview

All MC68k-64 instructions are encoded in a **32-bit base word**, optionally followed by one or two 32-bit **extension words**. The number of extension words is always determined by the base word alone — the decoder never needs to speculatively fetch additional words.

The assembly syntax follows the 68k convention (`MNEMONIC.SIZE SRC, DST`) regardless of the binary encoding. The assembler is responsible for selecting the correct binary form; the programmer sees only the familiar 68k-style notation.

---

## 2. Base Word Envelope

Every instruction begins with a 32-bit base word with the following fixed fields:

```
 31 30  29 28 27 26  25 24  23                    0
┌──────┬────────────┬──────┬────────────────────────┐
│ EXT  │   CLASS    │ SIZE │        PAYLOAD         │
└──────┴────────────┴──────┴────────────────────────┘
  2 b       4 b       2 b           24 b
```

### 2.1 EXT — Extension Word Count (bits 31:30)

Declares how many 32-bit extension words follow the base word.

| EXT | Meaning | Total instruction size |
|-----|---------|----------------------|
| 00 | No extension | 32 bits (4 bytes) |
| 01 | One extension word | 64 bits (8 bytes) |
| 10 | Two extension words | 96 bits (12 bytes) |
| 11 | Reserved | — |

The extension word count is always available after reading the first 32 bits. The instruction stream remains fully predictable: the address of the next instruction is always `PC + 4 + (EXT × 4)`.

### 2.2 CLASS — Instruction Class (bits 29:26)

Selects the instruction family and determines how the 24-bit PAYLOAD is interpreted.

| CLASS | Value | Family |
|-------|-------|--------|
| ALU | 0000 | Integer dyadic: ADD, SUB, AND, OR, XOR, CMP |
| MOVE | 0001 | Data movement and unary: MOVE, LEA, CLR, NEG, NOT, SWAP, EXT |
| BRANCH | 0010 | Control flow: Bcc, BRA, BSR, JMP, JSR, RTS |
| SYSTEM | 0011 | Privileged and system: TRAP, RTE, MOVE SR, MOVE CSR, NOP, STOP |
| IMMED | 0100 | Immediate / literal forms: MOVEQ, ADDI, CMPI, SUBI |
| SHIFT | 0101 | Shifts, multiplies, divides, bit operations |
| FLOAT | 0110 | Floating-point operations |
| — | 0111 | Reserved (absorbed into ALU — see section 5.1) |
| — | 1000–1111 | Reserved for future use |

### 2.3 SIZE — Operand Size (bits 25:24)

Specifies the data size for the operation. Not all classes use this field; classes that do not use it treat it as reserved.

| SIZE | Value | Width |
|------|-------|-------|
| Byte | 00 | 8-bit |
| Word | 01 | 16-bit |
| Long | 10 | 32-bit |
| Quad | 11 | 64-bit |

### 2.4 PAYLOAD (bits 23:0)

24 bits whose interpretation is class-specific. Defined per class in section 5.

---

## 3. Effective Address (EA) Encoding

EA fields appear inside the PAYLOAD of most instruction classes. Each EA is encoded as a 3-bit MODE and a 4-bit REG field (7 bits total).

### 3.1 EA Modes

| MODE | Value | Syntax | Description |
|------|-------|--------|-------------|
| Dn | 000 | `Dn` | Data register direct |
| An | 001 | `An` | Address register direct |
| Ind | 010 | `(An)` | Address register indirect |
| Post | 011 | `(An)+` | Indirect with post-increment |
| Pre | 100 | `-(An)` | Indirect with pre-decrement |
| Disp | 101 | `d(An)` | Indirect with displacement |
| Idx | 110 | `d(An,Xn)` | Indirect indexed |
| Spec | 111 | — | Special (see 3.2) |

Post-increment and pre-decrement adjust the address register by the operand size in bytes (1, 2, 4, or 8 per SIZE).

### 3.2 Special EA (MODE=111)

When MODE=111, the REG field selects a special addressing form:

| REG | Value | Syntax | Description |
|-----|-------|--------|-------------|
| Imm | 000 | `#n` | Immediate — value in extension word |
| AbsW | 001 | `(addr).w` | Absolute 32-bit address in extension word |
| AbsL | 010 | `(addr).l` | Absolute 64-bit address in two extension words |
| PCRel | 011 | `d(PC)` | PC-relative displacement in extension word |
| — | 100–111 | — | Reserved |

### 3.3 Indexed EA (MODE=110)

When MODE=110, an extension word carries the full indexed descriptor:

```
 31      28 27   24 23 22 21 20         0
┌──────────┬───────┬──┬──┬──┬───────────┐
│ IDX_REG  │  res  │IS│SC│  │  DISP20   │
└──────────┴───────┴──┴──┴──┴───────────┘
```

| Field | Bits | Description |
|-------|------|-------------|
| IDX_REG | 31:28 | Index register (0–15, selects Dn or An) |
| IS | 23 | Index size: 0 = word (sign-extended), 1 = quad |
| SC | 22:21 | Scale: 00=×1, 01=×2, 10=×4, 11=×8 |
| DISP20 | 19:0 | Signed 20-bit displacement |

### 3.4 EA Symmetry

Source and destination EAs use identical mode encodings. Any mode valid for a source is valid for a destination, subject to the following exceptions:

- Immediate (`Spec/Imm`) is valid for source only
- PC-relative (`Spec/PCRel`) is valid for source only
- `An` direct is not valid as a destination for byte-size operations

---

## 4. Extension Words

Extension words follow the base word in order. Their meaning is determined by the CLASS, OP, and EA special flags of the base word.

### Extension word usage by context

| Context | Ext 1 | Ext 2 |
|---------|-------|-------|
| EA Spec/Imm, SIZE=b/w/l | 32-bit immediate (sign/zero-extended) | — |
| EA Spec/Imm, SIZE=q | Upper 32 bits of 64-bit immediate | Lower 32 bits |
| EA Spec/AbsW | 32-bit absolute address | — |
| EA Spec/AbsL | Upper 32 bits of address | Lower 32 bits |
| EA Spec/PCRel | 32-bit signed displacement | — |
| EA Idx (MODE=110) | Indexed descriptor (see 3.3) | — |
| Branch DISP32 | 32-bit signed displacement | — |

Extension words are always big-endian 32-bit values.

---

## 5. Instruction Class Formats

### 5.1 CLASS 0000 — ALU (Integer Dyadic)

Used for: `ADD`, `SUB`, `AND`, `OR`, `XOR`, `CMP`, `TST`

`CMP` and `TST` are encoded as ALU variants with writeback disabled (FLAGS.W=0). There is no separate class for compare/test operations.

```
 31 30  29   26  25 24  23  20  19 17  16  13  12 10  9   6  5      0
┌──────┬───────┬──────┬──────┬───────┬───────┬───────┬───────┬────────┐
│ EXT  │ 0000  │ SIZE │  OP  │SRC_MOD│SRC_REG│DST_MOD│DST_REG│  FLAGS │
└──────┴───────┴──────┴──────┴───────┴───────┴───────┴───────┴────────┘
  2       4      2      4       3       4       3       4        6
```

| Field | Bits | Description |
|-------|------|-------------|
| EXT | 31:30 | Extension count |
| CLASS | 29:26 | 0000 |
| SIZE | 25:24 | Operand size |
| OP | 23:20 | Operation code (ADD=0000, SUB=0001, AND=0010, OR=0011, XOR=0100, CMP=0101, ...) |
| SRC_MODE | 19:17 | Source EA mode |
| SRC_REG | 16:13 | Source register |
| DST_MODE | 12:10 | Destination EA mode |
| DST_REG | 9:6 | Destination register |
| FLAGS | 5:0 | Modifier flags (see below) |

**FLAGS field (CLASS 0000):**

| Bit | Name | Description |
|-----|------|-------------|
| 5 | W | Writeback: 1 = write result to destination, 0 = discard (CMP/TST mode) |
| 4 | U | Unsigned: affects overflow and carry semantics |
| 3 | F | Update CCR: 1 = update XNZVC, 0 = preserve CCR |
| 2:0 | — | Reserved |

Default behavior (W=1, F=1) writes result and updates CCR. Setting W=0 produces a compare. Setting F=0 suppresses CCR update.

---

### 5.2 CLASS 0001 — MOVE / Unary

Used for: `MOVE`, `LEA`, `CLR`, `NEG`, `NOT`, `SWAP`, `EXT`, `MOVEM`

Format is identical to CLASS 0000. The OP field selects the specific operation. Unary operations use SRC as the operand and DST as the destination; the other EA field encodes the operation variant.

---

### 5.3 CLASS 0010 — Branch / Control Flow

Used for: `Bcc`, `BRA`, `BSR`, `JMP`, `JSR`, `RTS`, `DBcc`

```
 31 30  29   26  25  24  23  20  19  16  15               0
┌──────┬───────┬────┬────┬───────┬───────┬─────────────────┐
│ EXT  │ 0010  │LINK│ rs │COND/OP│ REG   │   DISP16        │
└──────┴───────┴────┴────┴───────┴───────┴─────────────────┘
  2       4      1    1     4       4           16
```

| Field | Bits | Description |
|-------|------|-------------|
| EXT | 31:30 | 00=short branch (DISP16), 01=long branch (DISP32 in ext1) |
| CLASS | 29:26 | 0010 |
| LINK | 25 | 1 = save return address (BSR/JSR behavior) |
| rs | 24 | Reserved |
| COND/OP | 23:20 | Condition code or operation selector |
| REG | 19:16 | Base register (for indirect jumps) or DBcc counter register |
| DISP16 | 15:0 | Signed 16-bit displacement (PC-relative). If EXT=01, ignored; displacement comes from extension word |

**COND codes** (same as 68k):

| Code | Value | Condition |
|------|-------|-----------|
| T | 0000 | True (always) — BRA |
| F | 0001 | False (never) |
| HI | 0010 | Higher (C=0 and Z=0) |
| LS | 0011 | Lower or same |
| CC | 0100 | Carry clear |
| CS | 0101 | Carry set |
| NE | 0110 | Not equal |
| EQ | 0111 | Equal |
| VC | 1000 | Overflow clear |
| VS | 1001 | Overflow set |
| PL | 1010 | Plus |
| MI | 1011 | Minus |
| GE | 1100 | Greater or equal (signed) |
| LT | 1101 | Less than (signed) |
| GT | 1110 | Greater than (signed) |
| LE | 1111 | Less or equal (signed) |

---

### 5.4 CLASS 0011 — System / Privileged

Used for: `TRAP`, `RTE`, `NOP`, `STOP`, `RESET`, `MOVE SR`, `MOVE CSR`

```
 31 30  29   26  25 24  23  20  19  16  15  13  12   9  8       0
┌──────┬───────┬──────┬───────┬───────┬───────┬───────┬──────────┐
│ EXT  │ 0011  │  rs  │  OP   │SYSREG │EA_MODE│ EA_REG│ IMM/SUB  │
└──────┴───────┴──────┴───────┴───────┴───────┴───────┴──────────┘
  2       4      2      4        4       3        4        9
```

SYSREG encodes which control register is accessed:

| SYSREG | Value | Register |
|--------|-------|----------|
| SR | 0000 | Status Register |
| USP | 0001 | User Stack Pointer |
| SSP | 0010 | Supervisor Stack Pointer |
| VBR | 0011 | Vector Base Register |
| EPC | 0100 | Exception PC |
| ECAUSE | 0101 | Exception Cause |
| EADDR | 0110 | Exception Address |
| FCSR | 0111 | FP Control/Status |
| HARTID | 1000 | Hardware Thread ID (future) |
| PTBR | 1001 | Page Table Base (future) |
| ASID | 1010 | Address Space ID (future) |
| — | 1011–1111 | Reserved |

---

### 5.5 CLASS 0100 — Immediate / Literal

Used for: `MOVEQ`, load-immediate, `ADDI`, `SUBI`, `CMPI`

```
 31 30  29   26  25 24  23  20  19  16  15               0
┌──────┬───────┬──────┬───────┬───────┬─────────────────┐
│ EXT  │ 0100  │ SIZE │  OP   │DST_REG│     IMM16        │
└──────┴───────┴──────┴───────┴───────┴─────────────────┘
  2       4      2      4        4           16
```

Immediate value rules:
- EXT=00: IMM16 is sign-extended or zero-extended to SIZE
- EXT=01: 32-bit immediate in extension word 1
- EXT=10: 64-bit immediate — extension word 1 = high 32 bits, extension word 2 = low 32 bits

---

### 5.6 CLASS 0101 — Shift / Multiply / Divide / Bit Operations

Used for: `LSL`, `LSR`, `ASL`, `ASR`, `ROL`, `ROR`, `MULS`, `MULU`, `DIVS`, `DIVU`, `BTST`, `BSET`, `BCLR`, `BCHG`

Uses the same 32-bit format as CLASS 0000 (ALU). The OP and FLAGS fields encode the specific operation and shift-count source (register or immediate count in FLAGS bits).

---

### 5.7 CLASS 0110 — Floating Point

Used for: `FMOVE`, `FADD`, `FSUB`, `FMUL`, `FDIV`, `FCMP`, `FSQRT`, FP conversions

```
 31 30  29   26  25 24  23  20  19  16  15  12  11   8  7      0
┌──────┬───────┬──────┬───────┬───────┬───────┬───────┬────────┐
│ EXT  │ 0110  │FSIZE │  OP   │SRC_FP │SRC_MOD│DST_FP │  SUB   │
└──────┴───────┴──────┴───────┴───────┴───────┴───────┴────────┘
  2       4      2      4        4       4        4        8
```

FSIZE encodes the memory format of FP operands (not the internal format, which is always double):

| FSIZE | Value | Format |
|-------|-------|--------|
| Single | 00 | 32-bit IEEE 754 |
| Double | 01 | 64-bit IEEE 754 |
| rs | 10 | Reserved |
| rs | 11 | Reserved |

Two sub-forms exist within CLASS 0110:

- **FP reg/reg** (SRC_MOD=0): both operands are FP registers
- **FP reg/EA** (SRC_MOD=1): source is a memory EA encoded in the SRC_FP field; destination is an FP register in DST_FP

---

## 6. Encoding Examples

### Example 1: `ADD.L D0, D1`

```
EXT=00  CLASS=0000  SIZE=10  OP=0000  SRC_MODE=000  SRC_REG=0000
DST_MODE=000  DST_REG=0001  FLAGS=001000
```

Binary: `00 0000 10 0000 000 0000 000 0001 001000`  
Single 32-bit word. No extensions.

---

### Example 2: `MOVE.Q #$1234_5678_9ABC_DEF0, D3`

```
EXT=10  CLASS=0100  SIZE=11  OP=MOVEQ  DST_REG=0011  IMM16=don't care
Extension 1 = 0x12345678 (high 32 bits)
Extension 2 = 0x9ABCDEF0 (low 32 bits)
```

Three 32-bit words total (96 bits).

---

### Example 3: `MOVE.L D0, 16(A1)`

```
EXT=01  CLASS=0001  SIZE=10  OP=MOVE  SRC_MODE=000  SRC_REG=0000
DST_MODE=101  DST_REG=1001
Extension 1 = 0x00000010 (displacement = 16)
```

Two 32-bit words (64 bits).

---

### Example 4: `BEQ.L label` (32-bit displacement)

```
EXT=01  CLASS=0010  LINK=0  COND=0111 (EQ)  REG=0000  DISP16=0
Extension 1 = signed 32-bit displacement from PC+8
```

Two 32-bit words (64 bits).

---

### Example 5: `MOVE.L SR, D0`

```
EXT=00  CLASS=0011  OP=MOVE_FROM_SR  SYSREG=0000 (SR)
EA_MODE=000  EA_REG=0000 (D0)
```

Single 32-bit word.

---

## 7. Decoder Algorithm

A conforming decoder proceeds as follows:

```
1. Fetch 32-bit base word from PC
2. Extract EXT[31:30] — determines extension count (0, 1, or 2)
3. Extract CLASS[29:26] — selects payload interpretation
4. Extract SIZE[25:24]
5. Decode PAYLOAD[23:0] per CLASS rules
6. If EXT >= 1: fetch extension word 1 from PC+4
7. If EXT == 2: fetch extension word 2 from PC+8
8. Advance PC by 4 + (EXT × 4)
```

The decoder never needs to inspect extension words to determine the instruction length. Step 2 always provides the complete information needed to fetch the right number of words.

---

## 8. Decisions Deferred

| Topic | Status | Document |
|-------|--------|----------|
| Complete OP code tables per class | Deferred | 04-instruction-set.md |
| Full condition code effect table per instruction | Deferred | 04-instruction-set.md |
| TRAP vector numbering | Deferred | 05-exception-model.md |
| MOVEM encoding detail | Deferred | 04-instruction-set.md |
| DBcc encoding detail | Deferred | 04-instruction-set.md |

---

*End of document 03-instruction-encoding.md*