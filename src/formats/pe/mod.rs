#![allow(non_camel_case_types)]

include!("pe_constants.rs");
use ansi_term::Color;
use scroll::{self, Pread};


use crate::Opt;
use crate::Problem;
use failure::{Error};
use crate::format::{fmt_indentln, fmt_pe};

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
pub struct COFF_header {
    pub machine:           u16,
    pub n_of_sections:     u16,
    pub timedate_stamp:    u32,
    pub pointer_to_symtab: u32,
    pub n_of_symtab:       u32,
    pub sz_of_opt_header:  u16,
    pub characteristics:   u16,
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

#[derive(Debug)]
struct Data_dirs {
    dirs: Vec<Image_data_dir>,
}

#[derive(Debug)]
struct COFF_optional_header {
    std_coff:   Std_COFF_header,
    win_fields: Windows_fields,
    data_dirs:  Data_dirs,
}

#[derive(Pread, Debug)]
struct Section_table {
    name:            [u8; 8],
    virt_sz:         u32,
    virt_addr:       u32,
    sz_raw_data:     u32,
    ptr_raw_data:    u32,
    ptr_relocs:      u32,
    ptr_linenum:     u32,
    n_relocs:        u16,
    n_linenum:       u16,
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
struct Attr_cert_table {
    dw_length:   u32,
    w_revision:  u16,
    w_cert_type: u16,
    b_cert:      Vec<u8>,
}

#[derive(Debug, Pread)]
struct Delay_import_table {
    attr:            u32,
    name:            u32,
    mod_handle:      u32,
    /// Delay Import Address Table
    delay_iat:       u32,
    /// Delay Import Name Table
    delay_int:       u32,
    /// Bound Delay Import Table
    bound_delay_it:  u32,
    /// Unload Delay Import Table
    unload_delay_it: u32,
    time_stamp:      u32,
}

#[derive(Pread, Debug)]
struct Debug_dir {
    characteristics: u32,
    timedate_stamp:  u16,
    major_ver:       u16,
    minor_ver:       u16,
    d_type:          u32,
    sz_data:         u32,
    addr_data:       u32,
    ptr_data:        u32,
}

#[derive(Debug)]
struct Export_dir {
    header:        Export_dir_table,
    func_addr:     Vec<u32>,
    func_names:    Vec<String>,
    func_ordinals: Vec<u16>,
    funcs:         Vec<Export_func>,
}

#[derive(Debug)]
struct Export_func {
    addr: u32,
    name: String,
    ordinal: u16,
}


#[derive(Pread, Debug)]
struct Export_dir_table {
    export_flags:        u32,
    timedate_stamp:      u32,
    major_ver:           u16,
    minor_ver:           u16,
    name_rva:            u32,
    ord_base:            u32,
    addr_tab_entries:    u32,
    n_name_ptr:          u32,
    export_addr_tab_rva: u32,
    name_ptr_rva:        u32,
    ord_tab_rva:         u32,
}

#[derive(Pread, Debug)]
struct Export_addr_table {
    export_rva: u32,
    forwarder_rva: u32,
}

#[derive(Debug)]
struct Import_dir {
    header:   Import_dir_table,
    name:     String,
    entries:  Vec<String>,
    ordinals: Vec<u16>,
}

#[derive(Pread, Debug)]
struct Import_dir_table {
    import_lkup_tab_rva: u32,
    timedate_stamp:      u32,
    forwarder_chain:     u32,
    name_rva:            u32,
    import_addr_tab_rva: u32,
}

impl Import_dir_table {
    fn is_null(&self) -> bool {
        self.import_addr_tab_rva == 0 && self.timedate_stamp == 0 &&
            self.forwarder_chain == 0 && self.name_rva == 0 && self.import_addr_tab_rva == 0
    }
}

#[derive(Pread, Debug)]
struct Tls_dir_32 {
    data_start_rva:  u32,
    data_end_rva:    u32,
    idx_addr:        u32,
    callback_addr:   u32,
    sz_zero_fill:    u32,
    characteristics: u32,
}

#[derive(Pread, Debug)]
struct Tls_dir {
    data_start_rva:  u64,
    data_end_rva:    u64,
    idx_addr:        u64,
    callback_addr:   u64,
    sz_zero_fill:    u32,
    characteristics: u32,
}

#[derive(Pread, Debug)]
struct Rsrc_dir_tab {
    characteristics: u32,
    timedate_stamp:  u32,
    major_ver:       u16,
    minor_ver:       u16,
    n_name_entries:  u16,
    n_id_entries:    u16,
}

#[derive(Pread, Debug)]
struct Rsrc_dir_entry {
    name_id: u32,
    data_subdir: u32,
}

#[derive(Pread, Debug)]
struct Rsrc_data_entry {
    data_rva: u32,
    size:     u32,
    codepage: u32,
    reserved: u32,
}

#[derive(Debug)]
pub struct Pe {
    opt:                  Opt,

