ENTRY(_start)

SECTIONS
{
    . = 0x80000;

    .text : ALIGN(8) {
        KEEP(*(.text._start))
        *(.text*)
    }

    .rodata : ALIGN(8) {
        *(.rodata*)
    }

    .data : ALIGN(8) {
        *(.data*)
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
    __stack_top = . + 0x40000;
}