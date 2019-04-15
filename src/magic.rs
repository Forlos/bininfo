use crate::formats::*;
use crate::Problem;

use failure::{
    Error,
};

const MAGIC_SIZE: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub enum Format {
    Png,
    Bmp,
    Elf,
    Gif,
    Pdf,
    Jpg,
    Pe,
    JavaClass,
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
    else if &magic[0..javaclass::CLASS_MAGIC_SIZE] == javaclass::CLASS_MAGIC {
        Ok(Format::JavaClass)
    }
    else {
        Ok(Format::Unknown)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_parsing() -> Result<(), Error> {
        use std::io::Cursor;

        assert!(&vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,][0..png::PNG_HEADER_SIZE] == png::PNG_HEADER);
        assert!(&vec![0x42, 0x4D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,][0..bmp::BMP_MAGIC_SIZE] == bmp::BMP_MAGIC);
        assert!(&vec![0x7F, 0x45, 0x4C, 0x46, 0x00, 0x00, 0x00, 0x00,][0..elf::ELF_MAGIC_SIZE] == elf::ELF_MAGIC);
        assert!(&vec![0x47, 0x49, 0x46, 0x38, 0x37, 0x61, 0x00, 0x00,][0..gif::GIF_MAGIC_SIZE] == gif::GIF87A_MAGIC);
        assert!(&vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x00, 0x00,][0..gif::GIF_MAGIC_SIZE] == gif::GIF89A_MAGIC);
        assert!(&vec![0x25, 0x50, 0x44, 0x46, 0x2D, 0x00, 0x00, 0x00,][0..pdf::PDF_MAGIC_SIZE] == pdf::PDF_MAGIC);
        assert!(&vec![0xFF, 0xD8, 0xFF, 0xDB, 0x00, 0x00, 0x00, 0x00,][0..jpg::JPG_MAGIC_SIZE] == jpg::JPG_MAGIC);
        assert!(&vec![0xFF, 0xD8, 0xFF, 0xEE, 0x00, 0x00, 0x00, 0x00,][0..jpg::JPG_MAGIC_SIZE] == jpg::JPG_MAGIC_2);
        assert!(&vec![0x4D, 0x5A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,][0..pe::PE_MAGIC_SIZE] == pe::PE_MAGIC);

        let magic = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                         0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,];
        assert_eq!(parse(&mut Cursor::new(&magic))?, Format::Unknown);

        Ok(())

    }

}
