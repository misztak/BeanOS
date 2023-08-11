use crate::paging::{PageSize, Page4KiB};
use core::marker::PhantomData;

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
}
