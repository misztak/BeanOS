use core::slice;

use x86_64::frame::Frame;
use x86_64::asm_wrappers::get_pml4_base_addr;

use crate::MemoryMap;
use crate::println;

pub struct FrameAllocator {
    memory_map: MemoryMap,
    next_frame: Frame,
}

impl FrameAllocator {
    pub fn starting_at(start_frame: Frame, memory_map: MemoryMap) -> FrameAllocator {
        FrameAllocator { memory_map, next_frame: start_frame }
    }

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

        println!("Identity mapping remaing physical address space:\n\tStart: 0x{:016X}, End: 0x{:016X}\n\tSize:  0x{:016X}, Required PDPEs: {}", phy_start_addr, phy_end_addr, remaining_size, needed_pdpes);

        // TODO: support address spaces that are not a multiple of 1GB
        assert!((remaining_size / 4096 / 512) % 512 == 0);

        let mut table_entry = phy_start_addr | (0x3 | (1 << 7));

        let current_frame = &mut self.next_frame;
        for i in 0..needed_pdpes {
            current_frame.clear();
            
            let page_dir_table = {
                let page_dir_table_ptr = current_frame.start_addr as *mut u64;
                unsafe { slice::from_raw_parts_mut(page_dir_table_ptr, 512) }
            };

            for entry in page_dir_table.iter_mut() {
                *entry = table_entry;
                table_entry += 1 << 21;
            }

            page_dir_ptr_table[i + 1] = current_frame.start_addr | 0x3;
            
            *current_frame += 1;
        }
    }
}
