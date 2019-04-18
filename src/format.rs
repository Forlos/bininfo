use crate::formats::png::{Prefix, Postfix};
use failure::Error;
use scroll::Pread;
use prettytable::{Cell, Row, Table};
use textwrap::fill;

pub fn fmt_indent(fmt: String) {
    print!("{:>2}", "");
    print!("{}", fmt);
}

pub fn fmt_indentln(fmt: String) {
    print!("{:>2}", "");
    println!("{}", fmt);
}

pub fn fmt_png_header(name: &'static str, prefix: &Prefix, postfix: &Postfix) {
    use ansi_term::Color;

    println!("{} {} {}",
             Color::White.underline().paint(name),
             Color::Yellow.paint(format!("Size: {}",prefix.size)),
             Color::Green.paint(format!("Checksum: {:#010X}", postfix.checksum)));
}

use crate::formats::elf::{
    Elf_header, Elf_symbol_header, Elf_section_header, Elf_rel, Elf_rela, Elf_dynamic,
    ET_REL, ET_EXEC, ET_DYN, ET_CORE,
    STB_LOCAL, STB_GLOBAL, STB_WEAK,
    STT_OBJECT, STT_FUNC, STT_SECTION, STT_FILE, STT_GNU_IFUNC,
    ELFDATA2LSB,
    et_to_str, machine_to_str, type_to_str, bind_to_str, r_to_str,
};

pub fn fmt_elf(header: &Elf_header) {
    use ansi_term::Color;

    let str_type = et_to_str(header.e_type);
    print!("ELF ");

    match header.e_type {
        ET_REL => {
            print!("{} ", Color::Yellow.paint(str_type));
        },
        ET_EXEC => {
            print!("{} ", Color::Red.paint(str_type));
        },
        ET_DYN => {
            print!("{} ", Color::Blue.paint(str_type));
        },
        ET_CORE => {
            print!("{} ", Color::Black.paint(str_type));
        },
        _ => {},
    }

    print!("{} ", Color::White.paint(machine_to_str(header.e_machine)));
    print!("{} ", Color::Blue.paint(if header.e_ident.ei_data == ELFDATA2LSB { "little-endian" }
           else { "big-endian" }));

    println!("@ {}:", Color::Red.paint(format!("{:#X}", header.e_entry)));
    println!("e_phoff: {} e_shoff: {} e_flags: {:#X} e_ehsize: {} e_phentsize: {} e_phnum: {} e_shentsize: {} e_shnum: {} e_shstrndx: {}",
             Color::Yellow.paint(format!("{:#X}", header.e_phoff)),
             Color::Yellow.paint(format!("{:#X}", header.e_shoff)),
             header.e_flags,
             header.e_ehsize,
             header.e_phentsize,
             header.e_phnum,
             header.e_shentsize,
             header.e_shnum,
             header.e_shstrndx);
}

pub fn fmt_elf_flags(flags: u32) -> String {

    match flags {
        7 => "RWX".to_owned(),
        6 => "RW-".to_owned(),
        5 => "R-X".to_owned(),
        4 => "R--".to_owned(),
        3 => "-WX".to_owned(),
        2 => "-W-".to_owned(),
        1 => "--X".to_owned(),
        _ => "---".to_owned(),
    }

}

pub fn fmt_elf_sym_table(symtab: &Vec<Elf_symbol_header>, symstr: &Vec<u8>, section_headers: &Vec<Elf_section_header>, sh_strtab: &Vec<u8>, wrap: usize) -> Result<(), Error> {

    let mut table = Table::new();
    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(1, 1)
        .build();
    table.set_format(format);
    table.add_row(row![r->"Addr", "Bind", "Type", "Symbol", "Section", "Size", "Other"]);
    for header in symtab.iter() {

        let bind_cell = {
            let bind_cell = Cell::new(&format!("{:<8}", bind_to_str(header.st_info >> 4)));
            match header.st_info >> 4 {
                STB_LOCAL => bind_cell.style_spec("bBCFD"),
                STB_GLOBAL => bind_cell.style_spec("bBRFD"),
                STB_WEAK => bind_cell.style_spec("bBMFD"),
                _ => bind_cell
            }
        };
        let typ_cell = {
            let typ_cell = Cell::new(&format!("{:<9}", type_to_str(header.st_info & 0xF)));
            match header.st_info & 0xF {
                STT_OBJECT => typ_cell.style_spec("bFY"),
                STT_FUNC => typ_cell.style_spec("bFR"),
                STT_GNU_IFUNC => typ_cell.style_spec("bFC"),
                STT_FILE => typ_cell.style_spec("bFB"),
                STT_SECTION => typ_cell.style_spec("bFW"),
                _ => typ_cell
            }
        };

        let symbol = symstr.pread::<&str>(header.st_name as usize)?;

        table.add_row(Row::new(vec![
            Cell::new(&format!("{:>#16X}", header.st_value)).style_spec("Frr"),
            bind_cell,
            typ_cell,
            Cell::new(&fill(symbol, wrap)).style_spec("Fy"),
            Cell::new(if (header.st_shndx as usize) < (section_headers.len()) {
                sh_strtab.pread::<&str>(section_headers[header.st_shndx as usize].sh_name as usize)?
            }
                      else {
                          "ABS"
                      }),
            Cell::new(&format!("{:#X}", header.st_size)).style_spec("Fg"),
            Cell::new(&format!("{:#X}", header.st_other)),
        ]));
    }
    table.printstd();

    Ok(())

}

