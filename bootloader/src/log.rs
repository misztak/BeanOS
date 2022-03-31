/*!
Print messages through the serial port (COM1).

Uses a fixed-size buffer for formatting.
*/

use core::fmt;

use x86_64::asm_wrappers::{read_io, write_io};

const COM1: u16 = 0x3F8;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::log::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Initialize the serial port (COM1)
pub fn init() {
    write_io(COM1 + 1, 0x00);   // disable all interrupts
    write_io(COM1 + 3, 0x80);   // enable DLAB
    write_io(COM1 + 0, 0x03);   // set divisor to 3 (lo byte) (38400 baud)
    write_io(COM1 + 1, 0x00);   //                  (hi byte)
    write_io(COM1 + 3, 0x03);   // 8 bits, no parity, one stop bit
    write_io(COM1 + 2, 0xC7);   // enable and clear FIFOs, 14 bytes
    write_io(COM1 + 4, 0x0B);   // set OUT2/RTS/DSR
}

pub fn _print(args: fmt::Arguments) {
    let mut buffer = [0u8; 128];

    let mut writer = FmtBuffer::new(&mut buffer);
    fmt::write(&mut writer, args).expect("Failed to format the string");

    if let Some(s) = writer.as_str() {
        for char in s.chars() {
            send_byte(char as u8);
        }
    }
}

fn send_byte(data: u8) {
    fn is_transmit_empty() -> bool {
        read_io(COM1 + 5) & 0x20 != 0
    }

    while !is_transmit_empty() {}

    write_io(COM1, data);
}

struct FmtBuffer<'a> {
    buffer: &'a mut [u8],
    used: usize,
}

impl<'a> FmtBuffer<'a> {
    fn new(buffer: &'a mut [u8]) -> Self {
        FmtBuffer { buffer, used: 0 }
    }

    fn as_str(self) -> Option<&'a str> {
        if self.used <= self.buffer.len() {
            use core::str::from_utf8_unchecked;
            Some(unsafe { from_utf8_unchecked(&self.buffer[..self.used]) })
        } else {
            None
        }
    }
}

impl<'a> fmt::Write for FmtBuffer<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining_buf = &mut self.buffer[self.used..];
        let str_bytes = s.as_bytes();
        let write_len = core::cmp::min(str_bytes.len(), remaining_buf.len());

        if str_bytes.len() > write_len {
            return Err(fmt::Error);
        }

        remaining_buf[..write_len].copy_from_slice(&str_bytes[..write_len]);
        self.used += str_bytes.len();

        Ok(())
    }
}
