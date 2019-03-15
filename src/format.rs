

pub fn fmt_indent(fmt: String) {
    print!("{:>2}", "");
    print!("{}", fmt);
}

pub fn fmt_indentln(fmt: String) {
    print!("{:>2}", "");
    println!("{}", fmt);
}
