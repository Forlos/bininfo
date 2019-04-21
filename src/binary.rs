use crate::Opt;
use crate::magic::{self, Format};
use crate::formats::*;

use failure::{
    Error,
};

// #[derive(Debug)]
pub enum Binary {
    Png(png::Png),
    Bmp(bmp::Bmp),
    Gif(gif::Gif),
    Jpg(jpg::Jpg),

    Pe(pe::Pe),
    Elf(elf::Elf),
    JavaClass(javaclass::JavaClass),
    MachO(macho::MachO),
    Lua(lua::Lua),

    Xp3(xp3::Xp3),
    Zip(zip::Zip),

    Pdf(pdf::Pdf),

    Unknown,
}

impl Binary {

    pub fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        use std::io::Cursor;

        match magic::parse(&mut Cursor::new(&buf))? {
            Format::Png => Ok(Binary::Png(png::Png::parse(opt, buf)?)),
            Format::Bmp => Ok(Binary::Bmp(bmp::Bmp::parse(opt, buf)?)),
            Format::Gif => Ok(Binary::Gif(gif::Gif::parse(opt, buf)?)),
            Format::Jpg => Ok(Binary::Jpg(jpg::Jpg::parse(opt, buf)?)),

            Format::Pe => Ok(Binary::Pe(pe::Pe::parse(opt, buf)?)),
            Format::Elf => Ok(Binary::Elf(elf::Elf::parse(opt, buf)?)),
            Format::JavaClass => Ok(Binary::JavaClass(javaclass::JavaClass::parse(opt, buf)?)),
            Format::MachO => Ok(Binary::MachO(macho::MachO::parse(opt, buf)?)),
            Format::Lua => Ok(Binary::Lua(lua::Lua::parse(opt, buf)?)),

            Format::Xp3 => Ok(Binary::Xp3(xp3::Xp3::parse(opt, buf)?)),
            Format::Zip => Ok(Binary::Zip(zip::Zip::parse(opt, buf)?)),

            Format::Pdf => Ok(Binary::Pdf(pdf::Pdf::parse(opt, buf)?)),

            Format::Unknown => Ok(Binary::Unknown),
        }

    }

}
