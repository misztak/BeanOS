/*!
A library containing wrappers and helper functions for the x86_64 architecture.

*/

#![no_std]

use core::fmt::Arguments;
type PrintFn = fn(Arguments);

/// Function pointer that needs to point to a print(fmt::Arguments) function in either the bootloader or kernel.
pub static mut PRINT: Option<PrintFn> = None;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let print_fn = unsafe { $crate::PRINT.expect("PRINT callback was not defined in main program") };
        print_fn(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}


/// Various utility functions.
pub mod utils;

/// Provides wrapper functions for routines that require inline assembly.
pub mod asm_wrappers;

/// Canonical virtual and physical 64-bit address types.
pub mod addr;

/// Structs and utilities for pages.
pub mod paging;

/// Page table abstractions.
pub mod page_table;

/// Abstractions for physical frames.
pub mod frame;

/// ELF file structs.
pub mod elf;
