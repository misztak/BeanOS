#[repr(C)]
pub struct MemRegion {
    pub address: u64,
    pub length: u64,
    pub reg_type: u32,
    pub attr: u32,
}

impl MemRegion {
    pub fn usable(&self) -> bool {
        self.reg_type == 1
    }
}