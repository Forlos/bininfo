pub mod png;
pub mod bmp;
pub mod elf;
pub mod gif;
pub mod pdf;

use failure::{Error};

pub trait FileFormat {
    type Item;

    fn parse(buf: &[u8]) -> Result<Self::Item, Error>;
    fn print(&self) -> Result<(), Error>;

}
