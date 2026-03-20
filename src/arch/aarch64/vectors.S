// src/arch/aarch64/vectors.S

.section .text.vectors, "ax"
.global __exception_vectors_start

// ExceptionContext layout:
//   x[0..30]   = 31 * 8 = 248 bytes
//   sp         = 8 bytes  -> offset 248
//   elr_el1    = 8 bytes  -> offset 256
//   spsr_el1   = 8 bytes  -> offset 264
// Total = 272 bytes (múltiplo de 16)
.set CTX_SIZE,     272

.set OFF_X0,         0
.set OFF_X2,        16
.set OFF_X4,        32
.set OFF_X6,        48
.set OFF_X8,        64
.set OFF_X10,       80
.set OFF_X12,       96
.set OFF_X14,      112
.set OFF_X16,      128
.set OFF_X18,      144
.set OFF_X20,      160
.set OFF_X22,      176
.set OFF_X24,      192
.set OFF_X26,      208
.set OFF_X28,      224
.set OFF_X30,      240
.set OFF_SP,       248
.set OFF_ELR_EL1,  256
.set OFF_SPSR_EL1, 264

// ---------------------------------------------------------------------------
// Macro: entrada de vetor (128 bytes por slot)
// ---------------------------------------------------------------------------
.macro VECTOR target
    b \target
    .space 128 - 4
.endm

