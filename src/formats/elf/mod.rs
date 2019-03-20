// https://github.com/m4b/goblin
include!("constants_header.rs");

use scroll::{self, Pread};

use failure::{
    Error,
};

use crate::format::{fmt_indent, fmt_indentln, fmt_elf};

pub const ELF_MAGIC: &'static [u8; ELF_MAGIC_SIZE] = b"\x7FELF";
pub const ELF_MAGIC_SIZE: usize = 4;

// Invalid class.
// const ELFCLASSNONE: u8 = 0;
// 32-bit objects.
const ELFCLASS32: u8 = 1;
// 64-bit objects.
const ELFCLASS64: u8 = 2;

// Invalid data encoding.
// const ELFDATANONE: u8 = 0;
// 2's complement, little endian.
pub const ELFDATA2LSB: u8 = 1;
// 2's complement, big endian.
pub const ELFDATA2MSB: u8 = 2;

// No file type.
const ET_NONE: u16 = 0;
// Relocatable file.
pub const ET_REL: u16 = 1;
// Executable file.
pub const ET_EXEC: u16 = 2;
// Shared object file.
pub const ET_DYN: u16 = 3;
// Core file.
pub const ET_CORE: u16 = 4;
// Number of defined types.
pub const ET_NUM: u16 = 5;

/// Convert an ET value to their associated string.
#[inline]
pub fn et_to_str(et: u16) -> &'static str {
    match et {
        ET_NONE => "NONE",
        ET_REL  => "REL",
        ET_EXEC => "EXEC",
        ET_DYN  => "DYN",
        ET_CORE => "CORE",
        ET_NUM  => "NUM",
        _       => "UNKNOWN_ET",
    }
}

// Program header table entry unused
pub const PT_NULL: u32 = 0;
// Loadable program segment
pub const PT_LOAD: u32 = 1;
// Dynamic linking information
pub const PT_DYNAMIC: u32 = 2;
// Program interpreter
pub const PT_INTERP: u32 = 3;
// Auxiliary information
pub const PT_NOTE: u32 = 4;
// Reserved
pub const PT_SHLIB: u32 = 5;
// Entry for header table itself
pub const PT_PHDR: u32 = 6;
// Thread-local storage segment
pub const PT_TLS: u32 = 7;
// Number of defined types
pub const PT_NUM: u32 = 8;
// Start of OS-specific
pub const PT_LOOS: u32 = 0x60000000;
// GCC .eh_frame_hdr segment
pub const PT_GNU_EH_FRAME: u32 = 0x6474e550;
// Indicates stack executability
pub const PT_GNU_STACK: u32 = 0x6474e551;
// Read-only after relocation
pub const PT_GNU_RELRO: u32 = 0x6474e552;
// Sun Specific segment
pub const PT_LOSUNW: u32 = 0x6ffffffa;
// Sun Specific segment
pub const PT_SUNWBSS: u32 = 0x6ffffffa;
// Stack segment
pub const PT_SUNWSTACK: u32 = 0x6ffffffb;
// End of OS-specific
pub const PT_HISUNW: u32 = 0x6fffffff;
// End of OS-specific
pub const PT_HIOS: u32 = 0x6fffffff;
// Start of processor-specific
pub const PT_LOPROC: u32 = 0x70000000;
// ARM unwind segment
pub const PT_ARM_EXIDX: u32 = 0x70000001;
// End of processor-specific
pub const PT_HIPROC: u32 = 0x7fffffff;

// Segment is executable
pub const PF_X: u32 = 1 << 0;
// Segment is writable
pub const PF_W: u32 = 1 << 1;
// Segment is readable
pub const PF_R: u32 = 1 << 2;

