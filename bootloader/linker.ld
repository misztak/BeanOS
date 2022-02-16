ENTRY(_start)

SECTIONS {
    . = 0x500;
    __bootloader_start = .;

    _stack_start = .;
    . = 0x7c00;
    _stack_end = .;

    .bootloader :
    {
        /* stage one*/
        *(.boot-stage-one)

        /* rest of bootloader */
        _rest_of_bootloader_start_addr = .;
        /* *(.boot) */
        /* *(.context_switch) */
        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.bss .bss.*)
        *(.got)
        . = ALIGN(512);
        _rest_of_bootloader_end_addr = .;
        __bootloader_end = .;
    }

}
