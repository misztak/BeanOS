/*!
A library containing wrappers and helper functions for the x86_64 architecture.

*/

#![no_std]

/// Provides wrapper functions for routines that require inline assembly.
pub mod asm_wrappers;

/// Structs and utilities for page frames and other paging-related data structures.
pub mod paging;

/// ELF file structs.
pub mod elf;