    coff:                 COFF_header,
    coff_optional_header: Option<COFF_optional_header>,

    sections:             Vec<Section_table>,

    exports:              Option<Export_dir>,
    imports:              Vec<Import_dir>,
    resources:            Option<Rsrc_dir_tab>,
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

        let mut exports   = None;
        let mut imports   = Vec::new();
        let mut resources = None;

        let mut sections: Vec<Section_table> = Vec::with_capacity(coff.n_of_sections as usize);

        if coff.sz_of_opt_header > 0 {
            let mut std_coff = buf.pread_with::<Std_COFF_header>(pe_sig + STD_COFF_HEADER_OFFSET, scroll::LE)?;

            if std_coff.magic == PE32PLUS_MAGIC {
                std_coff.base_of_data = 0;
                win_fields = buf.pread_with(pe_sig + PE32PLUS_WIN_FIELDS_OFFSET, scroll::LE)?;
                let mut dirs = Vec::with_capacity(16);

                for i in 0..16 {
                    dirs.push(buf.pread_with(pe_sig + PE32PLUS_DATA_DIRS_OFFSET + i * 8, scroll::LE)?);
                }
                data_dirs = Data_dirs {
                    dirs,
                };

                for i in 0..coff.n_of_sections as usize {
                    sections.push(buf.pread_with(pe_sig + PE32PLUS_SECTIONS_OFFSET + i * 40, scroll::LE)?);
                }
            }
            else if std_coff.magic == PE32_MAGIC {
                win_fields = Windows_fields::from(buf.pread_with::<Windows_fields_32>(pe_sig + PE32_WIN_FIELDS_OFFSET, scroll::LE)?);
                let mut dirs = Vec::with_capacity(16);

                for i in 0..16 {
                    dirs.push(buf.pread_with(pe_sig + PE32_DATA_DIRS_OFFSET + i * 8, scroll::LE)?);
                }
                data_dirs = Data_dirs{
                    dirs,
                };

                for i in 0..coff.n_of_sections as usize {
                    sections.push(buf.pread_with(pe_sig + PE32_SECTIONS_OFFSET + i * 40, scroll::LE)?);
                }
            }
            else {
                return Err(Error::from(Problem::Msg(format!("Invalid PE magic"))));
            }
            coff_optional_header = Some(COFF_optional_header {
                std_coff,
                win_fields,
                data_dirs,
            });
        }

