ENTRY(_start)

SECTIONS
{
    . = 0x80000;

    .text : ALIGN(4096) {
        KEEP(*(.text._start))
        *(.text*)
    }

    .rodata : ALIGN(4096) {
        *(.rodata*)
    }

    .data : ALIGN(4096) {
        *(.data*)
    }

    /*
     * Seção dedicada para tabelas de página MMU.
     *
     * Fica separada do BSS por duas razões:
     *   1. Garante alinhamento a 4KB sem depender de onde o BSS termina.
     *   2. Permite flush de cache seletivo (só essa região) antes de
     *      habilitar a MMU, evitando problemas de coerência.
     *
     * O zeroing desta seção é feito pelo loop de BSS do boot (inclui COMMON).
     */
    .mmu : ALIGN(4096) {
        __mmu_start = .;
        *(.mmu*)
        . = ALIGN(4096);
        __mmu_end = .;
    }

    .bss : {
        . = ALIGN(16);
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        . = ALIGN(16);
        __bss_end = .;
    }

    . = ALIGN(16);
    __stack_top = . + 0x40000; /* 256 KiB stack */

    /* Região reservada para o estado do emulador Omega (16MB) */
    /* Posicionada em 16MB — bem acima do kernel + stack */
    __emu_start = 0x1000000;
    __emu_end   = 0x2000000;
}
