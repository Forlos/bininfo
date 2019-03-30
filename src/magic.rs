use crate::formats::*;
use crate::Problem;

use failure::{
    Error,
};

const MAGIC_SIZE: usize = 16;

pub enum Format {
    Png,
    Bmp,
    Elf,
    Gif,
    Pdf,
    Jpg,
    Pe,
    Unknown,
}

pub fn parse<T: std::io::Read + std::io::Seek>(fd: &mut T) -> Result<Format, Error> {

    use std::io::SeekFrom;

    const BYTES_READ: usize = 16;

    let mut bytes = [0u8; BYTES_READ];
    fd.seek(SeekFrom::Start(0))
        .map_err(|_| Problem::Msg(format!("Could not seek file")))?;
    fd.read_exact(&mut bytes)
        .map_err(|e| Problem::Msg(format!("File should be atleast {} bytes long: {}", 16 , e)))?;
    fd.seek(SeekFrom::Start(0))
        .map_err(|_| Problem::Msg(format!("Could not seek file")))?;

    check_magic(&mut bytes)

}

fn check_magic(magic: &[u8; MAGIC_SIZE]) -> Result<Format, Error> {

    if &magic[0..png::PNG_HEADER_SIZE] == png::PNG_HEADER {
        Ok(Format::Png)
    }
    else if &magic[0..bmp::BMP_MAGIC_SIZE] == bmp::BMP_MAGIC {
        Ok(Format::Bmp)
    }
    else if &magic[0..elf::ELF_MAGIC_SIZE] == elf::ELF_MAGIC {
        Ok(Format::Elf)
    }
    else if &magic[0..gif::GIF_MAGIC_SIZE] == gif::GIF87A_MAGIC
         || &magic[0..gif::GIF_MAGIC_SIZE] == gif::GIF89A_MAGIC {
             Ok(Format::Gif)
        }
    else if &magic[0..pdf::PDF_MAGIC_SIZE] == pdf::PDF_MAGIC {
        Ok(Format::Pdf)
    }
    else if &magic[0..jpg::JPG_MAGIC_SIZE] == jpg::JPG_MAGIC
        || &magic[0..jpg::JPG_MAGIC_SIZE] == jpg::JPG_MAGIC_2 {
            Ok(Format::Jpg)
        }
    else if &magic[0..pe::PE_MAGIC_SIZE] == pe::PE_MAGIC {
        Ok(Format::Pe)
    }
    else {
        Ok(Format::Unknown)
    }

}
