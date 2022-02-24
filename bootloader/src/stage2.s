.section .boot-stage-two, "awx"
.code16

# Stage 2 of the BIOS bootloader
# 

stage_2:
	mov si, offset stage2_start
	call rm_println

	jmp spin

# DATA

stage2_start: .asciz "Starting stage two..."
