use crate::Opt;
use failure::{Error};

pub const XP3_MAGIC: &'static [u8; XP3_MAGIC_SIZE] = b"XP3\x0d\x0a\x20\x0a\x1a";
pub const XP3_MAGIC_SIZE: usize = 8;

#[derive(Debug)]
pub struct Xp3 {
    opt: Opt,
}

impl super::FileFormat for Xp3 {
    type Item = Self;

    fn parse(opt: Opt, _buf: &[u8]) -> Result<Self, Error> {

        Ok(Xp3 {
            opt,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("XP3");
        println!("{:#X?}", self);

        Ok(())
    }


}
