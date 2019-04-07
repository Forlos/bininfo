use scroll::{self, Pread, ctx};

use crate::Opt;
use crate::Problem;
use failure::{Error};

pub const PE_MAGIC: &'static [u8; PE_MAGIC_SIZE] = b"MZ";
pub const PE_MAGIC_SIZE: usize = 2;

const PE_SIGNATURE_OFFSET: usize = 0x3C;
const PE_SIGNATURE: &'static [u8; 4] = b"PE\x00\x00";

const PE32_MAGIC: u16 = 0x10b;
const PE32PLUS_MAGIC: u16 = 0x20b;

#[allow(non_camel_case_types)]
#[derive(Pread, Debug)]
struct COFF_header {
    machine:           u16,
    n_of_sections:     u16,
    timedate_stamp:    u32,
    pointer_to_symtab: u32,
    n_of_symtab:       u32,
    sz_of_opt_header:  u16,
    characteristics:   u16,
}

#[allow(non_camel_case_types)]
#[derive(Pread, Debug)]
struct Std_COFF_header {
    magic:          u16,
    major_link_ver: u8,
    minor_link_ver: u8,
    sz_of_code:     u32,
    sz_of_init:     u32,
    sz_of_uninit:   u32,
    addr_of_entry:  u32,
    base_of_code:   u32,
}

#[derive(Debug)]
pub struct Pe {
    opt: Opt,

    coff: COFF_header,
    std_coff: Std_COFF_header,
}

impl super::FileFormat for Pe {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        let pe_sig = buf.pread::<u32>(PE_SIGNATURE_OFFSET)? as usize;

        if &buf[pe_sig..pe_sig + 4] != PE_SIGNATURE {
            return Err(Error::from(Problem::Msg(format!("Invalid PE signature"))));
        }

        let coff = buf.pread_with(pe_sig + 4, scroll::LE)?;
        let std_coff = buf.pread_with(pe_sig + 24, scroll::LE)?;

        Ok(Pe {
            opt,

            coff,
            std_coff,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("PE");
        println!("{:#X?}", self);

        Ok(())
    }


}