pub fn fmt_elf_rel_table(rel: &Vec<Elf_rel>, dynsym: &Vec<Elf_symbol_header>, dynstr: &Vec<u8>, machine: u16, wrap: usize) -> Result<(), Error>{

    let mut table = Table::new();
    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(1, 1)
        .build();
    table.set_format(format);
    table.add_row(row![r->"Offset", "Type", "Name"]);
    for header in rel {

        let info = header.r_info as usize >> 8;

        table.add_row(row![
            Fr->format!("{:>#16X}", header.r_offset),
            r_to_str(header.r_info as u32 & 0xFF, machine),
            Fy->fill(dynstr.pread::<&str>(dynsym[info].st_name as usize)?, wrap),
        ]);
    }
    table.printstd();

    Ok(())

}

pub fn fmt_elf_rela_table(rela: &Vec<Elf_rela>, dynsym: &Vec<Elf_symbol_header>, dynstr: &Vec<u8>, machine: u16, wrap: usize) -> Result<(), Error>{
    use ansi_term::Color;

    let mut table = Table::new();
    let format = prettytable::format::FormatBuilder::new()
        .column_separator(' ')
        .borders(' ')
        .padding(1, 1)
        .build();
    table.set_format(format);
    table.add_row(row![r->"Offset", "Type", "Name+addend"]);
    for header in rela {

        let info = header.r_info as usize >> 32;
        let mut name = Color::Yellow.paint(dynstr.pread::<&str>(dynsym[info].st_name as usize)?);

        if name.len() == 0 {
            name = Color::White.paint("ABS");
        }

        table.add_row(row![
            Fr->format!("{:>#16X}", header.r_offset),
            r_to_str(header.r_info as u32 & 0xFF, machine),
            fill(&format!("{}+{}", name, Color::Red.paint(format!("{}", header.r_addend))), wrap),
        ]);
    }
    table.printstd();

    Ok(())

}

pub fn fmt_elf_dynamic(dynamic: &Vec<Elf_dynamic>, dynstr: &Vec<u8>) -> Result<(), Error> {
    use ansi_term::Color;

    for header in dynamic {

        print!("  {:>16} ", tag_to_str(header.d_tag));

        match header.d_tag {
            DT_RPATH        => println!("{}", Color::Red.paint(dynstr.pread::<&str>(header.d_ptr as usize)?)),
            DT_NEEDED       => println!("{}", Color::Blue.paint(dynstr.pread::<&str>(header.d_ptr as usize)?)),
            DT_INIT         => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_FINI         => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_INIT_ARRAY   => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_INIT_ARRAYSZ => println!("{}", Color::Green.paint(format!("{:#X}", header.d_ptr))),
            DT_FINI_ARRAY   => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_FINI_ARRAYSZ => println!("{}", Color::Green.paint(format!("{:#X}", header.d_ptr))),
            DT_GNU_HASH     => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_STRTAB       => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_SYMTAB       => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_STRSZ        => println!("{}", Color::Green.paint(format!("{:#X}", header.d_ptr))),
            DT_PLTGOT       => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_PLTRELSZ     => println!("{}", Color::Green.paint(format!("{:#X}", header.d_ptr))),
            DT_JMPREL       => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_RELA         => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_RELASZ       => println!("{}", Color::Green.paint(format!("{:#X}", header.d_ptr))),
            DT_VERNEED      => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            DT_VERSYM       => println!("{}", Color::Red.paint(format!("{:#X}", header.d_ptr))),
            _ => println!("{:#x}", header.d_ptr),
        }

    }

    Ok(())

}

pub fn align(alignment: usize, mut offset: usize) -> usize {
    let diff = offset % alignment;
    if diff != 0 {
        offset += alignment - diff;
    }
    offset
}

