.section .boot-stage-three, "awx"
.code32

# Stage 3 of the BIOS bootloader
# Check if long mode is supported, enable 4-level paging, identity-map the first GB of memory,
# enable long mode and jump to stage 4 (main.rs)

stage_3:
    # set descriptors for data segments
    mov bx, 0x10
    mov ds, bx
    mov es, bx
    mov ss, bx

    mov esi, offset stage3_start_msg
    call vga_println

    #
    # Check if the processor supports Long Mode
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
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    # now use the extended functions to detect long mode
    mov eax, 0x80000001
    cpuid
    test edx, (1 << 29)     # check if LM-bit is set
    jz .no_long_mode


    # disable maskable interrupts
    cli

    # zero length IDT means that NMIs will cause triple faults
    lidt zero_length_idt


    #
    # Initialize 4-Level Paging
    #

    # zero out the page table buffer
    mov edi, offset __page_table_start
    mov ecx, offset __page_table_end
    sub ecx, edi
    shr ecx, 2              # each stosd zeros out 4 bytes
    xor eax, eax
    rep stosd               # repeat filling in with eax until ecx is zero

    # identity-map the first 1GB of the address space
    # P4
    mov eax, offset _p3
    or eax, 0x3
    mov [_p4], eax
    # P3
    mov eax, offset _p2
    or eax, 0x3
    mov [_p3], eax
    # P2
    mov eax, (0x3 | (1 << 7))   # the entries in the table point to pages instead of P1 tables (bit 7 set)
    mov ecx, 0
map_p2_table_entry:
    mov [_p2 + ecx * 8], eax    # each entry is 8 byte long
    add eax, (1 << 21)          # each P2 entry maps to a 2MB page
    inc ecx
    cmp ecx, 512
    jb map_p2_table_entry       # repeat for every entry -> 2MB * 512 = 1GB of identity-mapped memory


    #
    # Enable paging and long mode
    #

    # flush caches
    wbinvd
    mfence

    # load P4 address into CR3
    mov eax, offset _p4
    mov cr3, eax

    # set Physical Address Extension (PAE) bit in CR4
    mov eax, cr4
    or eax, (1 << 5)
    mov cr4, eax

    # set long mode bit in the EFER MSR
    # https://wiki.osdev.org/Setting_Up_Long_Mode#The_Switch_from_Protected_Mode
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8)
    wrmsr

    # enable paging in CR0
    mov eax, cr0
    or eax, (1 << 31)
    mov cr0, eax

    # load a 64-bit GDT
    lgdt gdt_64_ptr


    mov esi, offset stage3_done_msg
    call vga_println

    # finally jump to stage 4 in long mode
    push 0x08       # 64-bit CS descriptor
    mov eax, offset stage_4
    push eax
    retf            # far jump, flush instruction queue


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
stage3_done_msg: .asciz "Finished stage three"
no_cpuid_msg: .asciz "Processor does not support CPUID!"
no_long_mode_msg: .asciz "Processor does not support Long Mode!"

.align 4
zero_length_idt:
    .word 0
    .quad 0

gdt_64:
    .quad 0x0000000000000000    # null descriptor
    .quad 0x00209A0000000000    # code descriptor (rx)
    .quad 0x0000920000000000    # data descriptor (rw)

.align 4
    .word 0     # padding to make the "address of the GDT" field aligned on a 4-byte boundary

gdt_64_ptr:
    .word gdt_64_ptr - gdt_64 - 1   # 16-bit size of the GDT
    .long gdt_64                    # 32-bit base address of the GDT

vga_position: .double 0
