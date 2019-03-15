use crate::magic::{self, Format};
use crate::formats::*;

use failure::{
    Error,
};

// #[derive(Debug)]
pub enum Binary {
    Png(png::Png),
    Bmp(bmp::Bmp),
    Unknown,
}

impl Binary {

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        use std::io::Cursor;

        match magic::parse(&mut Cursor::new(&buf))? {
            Format::Png => Ok(Binary::Png(png::Png::parse(buf)?)),
            Format::Bmp => Ok(Binary::Bmp(bmp::Bmp::parse(buf)?)),
            Format::Unknown => Ok(Binary::Unknown),
        }

    }

}
