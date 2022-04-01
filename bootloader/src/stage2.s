.section .boot-stage-two, "awx"
.code16

# Stage 2 of the BIOS bootloader
# Load the kernel, create an e820 memory map and switch to protected mode

stage_2:
	mov si, offset stage2_start
	call rm_println

	# declare the target mode to the BIOS (0x02 - Long Mode Target Only)
	# should be done only once and before the first transition into long mode
	# save flags because CF is set if this callback is not supported (which seems to be common)
	# SeaBIOS does not implement it for example
	# https://forum.osdev.org/viewtopic.php?p=161565&sid=fef9739a304d873c5e231a41a0c86b46
	pushf
	mov ax, 0xEC00
	mov bl, 0x2				# Long Mode Target Only
	int 0x15
	popf


	#
	# Create a memory map
	# 
	# uses the INT15h, ax=0xE820 system service
	# https://wiki.osdev.org/Detecting_Memory_(x86)#BIOS_Function:_INT_0x15.2C_EAX_.3D_0xE820
	# https://wiki.osdev.org/Detecting_Memory_(x86)#Getting_an_E820_Memory_Map
	#

e820_init:
	lea di, [_memory_map]	# destination buffer for memory map
	xor ebx, ebx
	xor bp, bp				# use bp to store the entry count (doesn't count ignored regions)

	mov eax, 0xE820			# upper bits of eax should be zero, so use eax instead of ax
	mov ecx, 24				# ask for 24 bytes (instead of 20)
	mov edx, 0x0534D4150	# place magic number ("SMAP") into edx

	mov dword ptr [di + 20], 1	# needs to be set for every entry to make it compatible with ACPI

	int 0x15
	jc .int15h_failed		# system call not supported by BIOS if CF is set
	mov edx, 0x0534D4150	# int15h might trash edx
	cmp eax, edx			# on success eax has the same magic number as edx
	jne .int15h_failed
	test ebx, ebx			# ebx=0 means list only contains 1 entry, which is insufficient
	jz .int15h_failed
	jmp .e820_check_result	# already called int15h, so skip the start of the loop
.e820_loop_start:
	# reset eax and ecx after every subsequent call
	mov eax, 0xE820
	mov ecx, 24
	# set ACPI bit for every entry
	mov dword ptr [di + 20], 1

	int 0x15
	jc .e820_done			# CF set means "end of list already reached"
	mov edx, 0x0534D4150	# int15h might trash edx
.e820_check_result:
	jcxz .skip_entry		# skip any 0 length entries
	cmp cl, 20				# check if response is 20 or 24 bytes long
	jbe .no_extended_attr	# jump if only 20 bytes long
	test byte ptr [di + 20], 1	# 24 byte response, is the "ignore this data" bit clear?
	je .skip_entry
.no_extended_attr:
	mov ecx, [di + 8]		# get lower uint32_t of memory region length
	or ecx, [di + 12]		# 'or' it with the upper half to check for zero
	jz .skip_entry			# length=0 means ignore this region
	inc bp					# increment good region count
	add di, 24				# next storage spot in buffer
.skip_entry:
	test ebx, ebx			# ebx=0 means the list is complete
	jne .e820_loop_start	# continue with next invocation otherwise
.e820_done:
	mov [_memory_map_entries], bp	# save the entry count
	clc						# need to clear the CF after potential 'jc' jump


	#
	# Load the kernel
	#

load_kernel:
	# drive number
	mov dl, 0x80
	# load kernel into the transfer buffer located at 0:500h
	mov word ptr [dap_buffer_segment], 0
	mov eax, offset _kernel_buffer
	mov [dap_buffer_offset], ax

	# only 1 sector can be transferred at once
	mov word ptr [dap_num_sectors], 1

	# calc the start block index
	mov eax, offset _kernel_start_addr
	mov ebx, offset _start	# kernel_start - 0x7C00
	sub eax, ebx
	shr eax, 9				# div 9
	mov [dap_lba], eax

	# load the kernel at the 4MiB mark
	mov edi, 0x400000

	# sector count
	mov ecx, offset _kernel_size
	add ecx, 511		# align the kernel blob to 512 byte
	shr ecx, 9			# div 9

load_next_kernel_sector:
	# load the sector
	mov si, offset dap
	mov ah, 0x42
	int 0x13
	jc kernel_load_failed

	# copy sector from transfer buffer to the destination address
	push ecx
	push esi
	mov ecx, 512 / 4	# copy 4 byte at a time -> 128 iterations
	# mov esi, offset _kernel_buffer
	mov esi, offset _kernel_buffer
	# move from esi to edi ecx times, increments esi and edi
	rep movsd [edi], [esi]
	pop esi
	pop ecx

	# next sector
	mov eax, [dap_lba]
	inc eax
	mov [dap_lba], eax

	sub ecx, 1
	jnz load_next_kernel_sector


	mov si, offset stage2_done
	call rm_println


	#
	# Reenable protected mode and jump to stage 3
	#

	cli
	lgdt [gdt_pointer]

	mov eax, cr0
	or al, 1
	mov cr0, eax

	# far jump, set cs descriptor and flush instruction queue
	push 0x8
	mov eax, offset stage_3
	push eax
	retf


.int15h_failed:
	mov si, offset int15h_failed_msg
	call rm_println
	jmp spin

kernel_load_failed:
	mov si, offset kernel_load_failed_msg
	call rm_println
	jmp spin

# DATA

stage2_start: .asciz "Starting stage two..."
stage2_done: .asciz "Finished stage two"
int15h_failed_msg: .asciz "Failed to load e820 memory map"
kernel_load_failed_msg: .asciz "Failed to load the kernel"

# number of available memory regions
_memory_map_entries: .word 0
