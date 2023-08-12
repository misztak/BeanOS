use crate::paging::{PageSize, Page4KiB};

use core::marker::PhantomData;
use core::slice;
use core::ops::AddAssign;

/// A physical memory frame.
pub struct Frame<S: PageSize = Page4KiB> {
    pub start_addr: u64,
    size: PhantomData<S>,
}

impl<S: PageSize> Frame<S> {
    const FRAME_MASK: u64 = !(S::SIZE - 1);

    pub fn containing_address(address: u64) -> Frame<S> {
        Frame { start_addr: address & Self::FRAME_MASK, size: PhantomData }
    }

    pub fn clear(&mut self) {
        let frame_slice = {
            let ptr = self.start_addr as *mut u64;
            unsafe { slice::from_raw_parts_mut(ptr, 512) }
        };
        frame_slice.fill(0_u64);
    }
}

impl AddAssign<u64> for Frame {
    fn add_assign(&mut self, other: u64) {
        self.start_addr += other * 4096;
    }
}
