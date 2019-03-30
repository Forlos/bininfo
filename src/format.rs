use crate::formats::png::{Prefix, Postfix};
use failure::Error;
use scroll::Pread;
use prettytable::{Cell, Row, Table};

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
    Elf_header, Elf_symbol_header, Elf_section_header, Elf_rel, Elf_rela,
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

pub fn fmt_elf_sym_table(symtab: &Vec<Elf_symbol_header>, symstr: &Vec<u8>, section_headers: &Vec<Elf_section_header>, sh_strtab: &Vec<u8>) -> Result<(), Error> {
    use textwrap::{fill, termwidth};

    let term_width = termwidth() / 2;

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
            Cell::new(&fill(symbol, term_width)).style_spec("Fy"),
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

pub fn fmt_elf_rel_table(rel: &Vec<Elf_rel>, dynsym: &Vec<Elf_symbol_header>, dynstr: &Vec<u8>, machine: u16) -> Result<(), Error>{
    use textwrap::{fill, termwidth};

    let term_width = termwidth() / 2;

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
            Fy->fill(dynstr.pread::<&str>(dynsym[info].st_name as usize)?, term_width),
        ]);
    }
    table.printstd();

    Ok(())

}

pub fn fmt_elf_rela_table(rela: &Vec<Elf_rela>, dynsym: &Vec<Elf_symbol_header>, dynstr: &Vec<u8>, machine: u16) -> Result<(), Error>{
    use ansi_term::Color;
    use textwrap::{fill, termwidth};

    let term_width = termwidth() / 2;

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
        let name = dynstr.pread::<&str>(dynsym[info].st_name as usize)?;

        table.add_row(row![
            Fr->format!("{:>#16X}", header.r_offset),
            r_to_str(header.r_info as u32 & 0xFF, machine),
            fill(&format!("{}+{}", Color::Yellow.paint(name), Color::Red.paint(format!("{}", header.r_addend))), term_width),
        ]);
    }
    table.printstd();

    Ok(())

}

pub fn align(alignment: usize, mut offset: usize) -> usize {
    let diff = offset % alignment;
    if diff != 0 {
        offset += alignment - diff;
    }
    offset
}
