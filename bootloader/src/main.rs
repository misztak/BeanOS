#![allow(dead_code, unused_variables)]

#![no_std]
#![no_main]

#[cfg(not(target_os = "none"))]
compile_error!("Wrong target selected for bootloader. Must be 'x86_64-bean_os_bootloader'.");

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};
use core::slice;

use x86_64::elf::ElfFile;

mod log;
use log::LogMode;

mod memory;
use memory::{MemRegion, MemoryMap};

mod allocator;

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
    // give the x86_64 static library a pointer to the print function
    unsafe { x86_64::PRINT = Some(log::_print) };
    
    // initialize the logger
    log::init(LogMode::Serial);

    let kernel_start: usize = 0x400000;
    let kernel_end = kernel_start + kernel_size - 1;
    println!("Kernel blob size: {} byte", kernel_size);

    let memory_map = {
        let start_addr = memory_map_addr as *const MemRegion;
        MemoryMap::from(start_addr, memory_map_entries)
    };

    println!("{}", memory_map);

    let kernel_blob = {
        let start_addr = kernel_start as *const u8;
        unsafe { slice::from_raw_parts(start_addr, kernel_size) }
    };

    let elf_file = ElfFile::from(kernel_blob);
    println!("Kernel entry point: 0x{:016X}", elf_file.entry_point);
    
    elf_file.print_prog_header();

    // spin forever
    println!("HLT LOOP");
    x86_64::asm_wrappers::halt_loop();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("BOOTLOADER PANIC: {}", _info);

    x86_64::asm_wrappers::halt_loop();
}
