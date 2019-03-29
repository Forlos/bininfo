#![allow(dead_code)]
// https://github.com/m4b/goblin
include!("constants_header.rs");
include!("constants_relocation.rs");

use scroll::{self, Pread};

use failure::{
    Error,
};

use crate::format::{fmt_elf, fmt_elf_sym_table, fmt_elf_rel_table};

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
const PF_X: u32 = 1 << 0;
// Segment is writable
const PF_W: u32 = 1 << 1;
// Segment is readable
const PF_R: u32 = 1 << 2;

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

// Undefined section.
pub const SHN_UNDEF: u32 = 0;
// Start of reserved indices.
pub const SHN_LORESERVE: u32 = 0xff00;
// Start of processor-specific.
pub const SHN_LOPROC: u32 = 0xff00;
// Order section before all others (Solaris).
pub const SHN_BEFORE: u32 = 0xff00;
// Order section after all others (Solaris).
pub const SHN_AFTER: u32 = 0xff01;
// End of processor-specific.
pub const SHN_HIPROC: u32 = 0xff1f;
// Start of OS-specific.
pub const SHN_LOOS: u32 = 0xff20;
// End of OS-specific.
pub const SHN_HIOS: u32 = 0xff3f;
// Associated symbol is absolute.
pub const SHN_ABS: u32 = 0xfff1;
// Associated symbol is common.
pub const SHN_COMMON: u32 = 0xfff2;
// Index is in extra table.
pub const SHN_XINDEX: u32 = 0xffff;
// End of reserved indices.
pub const SHN_HIRESERVE: u32 = 0xffff;

// === Legal values for sh_type (section type). ===
// Section header table entry unused.
pub const SHT_NULL: u32 = 0;
// Program data.
pub const SHT_PROGBITS: u32 = 1;
// Symbol table.
pub const SHT_SYMTAB: u32 = 2;
// String table.
pub const SHT_STRTAB: u32 = 3;
// Relocation entries with addends.
pub const SHT_RELA: u32 = 4;
// Symbol hash table.
pub const SHT_HASH: u32 = 5;
// Dynamic linking information.
pub const SHT_DYNAMIC: u32 = 6;
// Notes.
pub const SHT_NOTE: u32 = 7;
// Program space with no data (bss).
pub const SHT_NOBITS: u32 = 8;
// Relocation entries, no addends.
pub const SHT_REL: u32 = 9;
// Reserved.
pub const SHT_SHLIB: u32 = 10;
// Dynamic linker symbol table.
pub const SHT_DYNSYM: u32 = 11;
// Array of constructors.
pub const SHT_INIT_ARRAY: u32 = 14;
// Array of destructors.
pub const SHT_FINI_ARRAY: u32 = 15;
// Array of pre-constructors.
pub const SHT_PREINIT_ARRAY: u32 = 16;
// Section group.
pub const SHT_GROUP: u32 = 17;
// Extended section indeces.
pub const SHT_SYMTAB_SHNDX: u32 = 18;
// Number of defined types.
pub const SHT_NUM: u32 = 19;
// Start OS-specific.
pub const SHT_LOOS: u32 = 0x60000000;
// Object attributes.
pub const SHT_GNU_ATTRIBUTES: u32 = 0x6ffffff5;
// GNU-style hash table.
pub const SHT_GNU_HASH: u32 = 0x6ffffff6;
// Prelink library list.
pub const SHT_GNU_LIBLIST: u32 = 0x6ffffff7;
// Checksum for DSO content.
pub const SHT_CHECKSUM: u32 = 0x6ffffff8;
// Sun-specific low bound.
pub const SHT_LOSUNW: u32 = 0x6ffffffa;
pub const SHT_SUNW_MOVE: u32 = 0x6ffffffa;
pub const SHT_SUNW_COMDAT: u32 = 0x6ffffffb;
pub const SHT_SUNW_SYMINFO: u32 = 0x6ffffffc;
// Version definition section.
pub const SHT_GNU_VERDEF: u32 = 0x6ffffffd;
// Version needs section.
pub const SHT_GNU_VERNEED: u32 = 0x6ffffffe;
// Version symbol table.
pub const SHT_GNU_VERSYM: u32 = 0x6fffffff;
// Sun-specific high bound.
pub const SHT_HISUNW: u32 = 0x6fffffff;
// End OS-specific type.
pub const SHT_HIOS: u32 = 0x6fffffff;
// Start of processor-specific.
pub const SHT_LOPROC: u32 = 0x70000000;
// End of processor-specific.
pub const SHT_HIPROC: u32 = 0x7fffffff;
// Start of application-specific.
pub const SHT_LOUSER: u32 = 0x80000000;
// End of application-specific.
pub const SHT_HIUSER: u32 = 0x8fffffff;