// Marks end of dynamic section
const DT_NULL: u64 = 0;
// Name of needed library
pub const DT_NEEDED: u64 = 1;
// Size in bytes of PLT relocs
const DT_PLTRELSZ: u64 = 2;
// Processor defined value
const DT_PLTGOT: u64 = 3;
// Address of symbol hash table
const DT_HASH: u64 = 4;
// Address of string table
const DT_STRTAB: u64 = 5;
// Address of symbol table
const DT_SYMTAB: u64 = 6;
// Address of Rela relocs
const DT_RELA: u64 = 7;
// Total size of Rela relocs
const DT_RELASZ: u64 = 8;
// Size of one Rela reloc
const DT_RELAENT: u64 = 9;
// Size of string table
const DT_STRSZ: u64 = 10;
// Size of one symbol table entry
const DT_SYMENT: u64 = 11;
// Address of init function
const DT_INIT: u64 = 12;
// Address of termination function
const DT_FINI: u64 = 13;
// Name of shared object
const DT_SONAME: u64 = 14;
// Library search path (deprecated)
const DT_RPATH: u64 = 15;
// Start symbol search here
const DT_SYMBOLIC: u64 = 16;
// Address of Rel relocs
const DT_REL: u64 = 17;
// Total size of Rel relocs
const DT_RELSZ: u64 = 18;
// Size of one Rel reloc
const DT_RELENT: u64 = 19;
// Type of reloc in PLT
const DT_PLTREL: u64 = 20;
// For debugging; unspecified
const DT_DEBUG: u64 = 21;
// Reloc might modify .text
const DT_TEXTREL: u64 = 22;
// Address of PLT relocs
const DT_JMPREL: u64 = 23;
// Process relocations of object
const DT_BIND_NOW: u64 = 24;
// Array with addresses of init fct
const DT_INIT_ARRAY: u64 = 25;
// Array with addresses of fini fct
const DT_FINI_ARRAY: u64 = 26;
// Size in bytes of DT_INIT_ARRAY
const DT_INIT_ARRAYSZ: u64 = 27;
// Size in bytes of DT_FINI_ARRAY
const DT_FINI_ARRAYSZ: u64 = 28;
// Library search path
const DT_RUNPATH: u64 = 29;
// Flags for the object being loaded
const DT_FLAGS: u64 = 30;
// Start of encoded range
const DT_ENCODING: u64 = 32;
// Array with addresses of preinit fct
const DT_PREINIT_ARRAY: u64 = 32;
// size in bytes of DT_PREINIT_ARRAY
const DT_PREINIT_ARRAYSZ: u64 = 33;
// Number used
const DT_NUM: u64 = 34;
// Start of OS-specific
const DT_LOOS: u64 = 0x6000000d;
// End of OS-specific
const DT_HIOS: u64 = 0x6ffff000;
// Start of processor-specific
const DT_LOPROC: u64 = 0x70000000;
// End of processor-specific
const DT_HIPROC: u64 = 0x7fffffff;
// Most used by any processor
// const DT_PROCNUM: u64 = DT_MIPS_NUM;

// DT_* entries which fall between DT_ADDRRNGHI & DT_ADDRRNGLO use the
// Dyn.d_un.d_ptr field of the Elf*_Dyn structure.
//
// If any adjustment is made to the ELF object after it has been
// built these entries will need to be adjusted.
const DT_ADDRRNGLO: u64 = 0x6ffffe00;
// GNU-style hash table
const DT_GNU_HASH: u64 = 0x6ffffef5;
//
const DT_TLSDESC_PLT: u64 = 0x6ffffef6;
//
const DT_TLSDESC_GOT: u64 = 0x6ffffef7;
// Start of conflict section
const DT_GNU_CONFLICT: u64 = 0x6ffffef8;
// Library list
const DT_GNU_LIBLIST: u64 = 0x6ffffef9;
// Configuration information
const DT_CONFIG: u64 = 0x6ffffefa;
// Dependency auditing
const DT_DEPAUDIT: u64 = 0x6ffffefb;
// Object auditing
const DT_AUDIT: u64 = 0x6ffffefc;
// PLT padding
const DT_PLTPAD: u64 = 0x6ffffefd;
// Move table
const DT_MOVETAB: u64 = 0x6ffffefe;
// Syminfo table
const DT_SYMINFO: u64 = 0x6ffffeff;
//
const DT_ADDRRNGHI: u64 = 0x6ffffeff;

//DT_ADDRTAGIDX(tag)	(DT_ADDRRNGHI - (tag))	/* Reverse order! */
const DT_ADDRNUM: u64 = 11;

// The versioning entry types. The next are defined as part of the GNU extension
const DT_VERSYM: u64 = 0x6ffffff0;
const DT_RELACOUNT: u64 = 0x6ffffff9;
const DT_RELCOUNT: u64 = 0x6ffffffa;
// State flags, see DF_1_* below
const DT_FLAGS_1: u64 = 0x6ffffffb;
// Address of version definition table
const DT_VERDEF: u64 = 0x6ffffffc;
// Number of version definitions
const DT_VERDEFNUM: u64 = 0x6ffffffd;
// Address of table with needed versions
const DT_VERNEED: u64 = 0x6ffffffe;
// Number of needed versions
const DT_VERNEEDNUM: u64 = 0x6fffffff;

