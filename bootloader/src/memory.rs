use core::slice;

#[repr(C)]
pub struct MemRegion {
    pub address: u64,
    pub length: u64,
    pub reg_type: u32,
    pub attr: u32,
}

pub struct MemoryMap {
    pub data: &'static [MemRegion],
    pub max_addr: u64,
}

impl MemRegion {
    pub fn usable(&self) -> bool {
        self.reg_type == 1
    }
}

impl MemoryMap {
    pub fn from(data_ptr: *const MemRegion, len: usize) -> MemoryMap {
        let data = unsafe { slice::from_raw_parts(data_ptr, len) };
        let max_addr = data
            .iter()
            .map(|region| region.address + region.length - 1)
            .max()
            .expect("no regions in memory map");
        MemoryMap { data, max_addr }
    }
}