// Legal values for sh_flags (section flags)
// Writable.
pub const SHF_WRITE: u32 = 0x1;
// Occupies memory during execution.
pub const SHF_ALLOC: u32 = 0x2;
// Executable.
pub const SHF_EXECINSTR: u32 = 0x4;
// Might be merged.
pub const SHF_MERGE: u32 = 0x10;
// Contains nul-terminated strings.
pub const SHF_STRINGS: u32 = 0x20;
// `sh_info' contains SHT index.
pub const SHF_INFO_LINK: u32 = 0x40;
// Preserve order after combining.
pub const SHF_LINK_ORDER: u32 = 0x80;
// Non-standard OS specific handling required.
pub const SHF_OS_NONCONFORMING: u32 = 0x100;
// Section is member of a group.
pub const SHF_GROUP: u32 = 0x200;
// Section hold thread-local data.
pub const SHF_TLS: u32 = 0x400;
// Section with compressed data.
pub const SHF_COMPRESSED: u32 = 0x800;
// OS-specific..
pub const SHF_MASKOS: u32 = 0x0ff00000;
// Processor-specific.
pub const SHF_MASKPROC: u32 = 0xf0000000;
// Special ordering requirement (Solaris).
pub const SHF_ORDERED: u32 = 1 << 30;
// Number of "regular" section header flags
pub const SHF_NUM_REGULAR_FLAGS: usize = 12;
// // Section is excluded unless referenced or allocated (Solaris).
// pub const SHF_EXCLUDE: u32 = 1U << 31;



pub const SHF_FLAGS: [u32; SHF_NUM_REGULAR_FLAGS] = [
    SHF_WRITE,
    SHF_ALLOC,
    SHF_EXECINSTR,
    SHF_MERGE,
    SHF_STRINGS,
    SHF_INFO_LINK,
    SHF_LINK_ORDER,
    SHF_OS_NONCONFORMING,
    SHF_GROUP,
    SHF_TLS,
    SHF_COMPRESSED,
    SHF_ORDERED,
];

pub fn sht_to_str(sht: u32) -> &'static str {
    match sht {
        SHT_NULL => "SHT_NULL",
        SHT_PROGBITS => "SHT_PROGBITS",
        SHT_SYMTAB => "SHT_SYMTAB",
        SHT_STRTAB => "SHT_STRTAB",
        SHT_RELA => "SHT_RELA",
        SHT_HASH => "SHT_HASH",
        SHT_DYNAMIC => "SHT_DYNAMIC",
        SHT_NOTE => "SHT_NOTE",
        SHT_NOBITS => "SHT_NOBITS",
        SHT_REL => "SHT_REL",
        SHT_SHLIB => "SHT_SHLIB",
        SHT_DYNSYM => "SHT_DYNSYM",
        SHT_INIT_ARRAY => "SHT_INIT_ARRAY",
        SHT_FINI_ARRAY => "SHT_FINI_ARRAY",
        SHT_PREINIT_ARRAY => "SHT_PREINIT_ARRAY",
        SHT_GROUP => "SHT_GROUP",
        SHT_SYMTAB_SHNDX => "SHT_SYMTAB_SHNDX",
        SHT_NUM => "SHT_NUM",
        SHT_LOOS => "SHT_LOOS",
        SHT_GNU_ATTRIBUTES => "SHT_GNU_ATTRIBUTES",
        SHT_GNU_HASH => "SHT_GNU_HASH",
        SHT_GNU_LIBLIST => "SHT_GNU_LIBLIST",
        SHT_CHECKSUM => "SHT_CHECKSUM",
        SHT_SUNW_MOVE => "SHT_SUNW_MOVE",
        SHT_SUNW_COMDAT => "SHT_SUNW_COMDAT",
        SHT_SUNW_SYMINFO => "SHT_SUNW_SYMINFO",
        SHT_GNU_VERDEF => "SHT_GNU_VERDEF",
        SHT_GNU_VERNEED => "SHT_GNU_VERNEED",
        SHT_GNU_VERSYM => "SHT_GNU_VERSYM",
        SHT_LOPROC => "SHT_LOPROC",
        SHT_HIPROC => "SHT_HIPROC",
        SHT_LOUSER => "SHT_LOUSER",
        SHT_HIUSER => "SHT_HIUSER",
        _ => "UNKNOWN_SHT",
    }
}

