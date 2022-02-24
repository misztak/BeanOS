target remote localhost:1234
set disassembly-flavor intel

layout asm

b *0x7c00
b *0x7c45
