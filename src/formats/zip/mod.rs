#![allow(non_camel_case_types)]
use crate::Opt;
use failure::{Error};

pub const ZIP_MAGIC:         &'static [u8; ZIP_MAGIC_SIZE] = b"PK\x03\x04";
pub const ZIP_MAGIC_EMPTY:   &'static [u8; ZIP_MAGIC_SIZE] = b"PK\x05\x06";
pub const ZIP_MAGIC_SPANNED: &'static [u8; ZIP_MAGIC_SIZE] = b"PK\x07\x08";
pub const ZIP_MAGIC_SIZE: usize = 4;

#[derive(Debug, Pread)]
struct Local_file_header {
    signature:     u32,
    version:       u16,
    /// Bit 0: If set, indicates that the file is encrypted.
    flag:          u16,
    compr_method:  u16,
    last_mod_time: u16,
    last_mod_date: u16,
    crc32:         u32,
    compr_sz:      u32,
    uncompr_sz:    u32,
    name_sz:       u16,
    extra_sz:      u16,
}

#[derive(Debug)]
struct Local_file {
    header: Local_file_header,
    name:   String,
    extra:  Vec<u8>,
}

/// This descriptor MUST exist if bit 3 of the general purpose bit flag is set (see below).
struct Data_descriptior {
    crc32:      u32,
    compr_sz:   u32,
    uncompr_sz: u32,
}

#[derive(Debug)]
struct Archive_extra_data {
    signature: u32,
    length:    u32,
    data:      Vec<u8>,
}

#[derive(Debug, Pread)]
struct Central_dir_header {
    signature:        u32,
    version_made:     u16,
    version_needed:   u16,
    flag:             u16,
    compr_method:     u16,
    last_mod_time:    u16,
    last_mod_date:    u16,
    crc32:            u32,
    compr_sz:         u32,
    uncompr_sz:       u32,
    name_sz:          u16,
    extra_sz:         u16,
    comment_sz:       u16,
    disk_num_start:   u16,
    internal_attr:    u16,
    external_attr:    u32,
    local_header_off: u32,
}

#[derive(Debug)]
struct Central_dir {
    header:  Central_dir_header,
    name:    String,
    extra:   Vec<u8>,
    comment: Vec<u8>,
}

#[derive(Debug)]
struct Digital_signature {
    signature:      u32,
    data_sz:        u16,
    signature_data: Vec<u8>,
}

#[derive(Debug, Pread)]
struct End_central_dir_header {
    signature:         u32,
    n_this_disk:       u16,
    n_start_disk:      u16,
    n_this_entries:    u16,
    n_central_entries: u16,
    central_sz:        u32,
    offset:            u32,
    zip_comment_sz:    u16,
}

#[derive(Debug)]
struct End_central_dir {
    header:  End_central_dir_header,
    comment: Vec<u8>,
}

#[derive(Debug, Pread)]
/// Zip64 end of central dir
struct Zip64_End_central_dir_header {
    signature:         u32,
    size:              u64,
    version_made:      u16,
    version_needed:    u16,
    n_this_disk:       u32,
    n_start_disk:      u32,
    n_this_entries:    u64,
    n_central_entries: u64,
    central_dir:       u64,
    central_dir_sz:    u64,
    start_disk_num:    u64,
}

#[derive(Debug)]
struct Zip64_End_central_dir {
    header: Zip64_End_central_dir_header,
    data:   Vec<u8>,
}

#[derive(Debug, Pread)]
/// zip64 end of central dir locator
struct Zip64_End_central_locator {
    signature:       u32,
    start_disk:      u32,
    end_central_dir: u64,
    n_disks:         u32,
}

// If one of the fields in the end of central directory record is too small to hold required data,
// the field SHOULD be set to -1 (0xFFFF or 0xFFFFFFFF) and the ZIP64 format record SHOULD be created.
#[derive(Debug)]
pub struct Zip {
    opt: Opt,
}

impl super::FileFormat for Zip {
    type Item = Self;

    fn parse(opt: Opt, _buf: &[u8]) -> Result<Self, Error> {

        Ok(Zip {
            opt,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("ZIP");
        println!("{:#X?}", self);

        Ok(())
    }


}
