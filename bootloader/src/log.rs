/*!
Print messages through the serial port (COM1) or the VGA buffer.

Uses a fixed-size buffer for formatting.
*/

use core::fmt;

use x86_64::asm_wrappers::{read_io, write_io};

#[allow(unused)]
#[derive(PartialEq, Clone, Copy)]
pub enum LogMode {
    None,
    Serial,
    VGA,
    Both,
}

pub static mut LOG_MODE: LogMode = LogMode::None;

const COM1: u16 = 0x3F8;

static mut VGA_BUFFER_OFFSET: u32 = 0;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::log::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    let mut buffer = [0u8; 256];

    let mut writer = FmtBuffer::new(&mut buffer);
    fmt::write(&mut writer, args).expect("Format buffer is too small");

    let string = writer.as_str().unwrap();
    match get_log_mode() {
        LogMode::None => (),
        LogMode::VGA => vga_print(string),
        LogMode::Serial => serial_print(string),
        LogMode::Both => {
            vga_print(string);
            serial_print(string);
        }
    }
}

pub fn init(log_mode: LogMode) {
    set_log_mode(log_mode);

    // init serial port
    write_io(COM1 + 1, 0x00);   // disable all interrupts
    write_io(COM1 + 3, 0x80);   // enable DLAB
    write_io(COM1 + 0, 0x03);   // set divisor to 3 (lo byte) (38400 baud)
    write_io(COM1 + 1, 0x00);   //                  (hi byte)
    write_io(COM1 + 3, 0x03);   // 8 bits, no parity, one stop bit
    write_io(COM1 + 2, 0xC7);   // enable and clear FIFOs, 14 bytes
    write_io(COM1 + 4, 0x0B);   // set OUT2/RTS/DSR

    // 'init' vga screen
    vga_clear_screen();

    println!("Logger initialized");
}

pub fn set_log_mode(log_mode: LogMode) {
    unsafe { LOG_MODE = log_mode; };
}

pub fn get_log_mode() -> LogMode {
    unsafe { LOG_MODE }
}

fn vga_print(string: &str) {
    let mut address = unsafe { (0xB8000 + VGA_BUFFER_OFFSET) as *mut u16 };
    for c in string.chars() {
        if c == '\n' {
            unsafe { VGA_BUFFER_OFFSET += 160 - VGA_BUFFER_OFFSET % 160; };
        } else {
            let vga_char = (0x0F00 as u16) | (c as u16);
            unsafe { 
                *address = vga_char;
                address = address.add(1);
                VGA_BUFFER_OFFSET += 2;
            }
        }
    }
}

fn vga_clear_screen() {
    let mut address = 0xB8000 as *mut u8;

    for _ in 0..(160 * 40) as usize {
        unsafe {
            *address = 0;
            address = address.add(1);
        }
    }
}

fn serial_print(string: &str) {
    fn is_transmit_empty() -> bool {
        read_io(COM1 + 5) & 0x20 != 0
    }

    for c in string.chars() {
        while !is_transmit_empty() {}

        write_io(COM1, c as u8);
    }
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
