.section .boot-stage-three, "awx"
.code32

# Stage 3 of the BIOS bootloader
# Detect and enter long mode

stage_3:
    # set descriptors for data segments
    mov bx, 0x10
    mov ds, bx
    mov es, bx
    mov ss, bx

    mov esi, offset stage3_start_msg
    call vga_println

    #
    # Detect and enter long mode (if present)
    #

    # check if CPUID is supported by flipping the ID bit (bit 21) in the FLAGS register
    # https://wiki.osdev.org/Setting_Up_Long_Mode#Detection_of_CPUID

    # copy FLAGS into eax and ecx
    pushfd
    pop eax
    mov ecx, eax

    # flip the ID bit
    xor eax, (1 << 21)

    # copy eax into FLAGS and back again
    push eax
    popfd

    pushfd
    pop eax

    # restore old value
    push ecx
    popfd

    # check if the bit was flipped
    xor eax, ecx
    jz .no_cpuid


    # detect long mode
    # https://wiki.osdev.org/Setting_Up_Long_Mode#x86_or_x86-64

    # first check if extended functions of CPUID (> 0x80000000) are available
    mov eax, 0x80000000     # A-register
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    # now use the extended functions to detect long mode
    mov eax, 0x80000001
    cpuid
    test edx, (1 << 29)     # check if LM-bit is set
    jz .no_long_mode


spin_with_halt:
    hlt
    jmp spin_with_halt

.no_cpuid:
    mov esi, offset no_cpuid_msg
    call vga_println
    jmp spin_with_halt

.no_long_mode:
    mov esi, offset no_long_mode_msg
    call vga_println
    jmp spin_with_halt


# print a string and a newline
vga_println:
    push eax
    push ebx
    push ecx
    push edx

    call vga_print

    # newline
    mov edx, 0
    mov eax, vga_position
    mov ecx, 80 * 2
    div ecx
    add eax, 1
    mul ecx
    mov vga_position, eax

    pop edx
    pop ecx
    pop ebx
    pop eax

    ret

# print a string
vga_print:
    cld
vga_print_loop:
    lodsb al, BYTE PTR [esi]
    test al, al
    jz vga_print_done
    call vga_print_char
    jmp vga_print_loop
vga_print_done:
    ret


# print a character
vga_print_char:
    mov ebx, vga_position
    mov ah, 0x0F
    mov [ebx + 0xB8000 + 1920], ax

    add ebx, 2
    mov [vga_position], ebx

    ret


# DATA

stage3_start_msg: .asciz "Starting stage three..."
no_cpuid_msg: .asciz "Processor does not support CPUID!"
no_long_mode_msg: .asciz "Processor does not support Long Mode!"

vga_position:
    .double 0
