extern crate ansi_term;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate scroll_derive;
extern crate scroll;
extern crate structopt;
#[macro_use]
extern crate strum_macros;
extern crate textwrap;

mod magic;
mod binary;
mod format;
mod formats;

use crate::binary::Binary;
use crate::formats::FileFormat;

use failure::Error;

use structopt::StructOpt;

#[derive(Debug, Fail)]
pub enum Problem {
    #[fail(display = "{}", _0)]
    Msg (String)
}

#[derive(StructOpt, Debug)]
#[structopt(name = "bininfo")]
pub struct Opt {

    /// Number of lines before trim
    #[structopt(short = "t", long = "trim", default_value = "20", help = "number of lines before trim")]
    trim_lines: usize,

    /// Number of chars before wrap
    #[structopt(short = "w", long = "wrap", default_value = "120", help = "number of chars before wrap")]
    wrap_chars: usize,

    /// File to print info about
    #[structopt(help = "file path")]
    file: String

}

use std::fs::File;
use std::io::Read;

fn main() {

    let opt = Opt::from_args();

    match run(opt) {
        Ok(()) => (),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

}

fn run(opt: Opt) -> Result<(), Error> {

    let file_path = &opt.file;

    let mut fd = File::open(file_path)
        .map_err(|e| Problem::Msg(format!("Cannot open file {:?}: {}", file_path, e)))?;


    let mut buffer = Vec::new();
    fd.read_to_end(&mut buffer)
        .map_err(|e| Problem::Msg(format!("Cannot read file {:?}: {}", file_path, e)))?;

    let bin = Binary::parse(opt, &buffer)?;

    match bin {
        Binary::Bmp(bmp) => {
            bmp.print()?;
        },
        Binary::Png(png) => {
            png.print()?;
        },
        Binary::Gif(gif) => {
            gif.print()?;
        },
        Binary::Jpg(jpg) => {
            jpg.print()?;
        },

        Binary::Pe(pe) => {
            pe.print()?;
        },
        Binary::Elf(elf) => {
            elf.print()?;
        },
        Binary::JavaClass(java_class) => {
            java_class.print()?;
        },
        Binary::MachO(macho) => {
            macho.print()?;
        }
        Binary::Lua(lua) => {
            lua.print()?;
        }

        Binary::Pdf(pdf) => {
            pdf.print()?;
        },

        Binary::Xp3(xp3) => {
            xp3.print()?;
        },
        Binary::Zip(zip) => {
            zip.print()?;
        }

        Binary::Unknown  => {
            use ansi_term::Color;

            eprintln!("{}", Color::Black.on(Color::Red).paint("Unknown/unsupported file format"));
            eprintln!("Check for newest version: {}", "https://github.com/Forlos/bininfo");
        },
    }

    Ok(())
}