pub fn shf_to_str(shf: u32) -> &'static str {
    match shf {
        SHF_WRITE => "SHF_WRITE",
        SHF_ALLOC => "SHF_ALLOC",
        SHF_EXECINSTR => "SHF_EXECINSTR",
        SHF_MERGE => "SHF_MERGE",
        SHF_STRINGS => "SHF_STRINGS",
        SHF_INFO_LINK => "SHF_INFO_LINK",
        SHF_LINK_ORDER => "SHF_LINK_ORDER",
        SHF_OS_NONCONFORMING => "SHF_OS_NONCONFORMING",
        SHF_GROUP => "SHF_GROUP",
        SHF_TLS => "SHF_TLS",
        SHF_COMPRESSED => "SHF_COMPRESSED",
        //SHF_MASKOS..SHF_MASKPROC => "SHF_OSFLAG",
        SHF_ORDERED => "SHF_ORDERED",
        _ => "SHF_UNKNOWN"
    }
}

// === Sym bindings ===
// Local symbol.
pub const STB_LOCAL: u8 = 0;
// Global symbol.
pub const STB_GLOBAL: u8 = 1;
// Weak symbol.
pub const STB_WEAK: u8 = 2;
// Number of defined types..
const STB_NUM: u8 = 3;
// Start of OS-specific.
const STB_LOOS: u8 = 10;
// Unique symbol..
const STB_GNU_UNIQUE: u8 = 10;
// End of OS-specific.
const STB_HIOS: u8 = 12;
// Start of processor-specific.
const STB_LOPROC: u8 = 13;
// End of processor-specific.
const STB_HIPROC: u8 = 15;

// === Sym types ===
// Symbol type is unspecified.
const STT_NOTYPE: u8 = 0;
// Symbol is a data object.
pub const STT_OBJECT: u8 = 1;
// Symbol is a code object.
pub const STT_FUNC: u8 = 2;
// Symbol associated with a section.
pub const STT_SECTION: u8 = 3;
// Symbol's name is file name.
pub const STT_FILE: u8 = 4;
// Symbol is a common data object.
const STT_COMMON: u8 = 5;
// Symbol is thread-local data object.
const STT_TLS: u8 = 6;
// Number of defined types.
const STT_NUM: u8 = 7;
// Start of OS-specific.
const STT_LOOS: u8 = 10;
// Symbol is indirect code object.
pub const STT_GNU_IFUNC: u8 = 10;
// End of OS-specific.
const STT_HIOS: u8 = 12;
// Start of processor-specific.
const STT_LOPROC: u8 = 13;
// End of processor-specific.
const STT_HIPROC: u8 = 15;

#[inline]
pub fn bind_to_str(typ: u8) -> &'static str {
    match typ {
        STB_LOCAL => "LOCAL",
        STB_GLOBAL => "GLOBAL",
        STB_WEAK => "WEAK",
        STB_NUM => "NUM",
        STB_GNU_UNIQUE => "GNU_UNIQUE",
        _ => "UNKNOWN_STB",
    }
}

