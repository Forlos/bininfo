#![allow(non_camel_case_types)]
use scroll::{self, Pread};

use crate::Opt;
use crate::Problem;
use failure::{Error};

pub const PE_MAGIC: &'static [u8; PE_MAGIC_SIZE] = b"MZ";
pub const PE_MAGIC_SIZE: usize = 2;

const PE_SIGNATURE_OFFSET: usize = 0x3C;
const PE_SIGNATURE: &'static [u8; 4] = b"PE\x00\x00";

const PE32_MAGIC: u16 = 0x10b;
const PE32PLUS_MAGIC: u16 = 0x20b;

const COFF_HEADER_OFFSER: usize = 4;
const STD_COFF_HEADER_OFFSET: usize = 24;

const PE32PLUS_WIN_FIELDS_OFFSET: usize = 24 + 24;
const PE32PLUS_DATA_DIRS_OFFSET:  usize = 24 + 112;
const PE32PLUS_SECTIONS_OFFSET:   usize = 24 + 240;

const PE32_WIN_FIELDS_OFFSET: usize = 24 + 28;
const PE32_DATA_DIRS_OFFSET:  usize = 24 + 96;
const PE32_SECTIONS_OFFSET:   usize = 24 + 224;

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
    base_of_data:   u32,
}

#[derive(Pread, Debug)]
struct Windows_fields_32 {
    image_base:       u32,
    section_align:    u32,
    file_align:       u32,
    major_os_ver:     u16,
    minor_os_ver:     u16,
    major_img_ver:    u16,
    minor_img_ver:    u16,
    major_sub_ver:    u16,
    minor_sub_ver:    u16,
    win32_ver:        u32,
    sz_of_img:        u32,
    sz_of_headers:    u32,
    checksum:         u32,
    subsys:           u16,
    dll_chara:        u16,
    sz_stack_reserve: u32,
    sz_stack_commit:  u32,
    sz_heap_reserve:  u32,
    sz_heap_commit:   u32,
    loader_flags:     u32,
    n_of_rva:         u32,
}

#[derive(Pread, Debug)]
struct Windows_fields {
    image_base:       u64,
    section_align:    u32,
    file_align:       u32,
    major_os_ver:     u16,
    minor_os_ver:     u16,
    major_img_ver:    u16,
    minor_img_ver:    u16,
    major_sub_ver:    u16,
    minor_sub_ver:    u16,
    win32_ver:        u32,
    sz_of_img:        u32,
    sz_of_headers:    u32,
    checksum:         u32,
    subsys:           u16,
    dll_chara:        u16,
    sz_stack_reserve: u64,
    sz_stack_commit:  u64,
    sz_heap_reserve:  u64,
    sz_heap_commit:   u64,
    loader_flags:     u32,
    n_of_rva:         u32,
}

impl From<Windows_fields_32> for Windows_fields {

    fn from(header: Windows_fields_32) -> Windows_fields {
        Windows_fields {
            image_base:       header.image_base as u64,
            section_align:    header.section_align,
            file_align:       header.file_align,
            major_os_ver:     header.major_os_ver,
            minor_os_ver:     header.minor_os_ver,
            major_img_ver:    header.major_img_ver,
            minor_img_ver:    header.minor_img_ver,
            major_sub_ver:    header.major_sub_ver,
            minor_sub_ver:    header.minor_sub_ver,
            win32_ver:        header.win32_ver,
            sz_of_img:        header.sz_of_img,
            sz_of_headers:    header.sz_of_headers,
            checksum:         header.checksum,
            subsys:           header.subsys,
            dll_chara:        header.dll_chara,
            sz_stack_reserve: header.sz_stack_reserve as u64,
            sz_stack_commit:  header.sz_stack_commit as u64,
            sz_heap_reserve:  header.sz_heap_reserve as u64,
            sz_heap_commit:   header.sz_heap_commit as u64,
            loader_flags:     header.loader_flags,
            n_of_rva:         header.n_of_rva,
        }
    }

}

#[derive(Pread, Debug)]
struct Image_data_dir {
    rva: u32,
    sz:  u32,
}

