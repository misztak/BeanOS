#![no_std]
#![no_main]

#![feature(pointer_is_aligned_to)]

#[cfg(not(target_os = "none"))]
compile_error!("Wrong target selected for bootloader. Must be 'x86_64-bean_os_bootloader'.");

use core::panic::PanicInfo;
use core::arch::{asm, global_asm};
use core::slice;

use x86_64::elf::ElfFile;
use x86_64::frame::Frame;
use x86_64::asm_wrappers;

mod log;
use log::LogMode;

mod memory;
use memory::{MemRegion, MemoryMap};

mod allocator;
use allocator::FrameAllocator;

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

/// Entry point for the Rust part of the bootloader.
/// 
/// Stage 3 jumps here after identity mapping the first GiB of physical memory 
/// and switching into long mode (64bit).
#[no_mangle]
unsafe extern "C" fn stage_4() -> ! {
    // set stack segment descriptor, clobbers ax
    // cannot be done earlier without breaking function calls
    asm!("xor ax, ax; mov ss, ax", out("ax") _);

    // make sure the stack is aligned to a 16-byte boundary
    asm_wrappers::align_stack_to(16);

    let memory_map_addr = core::ptr::addr_of!(_memory_map) as usize;
    let memory_map_entries = _memory_map_entries as usize;
    let kernel_size = core::ptr::addr_of!(_kernel_size) as usize;

    // sanity check to make sure the stack is aligned properly
    assert!(core::ptr::addr_of!(memory_map_addr).is_aligned_to(8));

    // move out of unsafe scope
    bootloader_start(kernel_size, memory_map_addr, memory_map_entries);
}

/// Main bootloader function.
/// 
/// Identity maps the remaining physical address space and loads the kernel ELF executable.
fn bootloader_start(kernel_size: usize, memory_map_addr: usize, memory_map_entries: usize) -> ! {
    // give the x86_64 static library a pointer to the print function
    unsafe { x86_64::PRINT = Some(log::_print); }
    
    // initialize the logger
    log::init(LogMode::Serial);

    // bootloader loads the kernel at the 4MiB mark
    let kernel_start: usize = 0x400000;
    let kernel_end = kernel_start + kernel_size - 1;
    println!("Kernel blob loaded at: [start=0x{:X}, end=0x{:X}, size={}]", kernel_start, kernel_end, kernel_size);

    let memory_map = {
        let start_addr = memory_map_addr as *const MemRegion;
        MemoryMap::from(start_addr, memory_map_entries)
    };

    println!("{}", memory_map);

    let free_frames_start_addr = (kernel_start + kernel_size + 4095) & !4095;
    println!("Start of available frame range: 0x{:X}", free_frames_start_addr);

    let mut allocator = {
        let starting_frame = Frame::containing_address(free_frames_start_addr as u64);
        FrameAllocator::starting_at(starting_frame, memory_map)
    };

    allocator.identity_map_all();

    let kernel_blob = {
        let start_addr = kernel_start as *const u8;
        unsafe { slice::from_raw_parts(start_addr, kernel_size) }
    };

    load_kernel(kernel_blob, &mut allocator).unwrap();

    // spin forever
    println!("HLT LOOP");
    asm_wrappers::halt_loop();
}

fn load_kernel(kernel_blob: &'static [u8], allocator: &mut FrameAllocator) -> Result<(), &'static str> {
    let elf = ElfFile::from(kernel_blob);
    println!("Kernel entry point: 0x{:016X}", elf.entry_point);
    
    elf.print_prog_header();

    for segment in elf.prog_headers {
        if segment.prog_type != 1 { continue; }

        if segment.filesz == 0 && segment.memsz > 0 {
            // TODO: load .bss segment
            continue;
        }

        assert!(segment.filesz == segment.memsz);

        debug_assert!(segment.align == 4096);

        // start at offset.align_down
        // go until (offset + filesz).align_up
        println!(
            "LOAD segment: mapping file from 4MiB + 0x{:X} until 0x{:X}",
            segment.offset & !4095,
            (segment.offset + segment.filesz + 4095) & !4095
        );

        let pml4_offset = (segment.vaddr >> 39) % 512;
        let pdpt_offset = (segment.vaddr >> 30) % 512;
        let pd_offset = (segment.vaddr >> 21) % 512;
        let page_offset = (segment.vaddr >> 12) % 512;
        println!(
            "PML4 Index: {}, PDPT Index: {}, PD Index: {}, Page Offset: {}",
            pml4_offset, pdpt_offset, pd_offset, page_offset
        );
    }

    Ok(())
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("BOOTLOADER PANIC: {}", _info);

    asm_wrappers::halt_loop();
}
