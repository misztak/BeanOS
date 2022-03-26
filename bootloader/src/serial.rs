use x86_64::asm_wrappers::{read_io, write_io};

pub const COM1: u16 = 0x3F8;

pub struct Serial;

impl Serial {
    pub fn init(com_port: u16) {
        write_io(com_port + 1, 0x00);   // disable all interrupts
        write_io(com_port + 3, 0x80);   // enable DLAB
        write_io(com_port + 0, 0x03);   // set divisor to 3 (lo byte) (38400 baud)
        write_io(com_port + 1, 0x00);   //                  (hi byte)
        write_io(com_port + 3, 0x03);   // 8 bits, no parity, one stop bit
        write_io(com_port + 2, 0xC7);   // enable and clear FIFOs, 14 bytes
        write_io(com_port + 4, 0x0B);   // set OUT2/RTS/DSR
    }

    pub fn send(com_port: u16, msg: &str) {
        for c in msg.chars() {
            Serial::send_byte(com_port, c as u8);
        }
    }

    fn send_byte(com_port: u16, data: u8) {
        fn is_transmit_empty(com_port: u16) -> bool {
            read_io(com_port + 5) & 0x20 != 0
        }

        while !is_transmit_empty(com_port) {}

        write_io(com_port, data);
    }
}
