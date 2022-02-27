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
	#
	# Enable protected mode
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

	# far jump after entering protected mode to clear real mode instruction queue
	jmp protected_mode

protected_mode:	
	# set data and extra segment selectors to point at third entry in GDT
	# selected descriptors will remain cached (i.e. valid) even after 
	# segment register values change (in unreal mode) 
	mov bx, 0x10
	mov ds, bx
	mov es, bx

	# switch back to real mode because we can't call BIOS system services in protected mode
	# the cached descriptors still point to the GDT which means that we can access all 4GB of
	# addressable memory from real mode (this variant of real mode is often called unreal mode)
	# https://wiki.osdev.org/Unreal_Mode
	and al, 0xFE
	mov cr0, eax

	# we are now in unreal mode, restore saved segment register values
	# this does not influence the descriptor caches
	pop es
	pop ds

	sti

	# check if INT13h supports LBA extensions
	# https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#LBA_in_Extended_Mode
	mov ah, 0x41	# "Check Extensions Present" function
	mov bx, 0x55AA	# magic number
	mov dl, 0x80	# "drive number", in this case the "C" drive
	int 0x13
	jc int13h_extensions_not_supported

	#
	# Load the rest of the bootloader from disk
	# (this routine only works if size of remaining bootloader is a multiple of 512)
	#

	# store the current base address offset
	mov ecx, 0

load_from_disk:
	# base address of the .boot-stage-two section
	mov eax, offset _rest_of_bootloader_start_addr
	# add current offset
	add eax, ecx

	# calc buffer segment from current address
	mov ebx, eax
	shr ebx, 4		# div 16 (packet size)
	mov [dap_buffer_segment], bx

	# calc buffer segment offset from current address
	and eax, 0xF	# mod 16 (packet size)
	mov [dap_buffer_offset], ax

	# reload current address
	mov eax, offset _rest_of_bootloader_start_addr
	add eax, ecx

	# calc remaining disk sectors to load (max 127 at once)
	mov ebx, offset _rest_of_bootloader_end_addr
	sub ebx, eax	# remaining bytes
	jz load_from_disk_complete
	shr ebx, 9		# div 512 (sector size)
	cmp ebx, 127	# max sector count
	jle load_next_sectors
	mov ebx, 127
load_next_sectors:
	mov [dap_num_sectors], bx

	# increment address offset for next iteration
	shl ebx, 9		# mul 512
	add ecx, ebx

	# calc current sector number (starts at 1 because we don't want to load stage1 again)
	mov ebx, offset _start	# 0x7c00 (first sector on disk)
	sub eax, ebx	# current_address - 0x7c00
	shr eax, 9		# div 512
	mov [dap_lba], eax

	# do the actual loading
	mov si, offset dap
	mov ah, 0x42
	int 0x13		# sets carry flag on error
	jc load_from_disk_failed

	# continue loading
	jmp load_from_disk

load_from_disk_complete:

	#
	# Jump to second stage of bootloader
	#

	mov si, offset stage1_done
	call rm_println

	jmp stage_2

#
# helpers
#

# infinite loop
spin:
	jmp spin


# real mode println
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

# failure messages

int13h_extensions_not_supported:
	mov si, offset no_extended_mode_support
	call rm_println
	jmp spin

load_from_disk_failed:
	mov si, offset load_failed
	call rm_println
	jmp spin

# DATA

stage1_start: .asciz "Starting stage one..."
stage1_done: .asciz "Finished stage one"
no_extended_mode_support: .asciz "INT13h extensions not supported"
load_failed: .asciz "Failed to load rest of bootloader from disk"

gdt_pointer:
	.word gdt_end - gdt - 1		# size of table (in bytes)
	.word gdt					# base address of table

# open up all 4 GB of addressable memory
# https://en.wikipedia.org/wiki/Global_Descriptor_Table#GDT_example
gdt:
	.quad 0
codedesc:		# cs descriptor at table offset 0x08
	.byte 0xFF
	.byte 0xFF
	.byte 0
	.byte 0
	.byte 0
	.byte 0x9A
	.byte 0xCF
	.byte 0
datadesc:		# ds, ss, es, fs and gs descriptor at table offset 0x10 
	.byte 0xFF
    .byte 0xFF
    .byte 0
    .byte 0
    .byte 0
    .byte 0x92
    .byte 0xCF
    .byte 0
gdt_end:

# Disk Address Packet Structure
# https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#LBA_in_Extended_Mode
dap:
	.byte 0x10	# size of packet (16 bytes)
	.byte 0		# always 0
dap_num_sectors:
	.word 0		# number of sectors to transfer
dap_buffer_offset:
	.word 0		# transfer buffer segment offset
dap_buffer_segment:
	.word 0		# transfer buffer segment
dap_lba:
	.quad 0		# logical block address


.org 510 		# padding (one sector)
.word 0xAA55 	# BIOS magic number for bootable sector (last word)
