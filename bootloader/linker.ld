/* Linker script for the bootloader */

SECTIONS {
    /* 0x000-0x4FF is reserved for the BIOS */
    . = 0x500;
    __bootloader_start = .;
    _stack_start = .;

    . = 0x7c00;
    _stack_end = .;

    /* 512-byte aligned */
    .bootloader :
    {
        /* stage one, entry point */
        /* one sector (512 bytes) */
        *(.boot-stage-one)

        /* remaining stages of the bootloader */
        _rest_of_bootloader_start_addr = .;

        *(.boot-stage-two)

        *(.text .text.*)
        *(.rodata .rodata.*)
        *(.data .data.*)
        *(.bss .bss.*)
        *(.got)

        /* stage 1 loads rest of bootloader in 512-byte sized blocks */
        /* so make sure we pad the bootloader section accordingly */
        /* this ensures that the size of the 'rest_of_bootloader' sections is a multiple of 512 */
        . = ALIGN(512);

        _rest_of_bootloader_end_addr = .;
        __bootloader_end = .;
    }
}