        if let Some(opt_header) = &coff_optional_header {

            for (i, dir) in opt_header.data_dirs.dirs.iter().enumerate() {
                if dir.rva != 0 && dir.sz != 0 {
                    match i {
                        0 => {
                            for sect in &sections {
                                if dir.rva >= sect.virt_addr && dir.rva < sect.virt_addr + sect.virt_sz {
                                    let header = buf.pread_with::<Export_dir_table>(sect.ptr_raw_data as usize + (dir.rva - sect.virt_addr) as usize, scroll::LE)?;
                                    let mut func_addr  = Vec::with_capacity(header.addr_tab_entries as usize);

                                    let addr_offset = &mut (sect.ptr_raw_data as usize + (header.export_addr_tab_rva - sect.virt_addr) as usize);
                                    for _ in 0..header.addr_tab_entries as usize {
                                        func_addr.push(buf.gread_with(addr_offset, scroll::LE)?);
                                    }
                                    let mut func_names = Vec::with_capacity(header.n_name_ptr as usize);

                                    let names_offset = &mut (sect.ptr_raw_data as usize + (header.name_ptr_rva - sect.virt_addr) as usize);
                                    for _ in 0..header.n_name_ptr as usize {
                                        let name_rva = sect.ptr_raw_data as usize + (buf.gread_with::<u32>(names_offset, scroll::LE)? - sect.virt_addr) as usize;
                                        //TODO do name demangling
                                        func_names.push(buf.pread::<&str>(name_rva)?.to_string());
                                    }
                                    let mut func_ordinals = Vec::with_capacity(header.n_name_ptr as usize);

                                    let ordinals_offset = &mut (sect.ptr_raw_data as usize + (header.ord_tab_rva - sect.virt_addr) as usize);
                                    for _ in 0..header.n_name_ptr as usize {
                                        func_ordinals.push(buf.gread_with(ordinals_offset, scroll::LE)?);
                                    }
                                    let mut funcs = Vec::with_capacity(header.addr_tab_entries as usize);

                                    // https://stackoverflow.com/questions/5653316/pe-export-directory-tables-ordinalbase-field-ignored
                                    for i in 0..header.n_name_ptr as usize {
                                        let ordinal =  func_ordinals[i] + header.ord_base as u16;
                                        funcs.push( Export_func { addr: func_addr[ordinal as usize - header.ord_base as usize] , name: func_names[i].clone(), ordinal } );
                                    }
                                    for i in funcs.len()..header.addr_tab_entries as usize {
                                        funcs.push( Export_func { addr: func_addr[i], name: "[NONAME]".to_owned(), ordinal: i as u16 } )
                                    }



                                    exports = Some(Export_dir {
                                        header,
                                        func_addr,
                                        func_names,
                                        func_ordinals,
                                        funcs,
                                    });
                                }
                            }
                        },
                        1 => {
                            for sect in &sections {
                                if dir.rva >= sect.virt_addr && dir.rva < sect.virt_addr + sect.virt_sz {
                                    let offset = &mut (sect.ptr_raw_data as usize + (dir.rva - sect.virt_addr) as usize);
                                    let mut header = buf.gread_with::<Import_dir_table>(offset, scroll::LE)?;
                                    while !header.is_null() {
                                        let entry_offset = &mut 0;
                                        // Original First Thunk
                                        if header.import_lkup_tab_rva != 0 {
                                            *entry_offset += sect.ptr_raw_data as usize + (header.import_lkup_tab_rva - sect.virt_addr) as usize;
                                        }
                                        // First Thunk
                                        else {
                                            *entry_offset += sect.ptr_raw_data as usize + (header.import_addr_tab_rva - sect.virt_addr) as usize;
                                        }
                                        // 64bit
                                        if opt_header.std_coff.magic == PE32PLUS_MAGIC {
                                            let mut entry = buf.gread_with::<u64>(entry_offset, scroll::LE)?;
                                            let mut entries = Vec::new();
                                            let mut ordinals = Vec::new();
                                            while entry != 0 {
                                                if entry & 0x8000000000000000 != 0 {
                                                    // by ordinal
                                                    ordinals.push((entry & 0x0000ffff) as u16);
                                                }
                                                else {
                                                    entry = sect.ptr_raw_data as u64 + (entry - sect.virt_addr as u64);
                                                    // by name
                                                    let name = buf.pread::<&str>(entry as usize + 2)?.to_string();
                                                    entries.push(name);
                                                }
                                                entry = buf.gread_with::<u64>(entry_offset, scroll::LE)?;
                                            }
                                            let name = buf.pread::<&str>(sect.ptr_raw_data as usize + (header.name_rva - sect.virt_addr) as usize)?.to_string();
                                            imports.push( Import_dir { header, name, entries, ordinals });
                                        }

                                        // 32bit
                                        else {
                                            let mut entry = buf.gread_with::<u32>(entry_offset, scroll::LE)?;
                                            println!("Offset: {:#X}", *entry_offset);
                                            let mut entries = Vec::new();
                                            let mut ordinals = Vec::new();
                                            while entry != 0 {
                                                if entry & 0x80000000 != 0 {
                                                    // by ordinal
                                                    ordinals.push((entry & 0x0000ffff) as u16);
                                                }
                                                else {
                                                    entry = sect.ptr_raw_data + (entry - sect.virt_addr);
                                                    // by name
                                                    let name = buf.pread::<&str>(entry as usize + 2)?.to_string();
                                                    entries.push(name);
                                                }
                                                entry = buf.gread_with::<u32>(entry_offset, scroll::LE)?;
                                            }
                                            let name = buf.pread::<&str>(sect.ptr_raw_data as usize + (header.name_rva - sect.virt_addr) as usize)?.to_string();
                                            imports.push( Import_dir { header, name, entries, ordinals });
                                        }
                                        header = buf.gread_with::<Import_dir_table>(offset, scroll::LE)?;
                                    }
                                }
                            }
                        },
                        2  => {
                            for sect in &sections {
                                if dir.rva >= sect.virt_addr && dir.rva < sect.virt_addr + sect.virt_sz {
                                    resources = Some(buf.pread_with(sect.ptr_raw_data as usize + (dir.rva - sect.virt_addr) as usize, scroll::LE)?);
                                }
                            }
                        },
                        _ => (),
                    }
                }
            }

        }

