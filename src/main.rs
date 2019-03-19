extern crate ansi_term;
extern crate clap;
#[macro_use]
extern crate prettytable;
extern crate scroll;
#[macro_use]
extern crate scroll_derive;
#[macro_use]
extern crate failure;

mod magic;
mod binary;
mod format;
mod formats;

use crate::binary::Binary;

use clap::{
    App,Arg,
};

use failure::{
    Error,
};

#[derive(Debug, Fail)]
pub enum Problem {
    #[fail(display = "{}", _0)]
    Msg (String)
}

use std::fs::File;
use std::io::Read;

fn main() {

    let app = App::new("bininfo")
        .about("Binary file info.")
        .version("0.1.0")
        .arg(Arg::with_name("FILE")
             .help("file path")
             .index(1)
             .required(true))
        .get_matches();

    let file_path = app.value_of("FILE").unwrap();

    match run(file_path) {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

}

fn run(file_path: &str) -> Result<(), Error> {

    let mut fd = File::open(file_path)
        .map_err(|e| Problem::Msg(format!("Cannot open file {:?}: {}", file_path, e)))?;

    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer)
        .map_err(|e| Problem::Msg(format!("Cannot read file {:?}: {}", file_path, e)))?;

    let bin = Binary::parse(&buffer)?;

    match bin {
        Binary::Bmp(bmp) => {
            bmp.print()?;
        },
        Binary::Png(png) => {
            png.print()?;
        },
        Binary::Elf(elf) => {
            elf.print()?;
        }
        Binary::Unknown  => (),
    }

    Ok(())
}
