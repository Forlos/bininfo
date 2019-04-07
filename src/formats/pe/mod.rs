use crate::Opt;
use failure::{Error};

pub const PE_MAGIC: &'static [u8; PE_MAGIC_SIZE] = b"MZ";
pub const PE_MAGIC_SIZE: usize = 2;

pub struct Pe {
    opt: Opt,
}

impl super::FileFormat for Pe {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        Ok(Pe {
            opt,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("PE");

        Ok(())
    }


}
