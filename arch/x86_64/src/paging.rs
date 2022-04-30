const PAGE_SIZE: u64 = 4096;
const PAGE_MASK: u64 = !(PAGE_SIZE - 1);

pub struct Frame {
    pub start_addr: u64
}

impl Frame {
    pub fn containing_address(address: u64) -> Frame {
        Frame { start_addr: address & PAGE_MASK }
    }
}
