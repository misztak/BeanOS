#![no_std]
#![no_main]

#[cfg(not(target_os = "none"))]
compile_error!("Wrong target selected for bootloader. Must be 'x86_64-bean_os_bootloader'.");

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};
use core::ptr;

mod serial;
use serial::{Serial, COM1};

// load assembly files
global_asm!(include_str!("stage1.s"));
global_asm!(include_str!("stage2.s"));
global_asm!(include_str!("stage3.s"));

// linker-supplied symbols
extern "C" {
    // defined in stage2.s
    static _memory_map_entries: u16;
    // defined in kernel binary
    static _kernel_size: usize;
}

#[no_mangle]
unsafe extern "C" fn stage_4() -> ! {
    // set stack segment descriptor, clobbers ax
    asm!("xor ax, ax; mov ss, ax", out("ax") _);

    vga_println("Starting stage 4...");

    Serial::init(COM1);
    Serial::send(COM1, "Serial port initialized\n");

    // spin forever
    x86_64::asm_wrappers::halt_loop();
}

fn vga_println(string: &str) {
    #[allow(non_upper_case_globals)]
    static mut g_vga_buffer_offset: u32 = 0;

    let mut address = unsafe { (0xB8000 + g_vga_buffer_offset) as *mut u16 };
    for c in string.chars() {
        let vga_char = (0x0F00 as u16) | (c as u16);
        unsafe { 
            ptr::write(address, vga_char);
            address = address.add(1);
            g_vga_buffer_offset += 2;
        }
    }
    unsafe { g_vga_buffer_offset += 160 - g_vga_buffer_offset % 160; };
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_println("Bootloader panicked!");

    x86_64::asm_wrappers::halt_loop();
}