pub fn pt_to_str(pt: u32) -> &'static str {
    match pt {
        PT_NULL => "PT_NULL",
        PT_LOAD => "PT_LOAD",
        PT_DYNAMIC => "PT_DYNAMIC",
        PT_INTERP => "PT_INTERP",
        PT_NOTE => "PT_NOTE",
        PT_SHLIB => "PT_SHLIB",
        PT_PHDR => "PT_PHDR",
        PT_TLS => "PT_TLS",
        PT_NUM => "PT_NUM",
        PT_LOOS => "PT_LOOS",
        PT_GNU_EH_FRAME => "PT_GNU_EH_FRAME",
        PT_GNU_STACK => "PT_GNU_STACK",
        PT_GNU_RELRO => "PT_GNU_RELRO",
        PT_SUNWBSS => "PT_SUNWBSS",
        PT_SUNWSTACK => "PT_SUNWSTACK",
        PT_HIOS => "PT_HIOS",
        PT_LOPROC => "PT_LOPROC",
        PT_HIPROC => "PT_HIPROC",
        PT_ARM_EXIDX => "PT_ARM_EXIDX",
        _ => "UNKNOWN_PT",
    }
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct E_ident {
    pub ei_mag: [u8; 4],
    pub ei_class: u8,
    pub ei_data: u8,
    pub ei_version: u8,
    pub ei_osabi: u8,
    pub ei_abiversion: u8,
    ei_pad: [u8; 7],
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_header_32 {
    e_ident:     E_ident,
    e_type:      u16,
    e_machine:   u16,
    e_version:   u32,
    e_entry:     u32,
    e_phoff:     u32,
    e_shoff:     u32,
    e_flags:     u32,
    e_ehsize:    u16,
    e_phentsize: u16,
    e_phnum:     u16,
    e_shentsize: u16,
    e_shnum:     u16,
    e_shstrndx:  u16,
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Elf_header {
    pub e_ident:     E_ident,
    pub e_type:      u16,
    pub e_machine:   u16,
    pub e_version:   u32,
    pub e_entry:     u64,
    pub e_phoff:     u64,
    pub e_shoff:     u64,
    pub e_flags:     u32,
    pub e_ehsize:    u16,
    pub e_phentsize: u16,
    pub e_phnum:     u16,
    pub e_shentsize: u16,
    pub e_shnum:     u16,
    pub e_shstrndx:  u16,
}

impl From<Elf_header_32> for Elf_header {

    fn from(header: Elf_header_32) -> Self {

        Elf_header {
            e_ident:     header.e_ident,
            e_type:      header.e_type,
            e_machine:   header.e_machine,
            e_version:   header.e_version,
            e_entry:     header.e_entry as u64,
            e_phoff:     header.e_phoff as u64,
            e_shoff:     header.e_shoff as u64,
            e_flags:     header.e_flags,
            e_ehsize:    header.e_ehsize,
            e_phentsize: header.e_phentsize,
            e_phnum:     header.e_phnum,
            e_shentsize: header.e_shentsize,
            e_shnum:     header.e_shnum,
            e_shstrndx:  header.e_shstrndx,
        }

    }

}

const SIZE_OF_ELF_HEADER_32: usize = 52;
const SIZE_OF_ELF_HEADER_64: usize = 64;

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_program_header_32 {
    p_type:   u32,
    p_offset: u32,
    p_vaddr:  u32,
    p_paddr:  u32,
    p_filesz: u32,
    p_memsz:  u32,
    p_flags:  u32,
    p_align:  u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_program_header {
    p_type:   u32,
    p_flags:  u32,
    p_offset: u64,
    p_vaddr:  u64,
    p_paddr:  u64,
    p_filesz: u64,
    p_memsz:  u64,
    p_align:  u64,
}

impl From<Elf_program_header_32> for Elf_program_header {

    fn from(header: Elf_program_header_32) -> Self {

        Elf_program_header {
            p_type:   header.p_type,
            p_flags:  header.p_flags,
            p_offset: header.p_offset as u64,
            p_vaddr:  header.p_vaddr as u64,
            p_paddr:  header.p_paddr as u64,
            p_filesz: header.p_filesz as u64,
            p_memsz:  header.p_memsz as u64,
            p_align:  header.p_align as u64,
        }

    }

}

const SIZE_OF_PROGRAM_HEADER_32: usize = 8 * 4;
const SIZE_OF_PROGRAM_HEADER_64: usize = 6 * 8 + 2 * 4;

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_section_header_32 {
    sh_name:      u32,
    sh_type:      u32,
    sh_flags:     u32,
    sh_addr:      u32,
    sh_offset:    u32,
    sh_size:      u32,
    sh_link:      u32,
    sh_info:      u32,
    sh_addralign: u32,
    sh_entsize:   u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_section_header {
    sh_name:      u32,
    sh_type:      u32,
    sh_flags:     u64,
    sh_addr:      u64,
    sh_offset:    u64,
    sh_size:      u64,
    sh_link:      u32,
    sh_info:      u32,
    sh_addralign: u64,
    sh_entsize:   u64,
}

impl From<Elf_section_header_32> for Elf_section_header {

    fn from(header: Elf_section_header_32) -> Self {

        Elf_section_header {
            sh_name:      header.sh_name,
            sh_type:      header.sh_type,
            sh_flags:     header.sh_flags as u64,
            sh_addr:      header.sh_addr as u64,
            sh_offset:    header.sh_offset as u64,
            sh_size:      header.sh_size as u64,
            sh_link:      header.sh_link,
            sh_info:      header.sh_info,
            sh_addralign: header.sh_addralign as u64,
            sh_entsize:   header.sh_entsize as u64,
        }

    }

}

const SIZE_OF_SECTION_HEADER_32: usize = 10 * 4;
const SIZE_OF_SECTION_HEADER_64: usize = 6 * 8 + 4 * 4;

pub struct Elf {
    header: Elf_header,
    program_headers: Vec<Elf_program_header>,
    // section_headers: Vec<Section_header>,
}

impl Elf {


    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        let e_ident = buf.pread_with::<E_ident>(0, scroll::BE)?;
        let bit_format = e_ident.ei_class;
        let endianness = match e_ident.ei_data {
            ELFDATA2LSB => scroll::LE,
            ELFDATA2MSB => scroll::BE,
            _ => {
                panic!("Invalid EI_DATA");
            },
        };
        let header;
        let mut program_headers;

        match bit_format {

            ELFCLASS32 => {
                header = Elf_header::from(buf.pread_with::<Elf_header_32>(0, endianness)?);
                program_headers = Vec::with_capacity(header.e_phnum as usize);
                for i in 0..header.e_phnum as usize {
                    let p_head = Elf_program_header::from(
                        buf.pread_with::<Elf_program_header_32>(
                            header.e_phoff as usize + i * header.e_phentsize as usize, endianness)?);

                    program_headers.push(p_head);
                }
            },
            ELFCLASS64 => {
                header = buf.pread_with::<Elf_header>(0, endianness)?;
                program_headers = Vec::with_capacity(header.e_phnum as usize);
                for i in 0..header.e_phnum as usize {
                    let p_head = buf.pread_with(
                        header.e_phoff as usize + i * header.e_phentsize as usize, endianness)?;

                    program_headers.push(p_head);
                }
            },
            _ => {
                panic!("Invalid EI_CLASS");
            },

        }

        Ok(Elf {
            header,
            program_headers,
        })

    }

    pub fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;
        use prettytable::Table;

        //
        // ELF file
        //
        fmt_elf(&self.header);
        println!();

        //
        // Program Headers
        //
        println!("{}({})",
                 Color::White.paint("ProgramHeaders"),
                 self.program_headers.len());
        let mut table = Table::new();
        let format = prettytable::format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .padding(1, 1)
            .build();
        table.set_format(format);
        table.add_row(row!["Idx", "Type", "Flags", "Offset", "Vaddr", "Paddr", "Filesz","Memsz", "Align"]);
        for (i, header) in self.program_headers.iter().enumerate() {
            use ansi_term::{Color, ANSIString, ANSIStrings};
            let strings: &[ANSIString<'static>] = &[
                if header.p_flags & 0x04 != 0 { Color::Fixed(83).paint("R") } else { Color::Fixed(138).paint("-") },
                if header.p_flags & 0x02 != 0 { Color::Fixed(10).paint("W") } else { Color::Fixed(138).paint("-") },
                if header.p_flags & 0x01 != 0 { Color::Red.paint("X") } else { Color::Fixed(138).paint("-") },
            ];
            table.add_row(row![
                i,
                pt_to_str(header.p_type),
                format!("{}", ANSIStrings(strings)),
                Fy->format!("{:#X}", header.p_offset),
                Fr->format!("{:#X}", header.p_vaddr),
                Fr->format!("{:#X}", header.p_paddr),
                Fg->format!("{:#X}", header.p_filesz),
                Fg->format!("{:#X}", header.p_memsz),
                format!("{:#X}", header.p_align),
            ]);
        }
        table.printstd();
        println!();

        Ok(())

    }

}
