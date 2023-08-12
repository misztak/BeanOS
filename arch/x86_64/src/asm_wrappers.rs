use core::arch::asm;

/// Read from the specified 8-bit IO port.
#[inline]
pub fn read_io(port: u16) -> u8 {
    let value: u8;
    unsafe { asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags)); }
    value
}

/// Write to the specified 8-bit IO port.
#[inline]
pub fn write_io(port: u16, data: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") data, options(nomem, nostack, preserves_flags)); }
}

/// Get the 4KB aligned physical PML4 table address.
#[inline]
pub fn get_pml4_base_addr() -> u64 {
    let pml4_addr: u64;
    unsafe { asm!("mov {val}, cr3", val = out(reg) pml4_addr, options(nomem, nostack)) };
    pml4_addr & !0xFFF_u64
}

/// Spin forever.
#[inline]
pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("cli; hlt", options(nomem, nostack)); };
    }
}
