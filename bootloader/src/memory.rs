use core::slice;
use core::fmt;

use x86_64::read_from_packed;
use crate::{print, println};

#[repr(C)]
pub struct MemRegion {
    pub address: u64,
    pub length: u64,
    pub reg_type: u32,
    pub attr: u32,
}

/// Rust representation of an e802 memory map.
pub struct MemoryMap {
    pub data: &'static [MemRegion],
    pub max_addr: u64,
}

impl MemRegion {
    pub fn usable(&self) -> bool {
        read_from_packed!(self.reg_type) == 1
    }
}

impl MemoryMap {
    pub fn from(data_ptr: *const MemRegion, len: usize) -> MemoryMap {
        let data = unsafe { slice::from_raw_parts(data_ptr, len) };

        // HACK: we ignore regions starting above 4GiB because the current allocator cannot handle too many frames
        // TODO: improve the allocator so that it can identity map large address spaces (maybe use 1GiB hugepages?)
        let max_addr = data
            .iter()
            .filter(|&region| region.address <= (1_u64 << 32))
            .map(|region| region.address + region.length - 1)
            .max()
            .expect("no regions in memory map");

        MemoryMap { data, max_addr }
    }
}

impl fmt::Display for MemoryMap {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        println!("Memory Map [{} regions]:", self.data.len());
        println!("Base Address       | Length             | Type");

        for region in self.data.iter() {
            let reg_type = if region.usable() { "Free Memory (1)" } else { "Reserved Memory (2)" };
            println!("0x{:016X} | 0x{:016X} | {}", read_from_packed!(region.address), read_from_packed!(region.length), reg_type);
        }

        print!("Max address: 0x{:016X}", self.max_addr);

        Ok(())
    }
}
