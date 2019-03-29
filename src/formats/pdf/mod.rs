use failure::{Error};

// use crate::Problem;

pub const PDF_MAGIC: &'static [u8; PDF_MAGIC_SIZE] = b"%PDF-";
pub const PDF_MAGIC_SIZE: usize = 5;

pub struct Pdf {
    
}

impl super::FileFormat for Pdf {
    type Item = Self;

    fn parse(buf: &[u8]) -> Result<Self, Error> {
        Ok(Pdf {

        })
    }

    fn print(&self) -> Result<(), Error> {

        println!("PDF");

        Ok(())
    }

}
