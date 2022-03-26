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

/// Spin forever.
#[inline]
pub fn halt_loop() -> ! {
    loop {
        unsafe { asm!("cli; hlt", options(nomem, nostack)); };
    }
}