// ---------------------------------------------------------------------------
// Macro: salva contexto completo na stack
// ---------------------------------------------------------------------------
.macro SAVE_CONTEXT
    sub sp, sp, #CTX_SIZE

    stp x0,  x1,  [sp, #OFF_X0]
    stp x2,  x3,  [sp, #OFF_X2]
    stp x4,  x5,  [sp, #OFF_X4]
    stp x6,  x7,  [sp, #OFF_X6]
    stp x8,  x9,  [sp, #OFF_X8]
    stp x10, x11, [sp, #OFF_X10]
    stp x12, x13, [sp, #OFF_X12]
    stp x14, x15, [sp, #OFF_X14]
    stp x16, x17, [sp, #OFF_X16]
    stp x18, x19, [sp, #OFF_X18]
    stp x20, x21, [sp, #OFF_X20]
    stp x22, x23, [sp, #OFF_X22]
    stp x24, x25, [sp, #OFF_X24]
    stp x26, x27, [sp, #OFF_X26]
    stp x28, x29, [sp, #OFF_X28]
    str x30,      [sp, #OFF_X30]

    add x0, sp, #CTX_SIZE
    str x0, [sp, #OFF_SP]

    mrs x0, ELR_EL1
    str x0, [sp, #OFF_ELR_EL1]

    mrs x0, SPSR_EL1
    str x0, [sp, #OFF_SPSR_EL1]
.endm

// ---------------------------------------------------------------------------
// Macro: restaura contexto da stack e retorna via eret
// ---------------------------------------------------------------------------
.macro RESTORE_CONTEXT
    mov x0, sp
    ldr x1, [x0, #OFF_SP]

    ldr x2, [x0, #OFF_ELR_EL1]
    msr ELR_EL1, x2

    ldr x2, [x0, #OFF_SPSR_EL1]
    msr SPSR_EL1, x2

    ldp x2,  x3,  [x0, #OFF_X2]
    ldp x4,  x5,  [x0, #OFF_X4]
    ldp x6,  x7,  [x0, #OFF_X6]
    ldp x8,  x9,  [x0, #OFF_X8]
    ldp x10, x11, [x0, #OFF_X10]
    ldp x12, x13, [x0, #OFF_X12]
    ldp x14, x15, [x0, #OFF_X14]
    ldp x16, x17, [x0, #OFF_X16]
    ldp x18, x19, [x0, #OFF_X18]
    ldp x20, x21, [x0, #OFF_X20]
    ldp x22, x23, [x0, #OFF_X22]
    ldp x24, x25, [x0, #OFF_X24]
    ldp x26, x27, [x0, #OFF_X26]
    ldp x28, x29, [x0, #OFF_X28]
    ldr x30,      [x0, #OFF_X30]

    mov sp, x1

    ldr x1, [x0, #8]
    ldr x0, [x0, #0]
.endm

// ---------------------------------------------------------------------------
// Macro: handler genérico para exceções que não fazem switch
// ---------------------------------------------------------------------------
.macro HANDLER label, rust_fn
\label:
    SAVE_CONTEXT
    mov x0, sp
    bl \rust_fn
    RESTORE_CONTEXT
    eret
.endm

// ---------------------------------------------------------------------------
// Tabela de vetores — DEVE estar alinhada a 2KB (2^11)
// Todos os handlers são definidos APÓS esta tabela.
// ---------------------------------------------------------------------------
.align 11
__exception_vectors_start:

    // Current EL with SP0
    VECTOR exc_current_el_sp0_sync
    VECTOR exc_current_el_sp0_irq
    VECTOR exc_current_el_sp0_fiq
    VECTOR exc_current_el_sp0_serror

    // Current EL with SPx
    VECTOR exc_current_el_spx_sync_svc     // sync → handler de SVC
    VECTOR exc_current_el_spx_irq
    VECTOR exc_current_el_spx_fiq
    VECTOR exc_current_el_spx_serror

    // Lower EL using AArch64
    VECTOR exc_lower_el_aarch64_sync
    VECTOR exc_lower_el_aarch64_irq
    VECTOR exc_lower_el_aarch64_fiq
    VECTOR exc_lower_el_aarch64_serror

    // Lower EL using AArch32
    VECTOR exc_lower_el_aarch32_sync
    VECTOR exc_lower_el_aarch32_irq
    VECTOR exc_lower_el_aarch32_fiq
    VECTOR exc_lower_el_aarch32_serror

// ---------------------------------------------------------------------------
// Handlers — definidos APÓS a tabela para não serem afetados pelo .align 11
// ---------------------------------------------------------------------------

HANDLER exc_current_el_sp0_sync,        rust_sync_exception_current_el_sp0
HANDLER exc_current_el_sp0_irq,         rust_irq_exception_current_el_sp0
HANDLER exc_current_el_sp0_fiq,         rust_fiq_exception_current_el_sp0
HANDLER exc_current_el_sp0_serror,      rust_serror_exception_current_el_sp0

// Handler especial para sync de EL1 SPx — detecta SVC e despacha
exc_current_el_spx_sync_svc:
    SAVE_CONTEXT

    // Lê ESR_EL1 para identificar se é SVC.
    // EC = bits [31:26]. SVC gerado de EL1 tem EC = 0x15.
    mrs x1, ESR_EL1
    lsr x2, x1, #26
    and x2, x2, #0x3F
    cmp x2, #0x15
    b.ne exc_current_el_spx_sync_not_svc

    // É SVC: extrai imediato dos bits [15:0] do ESR_EL1.
    and x0, x1, #0xFFFF   // x0 = svc_num
    mov x1, sp             // x1 = ponteiro para ExceptionContext na stack
    bl rust_svc_handler

    RESTORE_CONTEXT
    eret

exc_current_el_spx_sync_not_svc:
    // Não é SVC — trata como exceção síncrona genérica.
    mov x0, sp
    bl rust_sync_exception_current_el_spx
    RESTORE_CONTEXT
    eret

HANDLER exc_current_el_spx_irq,         rust_irq_exception_current_el_spx
HANDLER exc_current_el_spx_fiq,         rust_fiq_exception_current_el_spx
HANDLER exc_current_el_spx_serror,      rust_serror_exception_current_el_spx

HANDLER exc_lower_el_aarch64_sync,      rust_sync_exception_lower_el_aarch64
HANDLER exc_lower_el_aarch64_irq,       rust_irq_exception_lower_el_aarch64
HANDLER exc_lower_el_aarch64_fiq,       rust_fiq_exception_lower_el_aarch64
HANDLER exc_lower_el_aarch64_serror,    rust_serror_exception_lower_el_aarch64

HANDLER exc_lower_el_aarch32_sync,      rust_sync_exception_lower_el_aarch32
HANDLER exc_lower_el_aarch32_irq,       rust_irq_exception_lower_el_aarch32
HANDLER exc_lower_el_aarch32_fiq,       rust_fiq_exception_lower_el_aarch32
HANDLER exc_lower_el_aarch32_serror,    rust_serror_exception_lower_el_aarch32