        Ok(Pe {
            opt,

            coff,
            coff_optional_header,

            sections,

            exports,
            imports,
            resources,
        })

    }

    fn print(&self) -> Result<(), Error> {
        use prettytable::Table;
        // println!("{:#X?}", self);

        //
        // PE HEADER
        //
        fmt_pe(&self.coff);

        if let Some(opt) = &self.coff_optional_header {
            //
            // OPTIONAL HEADER
            //
            println!("{}", Color::White.underline().paint("Optional Header"));
            fmt_indentln(format!("Major linker version: {}, Minor linker version: {}",
                     opt.std_coff.major_link_ver,
                     opt.std_coff.minor_link_ver));
            fmt_indentln(format!("Size of code: {}", Color::Green.paint(format!("{:#X}", opt.std_coff.sz_of_code))));
            fmt_indentln(format!("Size of initialized: {}, Size of uninitialized: {}",
                                 Color::Green.paint(format!("{:#X}", opt.std_coff.sz_of_init)),
                                 Color::Green.paint(format!("{:#X}", opt.std_coff.sz_of_uninit))));
            fmt_indentln(format!("Entry point: {}", Color::Red.paint(format!("{:#X}", opt.std_coff.addr_of_entry))));
            fmt_indentln(format!("Base of code: {}", Color::Red.paint(format!("{:#X}", opt.std_coff.base_of_code))));
            if opt.std_coff.magic == PE32_MAGIC {
                fmt_indentln(format!("Base of data: {}", Color::Red.paint(format!("{:#X}", opt.std_coff.base_of_data))));
            }

            //
            // WINDOWS FIELDS
            //
            println!();
            println!("{}", Color::White.underline().paint("Windows Fields"));
            fmt_indentln(format!("Image base: {}", Color::Red.paint(format!("{:#X}", opt.win_fields.image_base))));
            fmt_indentln(format!("Section alignment: {:#X}, File alignment: {:#X}",
                                 opt.win_fields.section_align,
                                 opt.win_fields.file_align));
            fmt_indentln(format!("Major OS version: {}, Minor OS version: {}",
                                 opt.win_fields.major_os_ver,
                                 opt.win_fields.minor_os_ver));
            fmt_indentln(format!("Major image version: {}, Minor image version: {}",
                                 opt.win_fields.major_img_ver,
                                 opt.win_fields.minor_img_ver));
            fmt_indentln(format!("Major subsys version: {}, Minor subsys version: {}",
                                 opt.win_fields.major_sub_ver,
                                 opt.win_fields.minor_sub_ver));
            fmt_indentln(format!("Size of image: {}, Size of headers: {}",
                                 Color::Green.paint(format!("{:#X}", opt.win_fields.sz_of_img)),
                                 Color::Green.paint(format!("{:#X}", opt.win_fields.sz_of_headers))));
            fmt_indentln(format!("Checksum: {:#X}", opt.win_fields.checksum));
            fmt_indentln(format!("Subsystem: {}", Color::White.paint(subsys_to_str(opt.win_fields.subsys))));
            fmt_indentln(format!("DLL Characteristics: {}", Color::White.paint(dllchara_to_str(opt.win_fields.dll_chara))));
            fmt_indentln(format!("Size of stack commit: {}, Size of stack reserve: {}",
                                 Color::Green.paint(format!("{:#X}", opt.win_fields.sz_stack_commit)),
                                 Color::Green.paint(format!("{:#X}", opt.win_fields.sz_stack_reserve))));
            fmt_indentln(format!("Size of heap commit: {}, Size of heap reserve: {}",
                                 Color::Green.paint(format!("{:#X}", opt.win_fields.sz_heap_commit)),
                                 Color::Green.paint(format!("{:#X}", opt.win_fields.sz_heap_reserve))));
            fmt_indentln(format!("Loader flags: {:#X}", opt.win_fields.loader_flags));
            fmt_indentln(format!("Number of Rva and Sizes: {}",
                                 Color::Purple.paint(format!("{:#X}", opt.win_fields.n_of_rva))));

            //
            // DATA DIRECTORIES
            //
            println!();
            println!("{}", Color::White.underline().paint("Data Directories"));
            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![r->"Idx", "Name", "VirtAddr", "Size"]);
            for (i, data) in opt.data_dirs.dirs.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i,
                    DATA_DIRS[i],
                    Fr->format!("{:#X}", data.rva),
                    Fg->format!("{:#X}", data.sz),
                ]);
            }

            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }

            println!();
        }

        //
        // SECTIONS
        //
        if self.sections.len() > 0 {
            println!("{}({})",
                     Color::White.underline().paint("Sections"),
                     self.sections.len());

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![r->"Idx", "Name", "VirtSz", "VirtAddr", "SzRawData",
            "PtrRawData", "PtrRelocs", "PtrLineNum", "nRelocs", "nLinenum", "Characteristics"]);

            for (i, sec) in self.sections.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i,
                    std::str::from_utf8(&sec.name)?,
                    Fg->format!("{:#X}",sec.virt_sz),
                    Fr->format!("{:#X}",sec.virt_addr),
                    Fg->format!("{:#X}",sec.sz_raw_data),
                    Fr->format!("{:#X}",sec.ptr_raw_data),
                    Fr->format!("{:#X}",sec.ptr_relocs),
                    Fr->format!("{:#X}",sec.ptr_linenum),
                    Fm->format!("{:#X}",sec.n_relocs),
                    Fm->format!("{:#X}",sec.n_linenum),
                    section_chara_to_str(sec.characteristics),
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }

        //
        // IMPORTS
        //
        if self.imports.len() >= 1 {
            let sum = self.imports.iter().fold(0, |sum, i| {
                sum + i.entries.len() + i.ordinals.len()
            });
            println!("{}({})",
                     Color::White.underline().paint("Imports"),
                     sum);

            for imp in &self.imports {
                fmt_indentln(format!("{}({})({})",
                         Color::Fixed(75).paint(&imp.name),
                         imp.entries.len(),
                         imp.ordinals.len()));

                if imp.entries.len() >= 1 {

                    let mut trimmed = false;
                    let mut table = Table::new();
                    let format = prettytable::format::FormatBuilder::new()
                        .borders(' ')
                        .column_separator(' ')
                        .padding(1, 1)
                        .build();
                    table.set_format(format);
                    table.add_row(row![" ", r->"Idx", "Name"]);

                    for (i, entry) in imp.entries.iter().enumerate() {
                        if i == self.opt.trim_lines {
                            trimmed = true;
                            break;
                        }
                        table.add_row(row![
                            " ",
                            i,
                            Fy->entry,
                        ]);
                    }
                    table.printstd();
                    if trimmed {
                        fmt_indentln(format!("Output trimmed..."));
                    }
                    println!();
                }
                if imp.ordinals.len() >= 1 {

                    let mut trimmed = false;
                    let mut table = Table::new();
                    let format = prettytable::format::FormatBuilder::new()
                        .borders(' ')
                        .column_separator(' ')
                        .padding(1, 1)
                        .build();
                    table.set_format(format);
                    table.add_row(row![" ", r->"Idx", "Ordinal"]);

                    for (i, entry) in imp.ordinals.iter().enumerate() {
                        if i == self.opt.trim_lines {
                            trimmed = true;
                            break;
                        }
                        table.add_row(row![
                            " ",
                            i,
                            Fb->entry,
                        ]);
                    }
                    table.printstd();
                    if trimmed {
                        fmt_indentln(format!("Output trimmed..."));
                    }
                    println!();
                }
            }
            println!();
        }

        //
        // EXPORTS
        //
        if let Some(exports) = &self.exports {
            println!("{}({})",
                     Color::White.underline().paint("Exports"),
                     exports.func_addr.len());
            fmt_indentln(format!("Export flags: {:#X}", exports.header.export_flags));
            fmt_indentln(format!("Time/date stamp: {:#X}", exports.header.timedate_stamp));
            fmt_indentln(format!("Version: {}.{}",
                     exports.header.major_ver,
                     exports.header.minor_ver));
            fmt_indentln(format!("Ordinal base: {}", exports.header.ord_base));
            fmt_indentln(format!("Number of functions: {}",
                     Color::Purple.paint(exports.header.addr_tab_entries.to_string())));
            fmt_indentln(format!("Number of names:     {}",
                     Color::Purple.paint(exports.header.n_name_ptr.to_string())));
            println!();

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Addr", "Name", "Ordinal"]);

            for (i, entry) in exports.funcs.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i,
                    Color::Red.paint(format!("{:#X}", entry.addr)),
                    Fy->entry.name,
                    Fb->entry.ordinal,
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();

        }

        //
        // LIBRARIES
        //
        if self.imports.len() >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("Libraries"),
                     self.imports.len());

            for lib in &self.imports {
                fmt_indentln(format!("{}", Color::Blue.paint(&lib.name)));
            }
            println!();
        }

        Ok(())
    }


}
