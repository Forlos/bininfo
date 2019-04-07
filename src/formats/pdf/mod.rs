use failure::{Error};
use scroll::{self, Pread};

use crate::Opt;
// use crate::Problem;

pub const PDF_MAGIC: &'static [u8; PDF_MAGIC_SIZE] = b"%PDF-";
pub const PDF_MAGIC_SIZE: usize = 5;

pub struct Pdf {
    version: String,
}

impl super::FileFormat for Pdf {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        // Really not sure if this is how I should go about this. I need to read some docs!
        let version = buf.pread_with::<&str>(PDF_MAGIC_SIZE, scroll::ctx::StrCtx::Delimiter(0x25))?.split_whitespace().next().unwrap().to_string();

        Ok(Pdf {
            version,
        })
    }

    fn print(&self) -> Result<(), Error> {

        println!("PDF{}", self.version);

        Ok(())
    }

}
