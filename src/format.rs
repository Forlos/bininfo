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
    et_to_str,
};

pub fn fmt_elf(header: &Elf_header) {
    use ansi_term::Color;

    let str_type = et_to_str(header.e_type);

    print!("ELF ");

    match header.e_type {
        ET_REL => {
            print!("{}", Color::Yellow.paint(str_type));
        },
        ET_EXEC => {
            print!("{}", Color::Red.paint(str_type));
        },
        ET_DYN => {
            print!("{}", Color::Blue.paint(str_type));
        },
        ET_CORE => {
            print!("{}", Color::Black.paint(str_type));
        },
        _ => {},
    }

    // match header.e_machine {
       
    // }

    println!();

}
