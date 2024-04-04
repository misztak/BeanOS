use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    pub struct PageDir: u64 {
        const Present   = 1_u64 << 0;
        const Write     = 1_u64 << 1;
        const Accessed  = 1_u64 << 5;
        const Dirty     = 1_u64 << 6;
        const HugePage  = 1_u64 << 7;
    }
}