#[inline]
pub fn type_to_str(typ: u8) -> &'static str {
    match typ {
        STT_NOTYPE => "NOTYPE",
        STT_OBJECT => "OBJECT",
        STT_FUNC => "FUNC",
        STT_SECTION => "SECTION",
        STT_FILE => "FILE",
        STT_COMMON => "COMMON",
        STT_TLS => "TLS",
        STT_NUM => "NUM",
        STT_GNU_IFUNC => "GNU_IFUNC",
        _ => "UNKNOWN_STT",
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

// const SIZE_OF_ELF_HEADER_32: usize = 52;
// const SIZE_OF_ELF_HEADER_64: usize = 64;

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

// const SIZE_OF_PROGRAM_HEADER_32: usize = 8 * 4;
// const SIZE_OF_PROGRAM_HEADER_64: usize = 6 * 8 + 2 * 4;

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
pub struct Elf_section_header {
    pub sh_name:      u32,
    pub sh_type:      u32,
    pub sh_flags:     u64,
    pub sh_addr:      u64,
    pub sh_offset:    u64,
    pub sh_size:      u64,
    pub sh_link:      u32,
    pub sh_info:      u32,
    pub sh_addralign: u64,
    pub sh_entsize:   u64,
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

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_symbol_header_32 {
    st_name: u32,
    st_value: u32,
    st_size: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Elf_symbol_header {
    pub st_name: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

impl From<Elf_symbol_header_32> for Elf_symbol_header {

    fn from(header: Elf_symbol_header_32) -> Self {

        Elf_symbol_header {
            st_name:  header.st_name,
            st_info:  header.st_info,
            st_other: header.st_other,
            st_shndx: header.st_shndx,
            st_value: header.st_value as u64,
            st_size:  header.st_size as u64,
        }

    }

}

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_rel_32 {
    r_offset: u32,
    r_info:   u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Elf_rel {
    pub r_offset: u64,
    pub r_info:   u64,
}

impl From<Elf_rel_32> for Elf_rel {

    fn from(header: Elf_rel_32) -> Self {

        Elf_rel {
            r_offset: header.r_offset as u64,
            r_info:   header.r_info   as u64,
        }

    }

}

#[derive(Debug, Pread)]
#[repr(C)]
struct Elf_rela_32 {
    r_offset: u32,
    r_info:   u32,
    r_addend: i32,
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Elf_rela {
    pub r_offset: u64,
    pub r_info:   u64,
    pub r_addend: i64,
}

impl From<Elf_rela_32> for Elf_rela {

    fn from(header: Elf_rela_32) -> Self {

        Elf_rela {
            r_offset: header.r_offset as u64,
            r_info:   header.r_info   as u64,
            r_addend: header.r_addend as i64,
        }

    }

}

// const SIZE_OF_SECTION_HEADER_32: usize = 10 * 4;
// const SIZE_OF_SECTION_HEADER_64: usize = 6 * 8 + 4 * 4;

pub struct Elf {
    header:          Elf_header,
    program_headers: Vec<Elf_program_header>,
    section_headers: Vec<Elf_section_header>,
    sh_strtab:       Vec<u8>,
    symtab:          Vec<Elf_symbol_header>,
    symstr:          Vec<u8>,
    dynsym:          Vec<Elf_symbol_header>,
    dynstr:          Vec<u8>,
    reldyn:          Vec<Elf_rel>,
    relplt:          Vec<Elf_rel>,
}

impl super::FileFormat for Elf {
    type Item = Self;

    fn parse(buf: &[u8]) -> Result<Self, Error> {

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
        let mut section_headers;
        let mut sh_strtab;
        let mut symtab = Vec::new();
        let mut symstr = Vec::new();
        let mut dynsym = Vec::new();
        let mut dynstr = Vec::new();
        let mut reldyn = Vec::new();
        let mut relplt = Vec::new();

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

                section_headers = Vec::with_capacity(header.e_shnum as usize);
                for i in 0..header.e_shnum as usize {
                    let s_head = Elf_section_header::from(
                        buf.pread_with::<Elf_section_header_32>(
                            header.e_shoff as usize + i * header.e_shentsize as usize, endianness)?);

                    section_headers.push(s_head)
                }

                let shstr_idx = section_headers[header.e_shstrndx as usize].sh_offset as usize;
                let shstr_size = section_headers[header.e_shstrndx as usize].sh_size as usize;
                sh_strtab = buf[shstr_idx..shstr_idx + shstr_size].to_vec();

                for head in &section_headers {
                    if head.sh_type == SHT_SYMTAB {
                        let size = head.sh_size as usize / head.sh_entsize as usize;
                        symtab = Vec::with_capacity(size);
                        for i in 0..size {
                            symtab.push(Elf_symbol_header::from(buf.pread_with::<Elf_symbol_header_32>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?));
                        }
                        let _symstr = &section_headers[head.sh_link as usize];
                        symstr = buf[_symstr.sh_offset as usize.._symstr.sh_offset as usize + _symstr.sh_size as usize].to_vec();
                    }
                    if head.sh_type == SHT_DYNSYM {
                        let size = head.sh_size as usize / head.sh_entsize as usize;
                        dynsym = Vec::with_capacity(size);
                        for i in 0..size {
                            dynsym.push(Elf_symbol_header::from(buf.pread_with::<Elf_symbol_header_32>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?));
                        }
                        let _dynstr = &section_headers[head.sh_link as usize];
                        dynstr = buf[_dynstr.sh_offset as usize.._dynstr.sh_offset as usize + _dynstr.sh_size as usize].to_vec();
                    }
                    if head.sh_type == SHT_REL {
                        if sh_strtab.pread::<&str>(head.sh_name as usize)? == ".rel.dyn" {
                            let size = head.sh_size as usize / head.sh_entsize as usize;
                            reldyn = Vec::with_capacity(size);
                            for i in 0..size {
                                reldyn.push(Elf_rel::from(buf.pread_with::<Elf_rel_32>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?));
                            }
                        }
                        if sh_strtab.pread::<&str>(head.sh_name as usize)? == ".rel.plt" {
                            let size = head.sh_size as usize / head.sh_entsize as usize;
                            relplt = Vec::with_capacity(size);
                            for i in 0..size {
                                relplt.push(Elf_rel::from(buf.pread_with::<Elf_rel_32>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?));
                            }
                        }
                    }
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

                section_headers = Vec::with_capacity(header.e_shnum as usize);
                for i in 0..header.e_shnum as usize {
                    let s_head = buf.pread_with(
                        header.e_shoff as usize + i * header.e_shentsize as usize, endianness)?;

                    section_headers.push(s_head)
                }

                let shstr_idx = section_headers[header.e_shstrndx as usize].sh_offset as usize;
                let shstr_size = section_headers[header.e_shstrndx as usize].sh_size as usize;
                sh_strtab = buf[shstr_idx..shstr_idx + shstr_size].to_vec();

                for head in &section_headers {
                    if head.sh_type == SHT_SYMTAB {
                        let size = head.sh_size as usize / head.sh_entsize as usize;
                        symtab = Vec::with_capacity(size);
                        for i in 0..size {
                            symtab.push(buf.pread_with::<Elf_symbol_header>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?);
                        }
                        let _symstr = &section_headers[head.sh_link as usize];
                        symstr = buf[_symstr.sh_offset as usize.._symstr.sh_offset as usize + _symstr.sh_size as usize].to_vec();
                    }
                    if head.sh_type == SHT_DYNSYM {
                        let size = head.sh_size as usize / head.sh_entsize as usize;
                        dynsym = Vec::with_capacity(size);
                        for i in 0..size {
                            dynsym.push(buf.pread_with::<Elf_symbol_header>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?);
                        }
                        let _dynstr = &section_headers[head.sh_link as usize];
                        dynstr = buf[_dynstr.sh_offset as usize.._dynstr.sh_offset as usize + _dynstr.sh_size as usize].to_vec();
                    }
                    if head.sh_type == SHT_REL {
                        if sh_strtab.pread::<&str>(head.sh_name as usize)? == ".rel.dyn" {
                            let size = head.sh_size as usize / head.sh_entsize as usize;
                            reldyn = Vec::with_capacity(size);
                            for i in 0..size {
                                reldyn.push(buf.pread_with::<Elf_rel>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?);
                            }
                        }
                        if sh_strtab.pread::<&str>(head.sh_name as usize)? == ".rel.plt" {
                            let size = head.sh_size as usize / head.sh_entsize as usize;
                            relplt = Vec::with_capacity(size);
                            for i in 0..size {
                                relplt.push(buf.pread_with::<Elf_rel>(head.sh_offset as usize + i * head.sh_entsize as usize, endianness)?);
                            }
                        }
                    }
                }

            },
            _ => {
                panic!("Invalid EI_CLASS");
            },

        }

        Ok(Elf {
            header,
            program_headers,
            section_headers,
            sh_strtab,
            symtab,
            symstr,
            dynsym,
            dynstr,
            reldyn,
            relplt,
        })

    }

    fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;
        use prettytable::{Table};

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
                if header.p_flags & PF_R != 0 { Color::Fixed(83).paint("R") } else { Color::Fixed(138).paint("-") },
                if header.p_flags & PF_W != 0 { Color::Fixed(10).paint("W") } else { Color::Fixed(138).paint("-") },
                if header.p_flags & PF_X != 0 { Color::Red.paint("X") } else { Color::Fixed(138).paint("-") },
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

        //
        // Section headers
        //
        println!("{}({})",
                 Color::White.paint("SectionHeaders"),
                 self.section_headers.len());
        let mut table = Table::new();
        let format = prettytable::format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .padding(1, 1)
            .build();
        table.set_format(format);
        table.add_row(row!["Idx", "Name", "Type", "Flags", "Addr", "Offset", "Size", "Link", "Entsize", "Align"]);
        for (i, header) in self.section_headers.iter().enumerate() {

            let flags_cell = {
                let shflags = header.sh_flags as u32;
                if shflags != 0 {
                    let mut flags = String::new();
                    for flag in &SHF_FLAGS {
                        let flag = *flag;
                        if shflags & flag == flag {
                            flags += &shf_to_str(flag).to_string().split_off(4);
                            flags += " ";
                        }
                    }
                    flags
                }
                else {
                    "".to_owned()
                }
            };

            table.add_row(row![
                i,
                self.sh_strtab.pread::<&str>(header.sh_name as usize)?,
                sht_to_str(header.sh_type),
                flags_cell,
                Fr->format!("{:#X}", header.sh_addr),
                Fy->format!("{:#X}", header.sh_offset),
                Fg->format!("{:#X}", header.sh_size),
                self.sh_strtab.pread::<&str>(self.section_headers[header.sh_link as usize].sh_name as usize)?,
                format!("{:#X}", header.sh_entsize),
                format!("{:#X}", header.sh_addralign),
            ]);
        }
        table.printstd();
        println!();

        //
        // Symbol table
        //
        println!("{}({})",
                 Color::White.paint("SymbolTable"),
                 self.symtab.len());
        if self.symtab.len() > 0 {
            fmt_elf_sym_table(&self.symtab, &self.symstr, &self.section_headers, &self.sh_strtab)?;
        }
        println!();

        //
        // DynSym table
        //
        println!("{}({})",
                 Color::White.paint("DynSymTable"),
                 self.dynsym.len());
        if self.dynsym.len() > 0 {
            fmt_elf_sym_table(&self.dynsym, &self.dynstr, &self.section_headers, &self.sh_strtab)?;
        }
        println!();

        //
        // RelDyn table
        //
        println!("{}({})",
                 Color::White.paint("RelDynTable"),
                 self.reldyn.len());
        if self.reldyn.len() > 0 {
            fmt_elf_rel_table(&self.reldyn, &self.dynsym, &self.dynstr, self.header.e_machine)?;
        }
        println!();

        //
        // RelPlt table
        //
        println!("{}({})",
                 Color::White.paint("RelPltTable"),
                 self.relplt.len());
        if self.relplt.len() > 0 {
            fmt_elf_rel_table(&self.relplt, &self.dynsym, &self.dynstr, self.header.e_machine)?;
        }
        println!();

        Ok(())

    }

}
