use x86_64::frame::Frame;

use crate::MemoryMap;

pub struct FrameAllocator {
    memory_map: MemoryMap,
    next_frame: Frame,
}

impl FrameAllocator {
    pub fn starting_at(start_frame: Frame, memory_map: MemoryMap) -> FrameAllocator {
        FrameAllocator { memory_map, next_frame: start_frame }
    }
}
