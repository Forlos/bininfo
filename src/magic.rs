use crate::formats::*;

const MAGIC_SIZE: usize = 16;

pub enum Format {
    Png,
    Bmp,
    Unknown,
}

pub fn parse<T: std::io::Read + std::io::Seek>(fd: &mut T) -> std::io::Result<Format> {

    use std::io::SeekFrom;

    const BYTES_READ: usize = 16;

    let mut bytes = [0u8; BYTES_READ];
    fd.seek(SeekFrom::Start(0))?;
    fd.read_exact(&mut bytes)?;

    fd.seek(SeekFrom::Start(0))?;
    check_magic(&mut bytes)

}

fn check_magic(magic: &[u8; MAGIC_SIZE]) -> std::io::Result<Format> {

    if &magic[0..png::PNG_HEADER_SIZE] == png::PNG_HEADER {
        Ok(Format::Png)
    }
    else if &magic[0..bmp::BMP_MAGIC_SIZE] == bmp::BMP_MAGIC {
        Ok(Format::Bmp)
    }
    else {
        Ok(Format::Unknown)
    }

}
