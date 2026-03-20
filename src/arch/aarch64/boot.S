// src/arch/aarch64/boot.S

.section .text._start
.global _start

_start:
    // Salva o ponteiro do DTB recebido em x0
    mov x20, x0

    // Descobre core atual
    mrs x0, mpidr_el1
    and x0, x0, #0x3
    cbz x0, 1f

// Estaciona cores secundários
0:
    wfe
    b 0b

1:
    // Configura stack temporária do core 0
    ldr x1, =__stack_top
    mov sp, x1

    // Limpa .bss
    ldr x2, =__bss_start
    ldr x3, =__bss_end

2:
    cmp x2, x3
    b.hs 3f
    str xzr, [x2], #8
    b 2b

3:
    // Se entrou em EL2, desce para EL1h antes de entrar no Rust.
    mrs x0, CurrentEL
    lsr x0, x0, #2
    cmp x0, #2
    b.ne 5f

    // SP_EL1 = stack já configurada
    mov x0, sp
    msr SP_EL1, x0

    // EL1 em AArch64.
    // RW=1 no HCR_EL2 para que EL1 execute em AArch64.
    mov x0, #(1 << 31)
    msr HCR_EL2, x0

    // Opcionalmente zerar CPTR_EL2 para não bloquear acesso básico a FP/SIMD depois.
    mov x0, #0
    msr CPTR_EL2, x0

    // SPSR_EL2:
    // M[3:0] = 0101 => EL1h
    // D, A, I, F = 0 (não mascarados)
    mov x0, #0x5
    msr SPSR_EL2, x0

    // Volta em EL1 para o rótulo 5
    adr x0, 5f
    msr ELR_EL2, x0

    eret

5:
    // Restaura o ponteiro do DTB em x0 para o Rust
    mov x0, x20
    bl rust_entry

6:
    wfe
    b 6b