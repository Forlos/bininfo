pub mod png;
pub mod bmp;
pub mod elf;
pub mod gif;
pub mod pdf;
pub mod jpg;
pub mod pe;

use crate::Opt;
use failure::{Error};

pub trait FileFormat {
    type Item;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self::Item, Error>;
    fn print(&self) -> Result<(), Error>;

}
