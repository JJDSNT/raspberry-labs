.section .text._start
.global _start

_start:
    // Descobre core atual
    mrs x0, mpidr_el1
    and x0, x0, #0x3
    cbz x0, 1f

// Estaciona cores secundários
0:
    wfe
    b 0b

1:
    // Configura stack
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
    bl kernel_main

4:
    wfe
    b 4b