#[inline]
fn tag_to_str(tag: u64) -> &'static str {
    match tag {
        DT_NULL => "DT_NULL",
        DT_NEEDED => "DT_NEEDED",
        DT_PLTRELSZ => "DT_PLTRELSZ",
        DT_PLTGOT => "DT_PLTGOT",
        DT_HASH => "DT_HASH",
        DT_STRTAB => "DT_STRTAB",
        DT_SYMTAB => "DT_SYMTAB",
        DT_RELA => "DT_RELA",
        DT_RELASZ => "DT_RELASZ",
        DT_RELAENT => "DT_RELAENT",
        DT_STRSZ => "DT_STRSZ",
        DT_SYMENT => "DT_SYMENT",
        DT_INIT => "DT_INIT",
        DT_FINI => "DT_FINI",
        DT_SONAME => "DT_SONAME",
        DT_RPATH => "DT_RPATH",
        DT_SYMBOLIC => "DT_SYMBOLIC",
        DT_REL => "DT_REL",
        DT_RELSZ => "DT_RELSZ",
        DT_RELENT => "DT_RELENT",
        DT_PLTREL => "DT_PLTREL",
        DT_DEBUG => "DT_DEBUG",
        DT_TEXTREL => "DT_TEXTREL",
        DT_JMPREL => "DT_JMPREL",
        DT_BIND_NOW => "DT_BIND_NOW",
        DT_INIT_ARRAY => "DT_INIT_ARRAY",
        DT_FINI_ARRAY => "DT_FINI_ARRAY",
        DT_INIT_ARRAYSZ => "DT_INIT_ARRAYSZ",
        DT_FINI_ARRAYSZ => "DT_FINI_ARRAYSZ",
        DT_RUNPATH => "DT_RUNPATH",
        DT_FLAGS => "DT_FLAGS",
        DT_PREINIT_ARRAY => "DT_PREINIT_ARRAY",
        DT_PREINIT_ARRAYSZ => "DT_PREINIT_ARRAYSZ",
        DT_NUM => "DT_NUM",
        DT_LOOS => "DT_LOOS",
        DT_HIOS => "DT_HIOS",
        DT_LOPROC => "DT_LOPROC",
        DT_HIPROC => "DT_HIPROC",
        DT_VERSYM => "DT_VERSYM",
        DT_RELACOUNT => "DT_RELACOUNT",
        DT_RELCOUNT => "DT_RELCOUNT",
        DT_GNU_HASH => "DT_GNU_HASH",
        DT_VERDEF => "DT_VERDEF",
        DT_VERDEFNUM => "DT_VERDEFNUM",
        DT_VERNEED => "DT_VERNEED",
        DT_VERNEEDNUM => "DT_VERNEEDNUM",
        DT_FLAGS_1 => "DT_FLAGS_1",
        _ => "UNKNOWN_TAG",
    }
}

use crate::formats::pe::{
    self,
    COFF_header,
    is_dll, is_exe,characteristics_to_str,
};

pub fn fmt_pe(header: &COFF_header) {
    use ansi_term::Color;

    print!("PE ");

    if is_dll(header.characteristics) {
        print!("{} ",Color::Blue.paint("DLL"));
    }
    if is_exe(header.characteristics) {
        print!("{} ", Color::Red.paint("EXE"))
    }

    println!("{}", Color::White.paint(pe::machine_to_str(header.machine))) ;
    println!("{}\n", characteristics_to_str(header.characteristics));

    println!("{}",Color::White.underline().paint("Header"));
    fmt_indentln(format!("Number of sections: {}", Color::Purple.paint(format!("{}", header.n_of_sections))));
    fmt_indentln(format!("Timedate stamp: {:#X}", header.timedate_stamp));
    fmt_indentln(format!("Pointer to symbol table: {:#X}", header.pointer_to_symtab));
    fmt_indentln(format!("Number of symbols: {}", Color::Purple.paint(format!("{}", header.n_of_symtab))));
    fmt_indentln(format!("Size of optional header: {}", Color::Green.paint(format!("{}", header.sz_of_opt_header))));
    println!();

}

use crate::formats::macho::{
    Mach_header,
    mach_is_exe, mach_is_lib,
};

pub fn fmt_macho(header: &Mach_header) {
    use ansi_term::Color;

    print!("Mach-O ");

    if mach_is_lib(header.filetype) {
        print!("{} ",Color::Blue.paint("LIB"));
    }
    if mach_is_exe(header.filetype) {
        print!("{} ", Color::Red.paint("EXE"))
    }
    println!();

}
