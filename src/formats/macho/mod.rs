#![allow(non_camel_case_types)]
include!("macho_constants.rs");
use failure::{Error};
use scroll::{self, Pread};

use crate::Opt;
// use crate::Problem;
use crate::format::{fmt_macho};

pub const MACHO_MAGIC_32: &'static [u8; MACHO_MAGIC_SIZE] = b"\xFE\xED\xFA\xCE";
pub const MACHO_MAGIC_64: &'static [u8; MACHO_MAGIC_SIZE] = b"\xFE\xED\xFA\xCF";
pub const MACHO_MAGIC_32_R: &'static [u8; MACHO_MAGIC_SIZE] = b"\xCE\xFA\xED\xFE";
pub const MACHO_MAGIC_64_R: &'static [u8; MACHO_MAGIC_SIZE] = b"\xCF\xFA\xED\xFE";
pub const MACHO_MAGIC_SIZE: usize = 4;

// pub const MACHO_FAT_MAGIC: &'static [u8; MACHO_MAGIC_SIZE] = b"\xCA\xFE\xBA\xBE";

#[derive(Debug, Pread)]
pub struct Mach_header {
    magic:       u32,
    pub cputype:     u32,
    pub cpusubtype:  u32,
    pub filetype:    u32,
    n_cmds:      u32,
    size_of_cmd: u32,
    pub flags:       u32,
}

#[derive(Debug)]
pub struct MachO {
    opt: Opt,

    header: Mach_header,
}

impl super::FileFormat for MachO {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {
        // const FAT_MAGIC: u32 = 0xCAFEBABE;

        const	MH_MAGIC: u32 = 0xFEEDFACE;
        const MH_CIGAM: u32 = 0xCEFAEDFE;

        const MH_MAGIC_64: u32 =  0xFEEDFACF;
        const MH_CIGAM_64: u32 =  0xCFFAEDFE;

        let endianess = match buf.pread::<u32>(0)? {
            MH_MAGIC | MH_MAGIC_64 => scroll::LE,
            MH_CIGAM | MH_CIGAM_64 => scroll::BE,
            // This should never happen as header have been already checked
            _ => unreachable!(),
        };

        let offset = &mut 0;
        let header = buf.gread_with(offset, endianess)?;

        Ok(MachO {
            opt,

            header,
        })

    }

    fn print(&self) -> Result<(), Error> {

        fmt_macho(&self.header);
        println!("{:#X?}", self);

        Ok(())
    }


}
