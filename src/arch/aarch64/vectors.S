// src/arch/aarch64/vectors.S

.section .text.vectors, "ax"
.global __exception_vectors_start

// ExceptionContext layout:
//   x[0..30]   = 31 * 8  = 248 bytes
//   sp         = 8       -> offset 248
//   elr_el1    = 8       -> offset 256
//   spsr_el1   = 8       -> offset 264
//   q[0..31]   = 32 *16  = 512 bytes -> offset 272
//   fpcr       = 8       -> offset 784
//   fpsr       = 8       -> offset 792
// Total = 800 bytes (múltiplo de 16)
.set CTX_SIZE,       800

.set OFF_X0,           0
.set OFF_X2,          16
.set OFF_X4,          32
.set OFF_X6,          48
.set OFF_X8,          64
.set OFF_X10,         80
.set OFF_X12,         96
.set OFF_X14,        112
.set OFF_X16,        128
.set OFF_X18,        144
.set OFF_X20,        160
.set OFF_X22,        176
.set OFF_X24,        192
.set OFF_X26,        208
.set OFF_X28,        224
.set OFF_X30,        240
.set OFF_SP,         248
.set OFF_ELR_EL1,    256
.set OFF_SPSR_EL1,   264

.set OFF_Q0,         272
.set OFF_Q1,         288
.set OFF_Q2,         304
.set OFF_Q3,         320
.set OFF_Q4,         336
.set OFF_Q5,         352
.set OFF_Q6,         368
.set OFF_Q7,         384
.set OFF_Q8,         400
.set OFF_Q9,         416
.set OFF_Q10,        432
.set OFF_Q11,        448
.set OFF_Q12,        464
.set OFF_Q13,        480
.set OFF_Q14,        496
.set OFF_Q15,        512
.set OFF_Q16,        528
.set OFF_Q17,        544
.set OFF_Q18,        560
.set OFF_Q19,        576
.set OFF_Q20,        592
.set OFF_Q21,        608
.set OFF_Q22,        624
.set OFF_Q23,        640
.set OFF_Q24,        656
.set OFF_Q25,        672
.set OFF_Q26,        688
.set OFF_Q27,        704
.set OFF_Q28,        720
.set OFF_Q29,        736
.set OFF_Q30,        752
.set OFF_Q31,        768

.set OFF_FPCR,       784
.set OFF_FPSR,       792

.macro VECTOR target
    b \target
    .space 128 - 4
