// src/arch/aarch64/context.S

.section .text.context_switch
.global context_switch
.type context_switch, %function

// void context_switch(old: *mut ExceptionContext, new: *const ExceptionContext)
//
// x0 = old
// x1 = new
//
// Este switch NÃO é o mecanismo central da preempção.
// Ele é apenas um shim para troca voluntária/bootstrap operando sobre
// o layout de ExceptionContext.
//
// ExceptionContext em Rust:
//   x[0]   @   0
//   ...
//   x[19]  @ 152
//   ...
//   x[30]  @ 240
//   sp     @ 248
//   elr    @ 256
//   spsr   @ 264
//
// O que este switch realmente salva/restaura:
//   - x19..x30
//   - sp
//   - elr_el1
//
// Observações:
// - x0..x18 não são preservados aqui; isso é intencional, pois esta rotina
//   é usada como troca cooperativa em fronteira de chamada.
// - spsr_el1 não participa desse caminho.
// - o retorno acontece por "br elr_el1", não por "eret".
//

.equ CTX_X19,   152
.equ CTX_X20,   160
.equ CTX_X21,   168
.equ CTX_X22,   176
.equ CTX_X23,   184
.equ CTX_X24,   192
.equ CTX_X25,   200
.equ CTX_X26,   208
.equ CTX_X27,   216
.equ CTX_X28,   224
.equ CTX_X29,   232
.equ CTX_X30,   240
.equ CTX_SP,    248
.equ CTX_ELR,   256

context_switch:
    // Preserva ponteiros old/new em registradores temporários.
    mov x9, x0
    mov x10, x1

    // Salva callee-saved no contexto antigo.
    stp x19, x20, [x9, #CTX_X19]
    stp x21, x22, [x9, #CTX_X21]
    stp x23, x24, [x9, #CTX_X23]
    stp x25, x26, [x9, #CTX_X25]
    stp x27, x28, [x9, #CTX_X27]
    stp x29, x30, [x9, #CTX_X29]

    mov x11, sp
    str x11, [x9, #CTX_SP]

    // Salva o ponto de retomada deste contexto.
    adr x11, .Lresume
    str x11, [x9, #CTX_ELR]

    // Restaura callee-saved do novo contexto.
    ldp x19, x20, [x10, #CTX_X19]
    ldp x21, x22, [x10, #CTX_X21]
    ldp x23, x24, [x10, #CTX_X23]
    ldp x25, x26, [x10, #CTX_X25]
    ldp x27, x28, [x10, #CTX_X27]
    ldp x29, x30, [x10, #CTX_X29]

    ldr x11, [x10, #CTX_SP]
    mov sp, x11

    // Salta para o PC salvo no novo contexto.
    ldr x11, [x10, #CTX_ELR]
    br x11

.Lresume:
    ret

.size context_switch, .-context_switch