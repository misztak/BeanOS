use core::slice;

use crate::read_from_packed;

#[repr(C, packed)]
pub struct ProgramHeader {
    pub prog_type: u32,
    pub flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

#[repr(C, packed)]
pub struct SectionHeader {
    pub name: u32,
    pub sect_type: u32,
    pub flags: u64,
    pub addr: u64,
    pub offset: u64,
    pub size: u64,
    pub link: u32,
    pub info: u32,
    pub addralign: u64,
    pub entsize: u64,
}

pub struct ElfFile {
    pub bytes: &'static [u8],
    pub entry_point: u64,
    pub prog_headers: &'static [ProgramHeader],
    pub sect_headers: &'static [SectionHeader],
}

impl ElfFile {
    pub fn from(bytes: &'static [u8]) -> ElfFile {
        let header = &bytes[..64];

        // sanity checks
        assert_eq!(header[..0x4], [0x7F_u8, 0x45, 0x4C, 0x46], "Wrong magic number in ELF header");
        assert_eq!(header[0x4], 2, "64-bit ELF file required");
        assert_eq!(header[0x5], 1, "ELF file needs to be little endian");
        assert_eq!(header[0x7], 0, "Invalid target ABI");
        assert_eq!(header[0x10..0x12], [0x02_u8, 0x00], "ELF file needs to be an executable");

        debug_assert_eq!(56, core::mem::size_of::<ProgramHeader>());
        debug_assert_eq!(64, core::mem::size_of::<SectionHeader>());

        let entry_point = u64::from_le_bytes(header[0x18..0x20].try_into().unwrap());

        let prog_headers = {
            let prog_headers_offset = u64::from_le_bytes(header[0x20..0x28].try_into().unwrap()) as usize;
            let prog_headers_num = u16::from_le_bytes(header[0x38..0x3A].try_into().unwrap()) as usize;

            unsafe {
                let prog_headers_ptr = 
                    bytes.as_ptr().add(prog_headers_offset) as *const ProgramHeader;
                slice::from_raw_parts(prog_headers_ptr, prog_headers_num)
            }
        };

        let sect_headers = {
            let sect_headers_offset = u64::from_le_bytes(header[0x28..0x30].try_into().unwrap()) as usize;
            let sect_headers_num = u16::from_le_bytes(header[0x3C..0x3E].try_into().unwrap()) as usize;

            unsafe {
                let sect_headers_ptr = 
                    bytes.as_ptr().add(sect_headers_offset) as *const SectionHeader;
                slice::from_raw_parts(sect_headers_ptr, sect_headers_num)
            }
        };

        ElfFile { bytes, entry_point, prog_headers, sect_headers }
    }

    pub fn print_prog_header(&self) {
        println!("Program Headers:");
        println!("  Type           Offset             VirtAddr           PhysAddr");
        println!("                 FileSiz            MemSiz              Flags  Align");

        for header in self.prog_headers {
            println!("  {:14} 0x{:016X} 0x{:016X} 0x{:016X}", read_from_packed!(header.prog_type), read_from_packed!(header.offset), read_from_packed!(header.vaddr), read_from_packed!(header.paddr));
            println!("  {:14} 0x{:016X} 0x{:016X}  {}  0x{:X}", "", read_from_packed!(header.filesz), read_from_packed!(header.memsz), read_from_packed!(header.flags), read_from_packed!(header.align));
        }
    }
}