.endm

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

    str q0,  [sp, #OFF_Q0]
    str q1,  [sp, #OFF_Q1]
    str q2,  [sp, #OFF_Q2]
    str q3,  [sp, #OFF_Q3]
    str q4,  [sp, #OFF_Q4]
    str q5,  [sp, #OFF_Q5]
    str q6,  [sp, #OFF_Q6]
    str q7,  [sp, #OFF_Q7]
    str q8,  [sp, #OFF_Q8]
    str q9,  [sp, #OFF_Q9]
    str q10, [sp, #OFF_Q10]
    str q11, [sp, #OFF_Q11]
    str q12, [sp, #OFF_Q12]
    str q13, [sp, #OFF_Q13]
    str q14, [sp, #OFF_Q14]
    str q15, [sp, #OFF_Q15]
    str q16, [sp, #OFF_Q16]
    str q17, [sp, #OFF_Q17]
    str q18, [sp, #OFF_Q18]
    str q19, [sp, #OFF_Q19]
    str q20, [sp, #OFF_Q20]
    str q21, [sp, #OFF_Q21]
    str q22, [sp, #OFF_Q22]
    str q23, [sp, #OFF_Q23]
    str q24, [sp, #OFF_Q24]
    str q25, [sp, #OFF_Q25]
    str q26, [sp, #OFF_Q26]
    str q27, [sp, #OFF_Q27]
    str q28, [sp, #OFF_Q28]
    str q29, [sp, #OFF_Q29]
    str q30, [sp, #OFF_Q30]
    str q31, [sp, #OFF_Q31]

    mrs x0, fpcr
    str x0, [sp, #OFF_FPCR]

    mrs x0, fpsr
    str x0, [sp, #OFF_FPSR]
.endm

.macro RESTORE_CONTEXT
    // x19 guarda o ponteiro base do contexto até o final.
    // Não pode ser destruído no meio do restore.
    mov x19, sp

    // Estado de retorno
    ldr x20, [x19, #OFF_SP]
    ldr x21, [x19, #OFF_ELR_EL1]
    msr ELR_EL1, x21

    ldr x21, [x19, #OFF_SPSR_EL1]
    msr SPSR_EL1, x21

    // SIMD/FPU
    ldr q0,  [x19, #OFF_Q0]
    ldr q1,  [x19, #OFF_Q1]
    ldr q2,  [x19, #OFF_Q2]
    ldr q3,  [x19, #OFF_Q3]
    ldr q4,  [x19, #OFF_Q4]
    ldr q5,  [x19, #OFF_Q5]
    ldr q6,  [x19, #OFF_Q6]
    ldr q7,  [x19, #OFF_Q7]
    ldr q8,  [x19, #OFF_Q8]
    ldr q9,  [x19, #OFF_Q9]
    ldr q10, [x19, #OFF_Q10]
    ldr q11, [x19, #OFF_Q11]
    ldr q12, [x19, #OFF_Q12]
    ldr q13, [x19, #OFF_Q13]
    ldr q14, [x19, #OFF_Q14]
    ldr q15, [x19, #OFF_Q15]
    ldr q16, [x19, #OFF_Q16]
    ldr q17, [x19, #OFF_Q17]
    ldr q18, [x19, #OFF_Q18]
    ldr q19, [x19, #OFF_Q19]
    ldr q20, [x19, #OFF_Q20]
    ldr q21, [x19, #OFF_Q21]
    ldr q22, [x19, #OFF_Q22]
    ldr q23, [x19, #OFF_Q23]
    ldr q24, [x19, #OFF_Q24]
    ldr q25, [x19, #OFF_Q25]
    ldr q26, [x19, #OFF_Q26]
    ldr q27, [x19, #OFF_Q27]
    ldr q28, [x19, #OFF_Q28]
    ldr q29, [x19, #OFF_Q29]
    ldr q30, [x19, #OFF_Q30]
    ldr q31, [x19, #OFF_Q31]

    ldr x21, [x19, #OFF_FPCR]
    msr fpcr, x21

    ldr x21, [x19, #OFF_FPSR]
    msr fpsr, x21

    // GPRs
    // x19 é o ponteiro base; restaure-o por último.
    ldp x0,  x1,  [x19, #OFF_X0]
    ldp x2,  x3,  [x19, #OFF_X2]
    ldp x4,  x5,  [x19, #OFF_X4]
    ldp x6,  x7,  [x19, #OFF_X6]
    ldp x8,  x9,  [x19, #OFF_X8]
    ldp x10, x11, [x19, #OFF_X10]
    ldp x12, x13, [x19, #OFF_X12]
    ldp x14, x15, [x19, #OFF_X14]
    ldp x16, x17, [x19, #OFF_X16]
    ldr x18,      [x19, #OFF_X18]
    ldr x22,      [x19, #OFF_X18 + 8]
    ldp x23, x24, [x19, #OFF_X20 + 8]
    ldp x25, x26, [x19, #OFF_X22 + 8]
    ldp x27, x28, [x19, #OFF_X24 + 8]
    ldp x29, x30, [x19, #OFF_X28]

    // restaura x20/x21 depois de usá-los como temporários
    ldr x20, [x19, #OFF_X20]
    ldr x21, [x19, #OFF_X20 + 8]

    // restaura o x19 original por último
    ldr x19, [x19, #OFF_X18 + 8]

    // sp do contexto restaurado
    mov sp, x20
.endm

.macro HANDLER label, rust_fn
\label:
    SAVE_CONTEXT
    mov x0, sp
    bl \rust_fn
    RESTORE_CONTEXT
    eret
.endm

.align 11
__exception_vectors_start:

    VECTOR exc_current_el_sp0_sync
    VECTOR exc_current_el_sp0_irq
    VECTOR exc_current_el_sp0_fiq
    VECTOR exc_current_el_sp0_serror

    VECTOR exc_current_el_spx_sync_svc
    VECTOR exc_current_el_spx_irq
    VECTOR exc_current_el_spx_fiq
    VECTOR exc_current_el_spx_serror

    VECTOR exc_lower_el_aarch64_sync
    VECTOR exc_lower_el_aarch64_irq
    VECTOR exc_lower_el_aarch64_fiq
    VECTOR exc_lower_el_aarch64_serror

    VECTOR exc_lower_el_aarch32_sync
    VECTOR exc_lower_el_aarch32_irq
    VECTOR exc_lower_el_aarch32_fiq
    VECTOR exc_lower_el_aarch32_serror

HANDLER exc_current_el_sp0_sync,        rust_sync_exception_current_el_sp0
HANDLER exc_current_el_sp0_irq,         rust_irq_exception_current_el_sp0
HANDLER exc_current_el_sp0_fiq,         rust_fiq_exception_current_el_sp0
HANDLER exc_current_el_sp0_serror,      rust_serror_exception_current_el_sp0

exc_current_el_spx_sync_svc:
    SAVE_CONTEXT

    mrs x1, ESR_EL1
    lsr x2, x1, #26
    and x2, x2, #0x3F
    cmp x2, #0x15
    b.ne exc_current_el_spx_sync_not_svc

    and x0, x1, #0xFFFF
    mov x1, sp
    bl rust_svc_handler

    RESTORE_CONTEXT
    eret

exc_current_el_spx_sync_not_svc:
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