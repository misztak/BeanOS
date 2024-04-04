# connect to client
target remote localhost:1234

# quit GDB without confirmation prompt
define hook-quit
    set confirm off
end

# settings
set disassembly-flavor intel
set print demangle on
set print asm-demangle on

# settings for gdb-dashboard
# https://github.com/cyrus-and/gdb-dashboard
dashboard -style syntax_highlighting 'native'
dashboard registers -style list 'rax rbx rcx rdx rsi rdi rbp rsp r8 r9 r10 r11 r12 r13 r14 r15 rip eflags cs ss ds es fs gs fs_base gs_base k_gs_base cr0 cr2 cr3 cr4 cr8 efer'

# breakpoints
#b _start
b bootloader::stage_4
b bootloader::bootloader_start
b bootloader::panic
