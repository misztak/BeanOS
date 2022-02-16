#![no_std]
#![no_main]

#[cfg(not(target_os = "none"))]
compile_error!("Wrong target for bootloader. Must be 'x86_64-bean_os_bootloader'.");

use core::panic::PanicInfo;
use core::arch::global_asm;

// load assembly files
global_asm!(include_str!("stage1.s"));


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _hello() -> ! {
    loop {}
}
