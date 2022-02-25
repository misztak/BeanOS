.section .boot-stage-three, "awx"
.code32

# Stage 3 of the BIOS bootloader
# 

stage_3:

    jmp spin32

spin32:
    jmp spin32

# DATA

stage3_start: .asciz "Starting stage three..."
