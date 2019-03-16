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
