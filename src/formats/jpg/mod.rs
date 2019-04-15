use crate::Opt;
use failure::{Error};

pub const JPG_MAGIC: &'static [u8; JPG_MAGIC_SIZE] = b"\xFF\xD8\xFF\xDB";
pub const JPG_MAGIC_2: &'static [u8; JPG_MAGIC_SIZE] = b"\xFF\xD8\xFF\xEE";
pub const JPG_MAGIC_SIZE: usize = 4;

pub const _JPG_MAGIC_JFIF: &'static [u8; _JFIF_SIZE] = b"\xFF\xD8\xFF\xE0\x00\x10\x4A\x46\x49\x46\x00\x01";
pub const _JFIF_SIZE: usize = 12;

pub const _JPG_MAGIC_EXIF_1: &'static [u8; JPG_MAGIC_SIZE] = b"\xFF\xD8\xFF\xE1";
pub const _JPG_MAGIC_EXIF_2: &'static [u8; _EXIF_SIZE] = b"\x45\x78\x69\x66\x00\x00";
pub const _EXIF_SIZE: usize = 6;

#[derive(Debug)]
pub struct Jpg {
    opt: Opt,
}

impl super::FileFormat for Jpg {
    type Item = Self;

    fn parse(opt: Opt, _buf: &[u8]) -> Result<Self, Error> {

        Ok(Jpg {
            opt,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("JPG");
        println!("{:#X?}", self);

        Ok(())
    }


}
