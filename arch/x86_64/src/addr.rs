
/// A canonical 64-bit virtual address.
/// 
/// The upper 16 bits need to be copies of bit 47.
#[repr(transparent)]
pub struct VAddr(u64);

/// A physical 64-bit address.
/// 
/// Only the lower 52 bits can be used. The upper 12 bits must always be zero.
#[repr(transparent)]
pub struct PAddr(u64);

impl VAddr {
    #[inline]
    pub fn from(value: u64) -> VAddr {
        let upper = value >> 47;
        if upper == 0_u64 || upper == 0x1FFFF_u64 {
            VAddr(value)
        } else {
            VAddr(((value << 16) as i64 >> 16) as u64)
        }
    }

    #[inline]
    pub fn zero() -> VAddr {
        VAddr(0_u64)
    }
}

impl PAddr {
    #[inline]
    pub fn from(value: u64) -> PAddr {
        const PHYS_MASK: u64 = (1_u64 << 52) - 1;
        PAddr(value & PHYS_MASK)
    }

    #[inline]
    pub fn zero() -> PAddr {
        PAddr(0_u64)
    }
}
