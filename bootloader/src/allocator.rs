use core::slice;

use x86_64::asm_wrappers::get_pml4_base_addr;
use x86_64::frame::Frame;
use x86_64::page_table::PageDir;

use crate::println;
use crate::{MemRegion, MemoryMap};

/// A rudimentary page frame allocator.
/// 
/// Implemented as a simple bump allocator. Panics if no usable memory is left.
pub struct FrameAllocator {
    memory_map: MemoryMap,
    current_region: &'static MemRegion,
    next_frame: Frame,
}

impl FrameAllocator {
    /// Creates a new allocator that starts at the specified frame.
    pub fn starting_at(start_frame: Frame, memory_map: MemoryMap) -> FrameAllocator {
        let addr = start_frame.start_addr;
        let current_region = memory_map
            .data
            .iter()
            .filter(|&region| region.usable())
            .find(|&region| addr >= region.address && addr + 4096 <= region.address + region.length)
            .expect("Tried to init allocator in invalid memory region");
        FrameAllocator {
            memory_map,
            current_region,
            next_frame: start_frame,
        }
    }

    /// Identity maps the remaining physical address space.
    ///
    /// This assumes that the first gigabyte was already identity mapped.
    /// Uses 2MiB hugepages for the mapping.
    pub fn identity_map_all(&mut self) {
        // find out how much physical memory is left
        // first GB already identity mapped in stage3.s
        let phy_start_addr = 1_u64 << 30;
        let phy_end_addr = self.memory_map.max_addr;
        let remaining_size = phy_end_addr - phy_start_addr + 1;

        let page_dir_ptr_table = {
            let ptr = unsafe { *(get_pml4_base_addr() as *const u64) } & !0xFFF_u64;
            unsafe { slice::from_raw_parts_mut(ptr as *mut u64, 512) }
        };

        let needed_pdpes = (remaining_size / 4096 / 512 / 512) as usize;

        println!(
            "Identity mapping remaing physical address space:\n\tStart: 0x{:016X}, End: 0x{:016X}\n\tSize:  0x{:016X}, Required PDPEs: {}", 
            phy_start_addr, phy_end_addr, remaining_size, needed_pdpes
        );

        // TODO: support address spaces that are not a multiple of 1GiB
        assert!((remaining_size / 4096 / 512) % 512 == 0);

        let mut table_entry =
            phy_start_addr | (PageDir::Present | PageDir::Write | PageDir::HugePage).bits();

        for i in 0..needed_pdpes {
            self.next_frame.clear();

            let page_dir_table = {
                let page_dir_table_ptr = self.next_frame.start_addr as *mut u64;
                unsafe { slice::from_raw_parts_mut(page_dir_table_ptr, 512) }
            };

            for entry in page_dir_table.iter_mut() {
                *entry = table_entry;
                table_entry += 1 << 21;
            }

            page_dir_ptr_table[i + 1] =
                self.next_frame.start_addr | (PageDir::Present | PageDir::Write).bits();

            self.increment();
        }
    }

    fn increment(&mut self) {
        self.next_frame += 1;

        // TODO: move to next usable memory region if current one is full

        assert!(
            self.next_frame.start_addr + 4095 <= self.current_region.address + self.current_region.length - 1
        );
    }
}
