.section .boot-stage-one, "awx"
.global _start
.code16

# Stage 1 of the BIOS bootloader
# Enable A20 line, initialize GDT and load rest of bootloader from disk

_start:
	# reset segment registers
	xor ax, ax
	mov ds, ax
	mov es, ax
	mov ss, ax
	mov fs, ax
	mov gs, ax

	# clear direction flag before using println
	cld

	# initialize stack
	mov sp, 0x7c00

	mov si, offset stage1_start
	call rm_println

	# enable A20 line (fast method)
	# https://wiki.osdev.org/A20_Line#Fast_A20_Gate
	in al, 0x92
	test al, 2
	jnz a20_enabled
	or al, 2
	and al, 0xFE
	out 0x92, al
a20_enabled:

	# enable protected mode
	#
	# clear interrupt flag
	cli
	# save real mode segment register values
	push ds
	push es

	# load GDT for 32bit mode
	lgdt [gdt_pointer]

	# set protected mode bit
	mov eax, cr0
	or al, 1
	mov cr0, eax

	jmp protected_mode	# prevents crashing on some architectures???

protected_mode:	
	# set data and extra segment
	mov bx, 0x10
	mov ds, bx
	mov es, bx

	# back to real mode
	and al, 0xFE
	mov cr0, eax

	# back in real mode, restore saved segment register values
	pop es
	pop ds

	sti

	mov si, offset stage1_done
	call rm_println

spin:
	jmp spin

# helpers

rm_println:
	# print the actual string
	call rm_print
	# print \r
	mov al, 13
	call rm_print_char
	# print \n
	mov al, 10
	jmp rm_print_char


rm_print:
	cld
rm_print_loop:
	# load next char
	lodsb al, BYTE PTR [si]
	# check if null terminator was reached
	test al, al
	jz rm_print_done
	call rm_print_char
	jmp rm_print_loop
rm_print_done:
	ret


rm_print_char:
	mov ah, 0x0E
	int 0x10
	ret


# DATA

stage1_start: .asciz "Starting stage one..."
stage1_done: .asciz "Finished stage one"

gdt_pointer:
	.word gdt_end - gdt - 1		# last byte in table
	.word gdt					# first byte in table

gdt:
	.quad 0
codedesc:
	.byte 0xFF
	.byte 0xFF
	.byte 0
	.byte 0
	.byte 0
	.byte 0x9A
	.byte 0xCF
	.byte 0
datadesc:
	.byte 0xFF
    .byte 0xFF
    .byte 0
    .byte 0
    .byte 0
    .byte 0x92
    .byte 0xCF
    .byte 0
gdt_end:

.org 510 		# padding
.word 0xAA55 	# BIOS magic number for bootable sector (last word)
