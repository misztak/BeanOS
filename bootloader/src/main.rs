// TODO: remove this
#![allow(dead_code, unused_variables)]

#![feature(panic_info_message)]

#![no_std]
#![no_main]

#[cfg(not(target_os = "none"))]
compile_error!("Wrong target selected for bootloader. Must be 'x86_64-bean_os_bootloader'.");

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};
use core::ptr;
use core::slice;

mod log;

mod mem_region;
use mem_region::MemRegion;

// load assembly files
global_asm!(include_str!("stage1.s"));
global_asm!(include_str!("stage2.s"));
global_asm!(include_str!("stage3.s"));

// linker-supplied symbols
extern "C" {
    // defined in stage2.s
    static _memory_map_entries: u16;

    // defined in linker script
    static _memory_map: usize;

    // defined in kernel binary
    static _kernel_size: usize;
}

#[no_mangle]
unsafe extern "C" fn stage_4() -> ! {
    // set stack segment descriptor, clobbers ax
    // cannot be done earlier without breaking function calls
    asm!("xor ax, ax; mov ss, ax", out("ax") _);

    let memory_map_addr = &_memory_map as *const _ as usize;
    let memory_map_entries = _memory_map_entries as usize;
    let kernel_size = &_kernel_size as *const _ as usize;

    // move out of unsafe scope
    bootloader_start(kernel_size, memory_map_addr, memory_map_entries);
}

fn bootloader_start(kernel_size: usize, memory_map_addr: usize, memory_map_entries: usize) -> ! {
    // initialize the serial port logger
    log::init();
    log::println(format_args!("Serial port logger initialized"));

    let mem_regions = {
        let start_addr = memory_map_addr as *const MemRegion;
        unsafe { slice::from_raw_parts(start_addr, memory_map_entries) }
    };

    print_memory_map(mem_regions);

    // spin forever
    x86_64::asm_wrappers::halt_loop();
}

fn print_memory_map(mem_regions: &[MemRegion]) {
    log::println(format_args!("Memory Map [{} regions]:", mem_regions.len()));
    log::println(format_args!("Base Address       | Length             | Type"));

    for region in mem_regions.iter() {
        let reg_type = if region.usable() { "Free Memory (1)" } else { "Reserved Memory (2)" };
        log::println(format_args!("0x{:016X} | 0x{:016X} | {}", region.address, region.length, reg_type));
    }
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

    if let Some(&args) = _info.message() {
        log::println(format_args!("panic occured: {:?}", args));
    } else {
        log::println(format_args!("panic occured"));
    }

    x86_64::asm_wrappers::halt_loop();
}
