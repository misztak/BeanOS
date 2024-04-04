#[macro_export]
macro_rules! read_from_packed {
    ($x:expr) => {
        {
            let unaligned = core::ptr::addr_of!($x);
            unsafe { core::ptr::read_unaligned(unaligned) }
        }
    };
}
