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
    else {
        Ok(Format::Unknown)
    }

}