#[derive(Pread, Debug)]
struct Data_dirs {
    export_tab:   Image_data_dir,
    import_tab:   Image_data_dir,
    resource_tab: Image_data_dir,
    except_tab:   Image_data_dir,
    cert_tab:     Image_data_dir,
    basrel_tab:   Image_data_dir,
    debug_tab:    Image_data_dir,
    arch_tab:     Image_data_dir,
    gloptr_tab:   Image_data_dir,
    tls_tab:      Image_data_dir,
    loadcfg_tab:  Image_data_dir,
    bound_imp:    Image_data_dir,
    iat:          Image_data_dir,
    // Delay Import Descriptor
    did:          Image_data_dir,
    clr_runtime:  Image_data_dir,
    reserved:     Image_data_dir,
}

#[derive(Pread, Debug)]
struct COFF_optional_header {
    std_coff: Std_COFF_header,
    win_fields:  Windows_fields,
    data_dirs:       Data_dirs,
}

#[derive(Pread, Debug)]
struct Section_table {
    name: [u8; 8],
    virt_sz: u32,
    virt_addr: u32,
    sz_raw_data: u32,
    ptr_raw_data: u32,
    ptr_relocs: u32,
    ptr_linenum: u32,
    n_relocs: u16,
    n_linenum: u16,
    characteristics: u32,
}

#[derive(Pread, Debug)]
struct COFF_reloc {
    virt_addr:  u32,
    symtab_idx: u32,
    c_type:     u16,
}

#[derive(Pread, Debug)]
struct COFF_linenum {
    idx:      u32,
    line_num: u16,
}

#[derive(Pread, Debug)]
struct COFF_symbol_table {
    name:          [u8; 8],
    value:         u32,
    sec_num:       u16,
    c_type:        u16,
    storage_class: u8,
    n_aux_sym:     u8,
}

// #[derive(Debug)]
// struct COFF_string_table {
//     size: u32,
//     text: String,
// }

#[derive(Pread, Debug)]
struct Aux_sym_record_1 {
    tag_index:   u32,
    total_sz:    u32,
    ptr_linenum: u32,
    ptr_nextfn:  u32,
    unused:      u16,
}

#[derive(Debug)]
pub struct Pe {
    opt:        Opt,

    coff:       COFF_header,
    coff_optional_header: Option<COFF_optional_header>,

    sections:   Vec<Section_table>,
}

impl super::FileFormat for Pe {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        let pe_sig = buf.pread::<u32>(PE_SIGNATURE_OFFSET)? as usize;

        if &buf[pe_sig..pe_sig + 4] != PE_SIGNATURE {
            return Err(Error::from(Problem::Msg(format!("Invalid PE signature"))));
        }

        let coff = buf.pread_with::<COFF_header>(pe_sig + COFF_HEADER_OFFSER, scroll::LE)?;
        let mut coff_optional_header = None;
        let win_fields;
        let data_dirs;

        let mut sections = Vec::with_capacity(coff.n_of_sections as usize);
        if coff.sz_of_opt_header > 0 {
            let mut std_coff = buf.pread_with::<Std_COFF_header>(pe_sig + STD_COFF_HEADER_OFFSET, scroll::LE)?;

            if std_coff.magic == PE32PLUS_MAGIC {
                std_coff.base_of_data = 0;
                win_fields = buf.pread_with(pe_sig + PE32PLUS_WIN_FIELDS_OFFSET, scroll::LE)?;
                data_dirs = buf.pread_with(pe_sig + PE32PLUS_DATA_DIRS_OFFSET, scroll::LE)?;

                for i in 0..coff.n_of_sections as usize {
                    sections.push(buf.pread_with(pe_sig + PE32PLUS_SECTIONS_OFFSET + i * 40, scroll::LE)?);
                }
            }
            else {
                win_fields = Windows_fields::from(buf.pread_with::<Windows_fields_32>(pe_sig + PE32_WIN_FIELDS_OFFSET, scroll::LE)?);
                data_dirs = buf.pread_with(pe_sig + PE32_DATA_DIRS_OFFSET, scroll::LE)?;

                for i in 0..coff.n_of_sections as usize {
                    sections.push(buf.pread_with(pe_sig + PE32_SECTIONS_OFFSET + i * 40, scroll::LE)?);
                }
            }
            coff_optional_header = Some(COFF_optional_header {
                std_coff,
                win_fields,
                data_dirs,
            });
        }



        Ok(Pe {
            opt,

            coff,
            coff_optional_header,

            sections,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("PE");
        println!("{:#X?}", self);

        Ok(())
    }


}
