use crate::formats::png::{Prefix, Postfix};

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
    Elf_header,
    ET_REL, ET_EXEC, ET_DYN, ET_CORE,
    ELFDATA2LSB,
    et_to_str, machine_to_str,
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
