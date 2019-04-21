// Image formats
pub mod png;
pub mod bmp;
pub mod gif;
pub mod jpg;

// Executable formats
pub mod pe;
pub mod elf;
pub mod javaclass;
pub mod macho;
pub mod lua;

// Archive formats
pub mod xp3;
pub mod zip;

// Document formats
pub mod pdf;

use crate::Opt;
use failure::{Error};

pub trait FileFormat {
    type Item;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self::Item, Error>;
    fn print(&self) -> Result<(), Error>;

}
