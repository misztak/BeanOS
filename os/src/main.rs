#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr;

use x86_64::asm_wrappers::halt_loop;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_println("Hello World!");
    
    halt_loop();
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
    vga_println("Kernel panicked!");    

    halt_loop();
}
