use scroll::{self, Pread};

use failure::{
    Error,
};

use crate::format::{fmt_indent, fmt_indentln, fmt_png_header};

pub const ELF_MAGIC: &'static [u8; ELF_MAGIC_SIZE] = b"\x7FELF";
pub const ELF_MAGIC_SIZE: usize = 4;

#[derive(Debug, Pread)]
#[repr(C)]
struct E_ident {
    ei_mag: [u8; 4],
    ei_class: u8,
    ei_data: u8,
    ei_version: u8,
    ei_osabi: u8,
    ei_abiversion: u8,
    ei_pad: [u8; 7],
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_header_32 {
    e_ident: E_ident,
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u32,
    e_phoff: u32,
    e_shoff: u32,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_header_64 {
    e_ident: E_ident,
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[derive(Debug)]
#[repr(C)]
enum Elf_header {
    E32(Elf_header_32),
    E64(Elf_header_64),
}

const SIZE_OF_ELF_HEADER_32: usize = 52;
const SIZE_OF_ELF_HEADER_64: usize = 64;
// Invalid class.
pub const ELFCLASSNONE: u8 = 0;
// 32-bit objects.
pub const ELFCLASS32: u8 = 1;
// 64-bit objects.
pub const ELFCLASS64: u8 = 2;

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
struct Elf_program_header_64 {
    p_type:   u32,
    p_flags:  u32,
    p_offset: u64,
    p_vaddr:  u64,
    p_paddr:  u64,
    p_filesz: u64,
    p_memsz:  u64,
    p_align:  u64,
}

#[derive(Debug)]
#[repr(C)]
enum Program_header {
    P32(Elf_program_header_32),
    P64(Elf_program_header_64),
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
struct Elf_section_header_64 {
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

#[derive(Debug)]
#[repr(C)]
enum Section_header {
    S32(Elf_section_header_32),
    S64(Elf_section_header_64),
}


const SIZE_OF_SECTION_HEADER_32: usize = 10 * 4;
const SIZE_OF_SECTION_HEADER_64: usize = 6 * 8 + 4 * 4;

pub struct Elf {
    header: Elf_header,
    // program_headers: Vec<Program_header>,
    // section_headers: Vec<Section_header>,
}

impl Elf {


    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        let e_ident = buf.pread_with::<E_ident>(0, scroll::BE)?;
        let bit_format = e_ident.ei_class;
        let endianness = e_ident.ei_data;

        let header;

        match bit_format {

            ELFCLASS32 => {
                header = Elf_header::E32(buf.pread_with(0 ,scroll::BE)?);
            },
            ELFCLASS64 => {
                
                header = Elf_header::E64(buf.pread_with(0 ,scroll::BE)?);
            },
            _ => {
                panic!("Invalid EI_CLASS");
            },

        }

        // let header = buf.pread_with(0, scroll::BE)?;
            
        Ok(Elf {
            header,
        })

    }

    pub fn print(&self) -> Result<(), Error> {

        //
        // ELF file
        //
        println!("ELF");
        println!("{:#X?}", self.header);

        Ok(())

    }

}
