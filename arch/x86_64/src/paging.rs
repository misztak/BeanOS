use core::marker::PhantomData;

/// Trait used for all the available x86_64 page sizes.
pub trait PageSize {
    const SIZE: u64;
}

/// Standard 4 KiB page size.
pub enum Page4KiB {}

impl PageSize for Page4KiB {
    const SIZE: u64 = 4096;
}

/// 2 MiB hugepage.
pub enum Page2MiB {}

impl PageSize for Page2MiB {
    const SIZE: u64 = 4096 * 512;
}

/// 1 GiB hugepage.
pub enum Page1GiB {}

impl PageSize for Page1GiB {
    const SIZE: u64 = 4096 * 512 * 512;
}

/// A virtual page.
pub struct Page<S: PageSize = Page4KiB> {
    start_addr: u64,
    size: PhantomData<S>,
}

impl<S: PageSize> Page<S> {
    pub const SIZE: u64 = S::SIZE;


}
