#![allow(dead_code, unused_variables)]

#![feature(panic_info_message)]

#![no_std]
#![no_main]

#[cfg(not(target_os = "none"))]
compile_error!("Wrong target selected for bootloader. Must be 'x86_64-bean_os_bootloader'.");

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};

mod log;
use log::LogMode;

mod memory;
use memory::{MemRegion, MemoryMap};

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
    // initialize the logger
    log::init(LogMode::Serial);

    let kernel_start: usize = 0x400000;
    let kernel_end = kernel_start + kernel_size - 1;

    let memory_map = {
        let start_addr = memory_map_addr as *const MemRegion;
        MemoryMap::from(start_addr, memory_map_entries)
    };

    print_memory_map(&memory_map);

    // spin forever
    x86_64::asm_wrappers::halt_loop();
}

fn print_memory_map(memory_map: &MemoryMap) {
    println!("Memory Map [{} regions]:", memory_map.data.len());
    println!("Base Address       | Length             | Type");

    for region in memory_map.data.iter() {
        let reg_type = if region.usable() { "Free Memory (1)" } else { "Reserved Memory (2)" };
        println!("0x{:016X} | 0x{:016X} | {}", region.address, region.length, reg_type);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    if let Some(&args) = _info.message() {
        println!("panic occured: {:?}", args);
    } else {
        println!("panic occured");
    }

    x86_64::asm_wrappers::halt_loop();
}
