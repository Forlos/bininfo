#![allow(non_camel_case_types)]
include!("macho_constants.rs");
use failure::{Error};
use scroll::{self, Pread};

use crate::Opt;
use crate::Problem;
use crate::format::{fmt_macho, fmt_indentln};

pub const MACHO_MAGIC_32: &'static [u8; MACHO_MAGIC_SIZE] = b"\xFE\xED\xFA\xCE";
pub const MACHO_MAGIC_64: &'static [u8; MACHO_MAGIC_SIZE] = b"\xFE\xED\xFA\xCF";
pub const MACHO_MAGIC_32_R: &'static [u8; MACHO_MAGIC_SIZE] = b"\xCE\xFA\xED\xFE";
pub const MACHO_MAGIC_64_R: &'static [u8; MACHO_MAGIC_SIZE] = b"\xCF\xFA\xED\xFE";
pub const MACHO_MAGIC_SIZE: usize = 4;

// pub const MACHO_FAT_MAGIC: &'static [u8; MACHO_MAGIC_SIZE] = b"\xCA\xFE\xBA\xBE";

#[derive(Debug, Pread)]
pub struct Mach_header_32 {
    magic:          u32,
    pub cputype:    u32,
    pub cpusubtype: u32,
    pub filetype:   u32,
    n_cmds:         u32,
    size_of_cmd:    u32,
    pub flags:      u32,
}

#[derive(Debug, Pread)]
pub struct Mach_header {
    magic:          u32,
    pub cputype:    u32,
    pub cpusubtype: u32,
    pub filetype:   u32,
    n_cmds:         u32,
    size_of_cmd:    u32,
    pub flags:      u32,
    reserved:       u32,
}

impl From<Mach_header_32> for Mach_header {
    fn from(header: Mach_header_32) -> Self {
        Mach_header {
            magic:       header.magic,
            cputype:     header.cputype,
            cpusubtype:  header.cpusubtype,
            filetype:    header.filetype,
            n_cmds:      header.n_cmds,
            size_of_cmd: header.size_of_cmd,
            flags:       header.flags,
            reserved:    0,
        }
    }
}

#[derive(Debug, AsRefStr)]
enum LoadCommand {
    Segment(u32, Segment_command),
    Fvmlib(u32, Fvmlib_command),
    Dylib(u32, Dylib_command),
    SubFramework(u32, Sub_framework_command),
    SubClient(u32, Sub_client_command),
    SubUmbrella(u32, Sub_umbrella_command),
    SubLibrary(u32, Sub_library_command),
    PreboundDylib(u32, Prebound_dylib_command),
    Dylinker(u32, Dylinker_command),
    Thread(u32, Thread_command),
    Routines(u32, Routines_command),
    SymTab(u32, Symtab_command),
    DySymTab(u32, Dysymtab_command),
    TwolevelHints(u32, Twolevel_hints_command),
    PrebindCksum(u32, Prebind_cksum_command),
    UUID(u32, UUID_command),
    Rpath(u32, Rpath_command),
    LinkeditData(u32, Linkedit_data_command),
    EncryptionInfo(u32, Encryption_info_command),
    VersionMin(u32, Version_min_command),
    BuildVersion(u32, Build_version_command),
    DyldInfo(u32, Dyld_info_command),
    LinkerOption(u32, Linker_option_command),
    Symseg(u32, Symseg_command),
    Ident(u32, Ident_command),
    FvmFile(u32, Fvmfile_command),
    EntryPoint(u32, Entry_point_command),
    SourceVersion(u32, Source_version_command),
    Note(u32, Note_command),
}

impl LoadCommand {
    fn get_cmd_type(&self) -> u32 {
        use LoadCommand::*;

        match self  {
            Segment(cmd,_)        => *cmd,
            Fvmlib(cmd,_)         => *cmd,
            Dylib(cmd,_)          => *cmd,
            SubFramework(cmd,_)   => *cmd,
            SubClient(cmd,_)      => *cmd,
            SubUmbrella(cmd,_)    => *cmd,
            SubLibrary(cmd,_)     => *cmd,
            PreboundDylib(cmd,_)  => *cmd,
            Dylinker(cmd,_)       => *cmd,
            Thread(cmd,_)         => *cmd,
            Routines(cmd,_)       => *cmd,
            SymTab(cmd,_)         => *cmd,
            DySymTab(cmd,_)       => *cmd,
            TwolevelHints(cmd,_)  => *cmd,
            PrebindCksum(cmd,_)   => *cmd,
            UUID(cmd,_)           => *cmd,
            Rpath(cmd,_)          => *cmd,
            LinkeditData(cmd,_)   => *cmd,
            EncryptionInfo(cmd,_) => *cmd,
            VersionMin(cmd,_)     => *cmd,
            BuildVersion(cmd,_)   => *cmd,
            DyldInfo(cmd,_)       => *cmd,
            LinkerOption(cmd,_)   => *cmd,
            Symseg(cmd,_)         => *cmd,
            Ident(cmd,_)          => *cmd,
            FvmFile(cmd,_)        => *cmd,
            EntryPoint(cmd,_)     => *cmd,
            SourceVersion(cmd,_)  => *cmd,
            Note(cmd,_)           => *cmd,
        }
    }
}

#[derive(Debug, Pread, Clone)]
struct Load_command {
    cmd:    u32,
    cmd_sz: u32,
}

#[derive(Debug, Pread)]
struct Segment_command_32 {
    cmd:       Load_command,
    seg_name:  [u8; 16],
    vm_addr:   u32,
    vm_sz:     u32,
    file_off:  u32,
    file_sz:   u32,
    max_prot:  u32,
    init_prot: u32,
    n_sects:   u32,
    flags:     u32,
}

#[derive(Debug, Pread, Clone)]
struct Segment_command {
    cmd:       Load_command,
    seg_name:  [u8; 16],
    vm_addr:   u64,
    vm_sz:     u64,
    file_off:  u64,
    file_sz:   u64,
    max_prot:  u32,
    init_prot: u32,
    n_sects:   u32,
    flags:     u32,
}

impl From<Segment_command_32> for Segment_command {
    fn from(seg: Segment_command_32) -> Self {
        Segment_command {
            cmd:       seg.cmd,
            seg_name:  seg.seg_name,
            vm_addr:   seg.vm_addr as u64,
            vm_sz:     seg.vm_sz as u64,
            file_off:  seg.file_off as u64,
            file_sz:   seg.file_sz as u64,
            max_prot:  seg.max_prot,
            init_prot: seg.init_prot,
            n_sects:   seg.n_sects,
            flags:     seg.flags,
        }
    }
}

#[derive(Debug)]
struct Segment {
    header: Segment_command,
    sects:  Vec<Section>,
}

#[derive(Debug, Pread)]
struct Section_32 {
    sect_name: [u8; 16],
    seg_name:  [u8; 16],
    addr:      u32,
    size:      u32,
    offset:    u32,
    align:     u32,
    reloff:    u32,
    n_reloc:   u32,
    flags:     u32,
    reserved1: u32,
    reserved2: u32,
}

#[derive(Debug, Pread)]
struct Section {
    sect_name: [u8; 16],
    seg_name:  [u8; 16],
    addr:      u64,
    size:      u64,
    offset:    u32,
    align:     u32,
    reloff:    u32,
    n_reloc:   u32,
    flags:     u32,
    reserved1: u32,
    reserved2: u32,
    reserved3: u32,
}

impl From<Section_32> for Section {
    fn from(sec: Section_32) -> Self {
        Section {
            sect_name: sec.sect_name,
            seg_name:  sec.seg_name,
            addr:      sec.addr as u64,
            size:      sec.size as u64,
            offset:    sec.offset,
            align:     sec.align,
            reloff:    sec.reloff,
            n_reloc:   sec.n_reloc,
            flags:     sec.flags,
            reserved1: sec.reserved1,
            reserved2: sec.reserved2,
            reserved3: 0,
        }
    }
}

#[derive(Debug, Pread)]
struct Fvmlib {
    lc_str: u64,
    minor_ver: u32,
    header_adr: u32,
}

#[derive(Debug, Pread)]
struct Fvmlib_command {
    cmd: Load_command,
    fvmlib: Fvmlib,
}

#[derive(Debug, Pread)]
struct Dylib {
    lc_str:    u64,
    timestamp: u32,
    cur_ver:   u32,
    comp_ver:  u32,
}

#[derive(Debug, Pread)]
struct Dylib_command {
    cmd: Load_command,
    dylib: Dylib,
}

#[derive(Debug, Pread)]
struct Sub_framework_command {
    cmd: Load_command,
    lc_str: u64,
}

#[derive(Debug, Pread)]
struct Sub_client_command {
    cmd: Load_command,
    lc_str: u64,
}

#[derive(Debug, Pread)]
struct Sub_umbrella_command {
    cmd: Load_command,
    lc_str: u64,
}

#[derive(Debug, Pread)]
struct Sub_library_command {
    cmd: Load_command,
    lc_str: u64,
}

#[derive(Debug, Pread)]
struct Prebound_dylib_command {
    cmd: Load_command,
    lc_str_name: u64,
    n_modules: u32,
    lc_str_mods: u64,
}

#[derive(Debug, Pread)]
struct Dylinker_command {
    cmd: Load_command,
    lc_str: u64,
}

#[derive(Debug)]
struct Thread_command {
    cmd: Load_command,
    flavor: u32,
    cnt: u32,
    state: ThreadState,
}

#[derive(Debug)]
enum ThreadState {
    X86,
    ARM,
}

#[derive(Debug, Pread)]
struct Routines_command_32 {
    cmd:       Load_command,
    init_addr: u32,
    init_mod:  u32,

    reserved1: u32,
    reserved2: u32,
    reserved3: u32,
    reserved4: u32,
    reserved5: u32,
    reserved6: u32,
}

#[derive(Debug, Pread)]
struct Routines_command {
    cmd:       Load_command,
    init_addr: u64,
    init_mod:  u64,

    reserved1: u64,
    reserved2: u64,
    reserved3: u64,
    reserved4: u64,
    reserved5: u64,
    reserved6: u64,
}

impl From<Routines_command_32> for Routines_command {
    fn from(rou: Routines_command_32) -> Self {
        Routines_command {
            cmd:       rou.cmd,
            init_addr: rou.init_addr as u64,
            init_mod:  rou.init_mod as u64,

            reserved1: rou.reserved1 as u64,
            reserved2: rou.reserved2 as u64,
            reserved3: rou.reserved3 as u64,
            reserved4: rou.reserved4 as u64,
            reserved5: rou.reserved5 as u64,
            reserved6: rou.reserved6 as u64,
        }
    }
}

#[derive(Debug, Pread, Clone)]
struct Symtab_command {
    cmd:     Load_command,
    sym_off: u32,
    n_syms:  u32,
    str_off: u32,
    str_sz:  u32,
}

#[derive(Debug, Pread)]
struct Nlist_32 {
    n_un:    u32,
    n_type:  u8,
    n_sect:  u8,
    n_desc:  u16,
    n_value: u32,
}

#[derive(Debug, Pread)]
struct Nlist {
    n_un:    u32,
    n_type:  u8,
    n_sect:  u8,
    n_desc:  u16,
    n_value: u64,
}

impl From<Nlist_32> for Nlist {
    fn from(list: Nlist_32) -> Self {
        Nlist {
            n_un:    list.n_un,
            n_type:  list.n_type,
            n_sect:  list.n_sect,
            n_desc:  list.n_desc,
            n_value: list.n_value as u64,
        }
    }
}

#[derive(Debug)]
struct Symtab {
    header: Symtab_command,
    syms:   Vec<Nlist>,
    strs:   Vec<u8>,
}

#[derive(Debug, Pread)]
struct Dysymtab_command {
    cmd: Load_command,

    local_sym_idx: u32,
    local_sym_n: u32,
    ext_def_sym_idx: u32,
    ext_def_sym_n: u32,
    undef_sym_idx: u32,
    under_sym_n: u32,

    toc_off: u32,
    toc_n: u32,

    mod_tab_off: u32,
    mod_tab_n: u32,

    ext_ref_sym_off: u32,
    ext_ref_sym_n: u32,

    indirect_sym_off: u32,
    indirect_sym_n: u32,

    ext_rel_off: u32,
    ext_rel_n: u32,

    loc_rel_off: u32,
    loc_ref_n: u32,
}
// * a table of contents entry
#[derive(Debug, Pread)]
struct Dylib_toc {
    sym_idx: u32,
    mod_idx: u32,
}
// * a module table entry
#[derive(Debug, Pread)]
struct Dylib_module_32 {
    mod_name:              u32,

    ext_def_sym_idx:       u32,
    ext_def_sym_n:         u32,
    ref_sym_idx:           u32,
    ref_sym_n:             u32,
    local_sym_idx:         u32,
    local_sym_n:           u32,

    ext_rel_idx:           u32,
    ext_rel_n:             u32,

    init_iterm_idx:        u32,
    init_nterm_n:          u32,

    objc_module_info_addr: u32,
    objc_module_info_size: u32,
}
// * a module table entry
#[derive(Debug, Pread)]
struct Dylib_module {
    mod_name:              u32,

    ext_def_sym_idx:       u32,
    ext_def_sym_n:         u32,
    ref_sym_idx:           u32,
    ref_sym_n:             u32,
    local_sym_idx:         u32,
    local_sym_n:           u32,

    ext_rel_idx:           u32,
    ext_rel_n:             u32,

    init_iterm_idx:        u32,
    init_nterm_n:          u32,

    objc_module_info_addr: u32,
    objc_module_info_size: u64,
}

impl From<Dylib_module_32> for Dylib_module {
    fn from(dyl: Dylib_module_32) -> Self {
        Dylib_module {
            mod_name:              dyl.mod_name,

            ext_def_sym_idx:       dyl.ext_def_sym_idx,
            ext_def_sym_n:         dyl.ext_def_sym_n,
            ref_sym_idx:           dyl.ref_sym_idx,
            ref_sym_n:             dyl.ref_sym_n,
            local_sym_idx:         dyl.local_sym_idx,
            local_sym_n:           dyl.local_sym_n,

            ext_rel_idx:           dyl.ext_rel_idx,
            ext_rel_n:             dyl.ext_rel_n,

            init_iterm_idx:        dyl.init_iterm_idx,
            init_nterm_n:          dyl.init_nterm_n,

            objc_module_info_addr: dyl.objc_module_info_addr,
            objc_module_info_size: dyl.objc_module_info_size as u64,
        }
    }
}

#[derive(Debug, Pread)]
struct Dylib_reference {
    refer: u32,
}

impl Dylib_reference {
    fn get_index(&self) -> u32 {
        self.refer >> 8
    }
    fn get_flags(&self) -> u8 {
        ((self.refer << 24) >> 24) as u8
    }
}

#[derive(Debug, Pread)]
struct Twolevel_hints_command {
    cmd:     Load_command,
    offset:  u32,
    n_hints: u32,
}

#[derive(Debug, Pread)]
struct Twolevel_hint {
    hint: u32,
}

impl Twolevel_hint {
    fn get_image(&self) -> u8 {
        (self.hint >> 24) as u8
    }
    fn get_toc(&self) -> u32 {
        (self.hint << 8) >> 8
    }
}

#[derive(Debug, Pread)]
struct Prebind_cksum_command {
    cmd:   Load_command,
    cksum: u32,
}

#[derive(Debug, Pread)]
struct UUID_command {
    cmd:  Load_command,
    uuid: [u8; 16],
}

#[derive(Debug, Pread)]
struct Rpath_command {
    cmd:    Load_command,
    lc_str: u64,
}

#[derive(Debug, Pread)]
struct Linkedit_data_command {
    cmd:      Load_command,
    data_off: u32,
    data_sz:  u32,
}

#[derive(Debug, Pread)]
struct Encryption_info_command_32 {
    cmd:       Load_command,
    crypt_off: u32,
    crypt_sz:  u32,
    crypt_id:  u32,
}

#[derive(Debug, Pread)]
struct Encryption_info_command {
    cmd:       Load_command,
    crypt_off: u32,
    crypt_sz:  u32,
    crypt_id:  u32,
    pad:       u32,
}

impl From<Encryption_info_command_32> for Encryption_info_command {
    fn from(enc: Encryption_info_command_32) -> Self {
        Encryption_info_command {
            cmd:       enc.cmd,
            crypt_off: enc.crypt_off,
            crypt_sz:  enc.crypt_sz,
            crypt_id:  enc.crypt_id,
            pad:       0,
        }
    }
}

#[derive(Debug, Pread)]
struct Version_min_command {
    cmd: Load_command,
    version: u32,
    sdk: u32,
}

#[derive(Debug, Pread)]
struct Build_version_command {
    cmd:      Load_command,
    platform: u32,
    minos:    u32,
    sdk:      u32,
    n_tools:  u32,
}

#[derive(Debug, Pread)]
struct build_tool_version {
    tool:    u32,
    version: u32,
}

#[derive(Debug, Pread)]
struct Dyld_info_command {
    cmd:           Load_command,

    rebase_off:    u32,
    rebase_sz:     u32,

    bind_off:      u32,
    bind_sz:       u32,

    weak_bind_off: u32,
    weak_bind_sz:  u32,

    lazy_bind_off: u32,
    lazy_bind_sz:  u32,

    export_off:    u32,
    export_sz:     u32,
}

#[derive(Debug)]
struct Linker_option_command {
    cmd:     Load_command,
    cnt:     u32,
    strings: Vec<String>,
}

#[derive(Debug, Pread)]
struct Symseg_command {
    cmd:    Load_command,
    offset: u32,
    size:   u32,
}

#[derive(Debug)]
struct Ident_command {
    cmd:     Load_command,
    strings: Vec<String>,
}

#[derive(Debug, Pread)]
struct Fvmfile_command {
    cmd:         Load_command,
    lc_str:      u64,
    header_addr: u32,
}

#[derive(Debug, Pread)]
struct Entry_point_command {
    cmd:       Load_command,
    entry_off: u64,
    stack_sz:  u64,
}

#[derive(Debug, Pread)]
struct Source_version_command {
    cmd: Load_command,
    version: u64,
}

#[derive(Debug, Pread)]
struct Note_command {
    cmd: Load_command,
    data_owner: [u8; 16],
    offset: u64,
    size: u64,
}

#[derive(Debug)]
pub struct MachO {
    opt:      Opt,

    header:   Mach_header,
    commands: Vec<LoadCommand>,

    segments: Vec<Segment>,
    symtab:   Option<Symtab>,
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
            // This should never happen as header has been already checked
            _ => unreachable!(),
        };
        let is_64bit = match buf.pread::<u32>(0)? {
            MH_MAGIC | MH_CIGAM => false,
            MH_MAGIC_64 | MH_CIGAM_64 => true,
            // This should never happen as header has been already checked
            _ => unreachable!(),
        };

        let offset = &mut 0;
        let header;
        if is_64bit { header = buf.gread_with::<Mach_header>(offset, endianess)?; }
        else { header = Mach_header::from(buf.gread_with::<Mach_header_32>(offset, endianess)?); }

        let mut commands = Vec::with_capacity(header.n_cmds as usize);
        let mut segments = Vec::new();
        let mut symtab   = None;

        for i in 0..header.n_cmds {
            let cmd = buf.pread_with::<u32>(*offset, endianess)?;
            let cmd_sz = buf.pread_with::<u32>(*offset + 4, endianess)?;
            commands.push( match cmd {
                LC_SEGMENT => {
                    let segment = Segment_command::from(buf.pread_with::<Segment_command_32>(*offset, endianess)?);
                    let mut sects   = Vec::with_capacity(segment.n_sects as usize);
                    for i in 0..segment.n_sects as usize {
                        sects.push(Section::from(buf.pread_with::<Section_32>(*offset + 56 + (i * 68), endianess)?));
                    }
                    segments.push(Segment { header: segment.clone(), sects });
                    LoadCommand::Segment(cmd, segment)
                },
                LC_SEGMENT_64 => {
                    let segment = buf.pread_with::<Segment_command>(*offset, endianess)?;
                    let mut sects   = Vec::with_capacity(segment.n_sects as usize);
                    for i in 0..segment.n_sects as usize {
                        sects.push(buf.pread_with::<Section>(*offset + 72 + (i * 80), endianess)?);
                    }
                    segments.push(Segment { header: segment.clone(), sects });
                    LoadCommand::Segment(cmd, segment)
                },
                LC_IDFVMLIB | LC_LOADFVMLIB => {
                    LoadCommand::Fvmlib(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_ID_DYLIB | LC_LOAD_DYLIB | LC_LOAD_WEAK_DYLIB | LC_REEXPORT_DYLIB => {
                    LoadCommand::Dylib(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_SUB_FRAMEWORK => {
                    LoadCommand::SubFramework(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_SUB_CLIENT => {
                    LoadCommand::SubClient(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_SUB_UMBRELLA => {
                    LoadCommand::SubUmbrella(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_SUB_LIBRARY => {
                    LoadCommand::SubLibrary(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_PREBOUND_DYLIB => {
                    LoadCommand::PreboundDylib(cmd, buf.pread_with(*offset,endianess)?)
                },
                LC_ID_DYLINKER | LC_LOAD_DYLINKER | LC_DYLD_ENVIRONMENT => {
                    LoadCommand::Dylinker(cmd, buf.pread_with(*offset, endianess)?)
                },
                // LC_THREAD | LC_UNIXTHREAD => {
                //     LoadCommand::Thread(cmd, buf.pread_with(*offset, endianess)?)
                // },
                LC_ROUTINES => {
                    LoadCommand::Routines(cmd, Routines_command::from(buf.pread_with::<Routines_command_32>(*offset, endianess)?))
                },
                LC_ROUTINES_64 => {
                    LoadCommand::Routines(cmd, buf.pread_with::<Routines_command>(*offset, endianess)?)
                },
                LC_SYMTAB => {
                    let header = buf.pread_with::<Symtab_command>(*offset, endianess)?;
                    let mut syms = Vec::with_capacity(header.n_syms as usize);
                    for i in 0..header.n_syms as usize {
                        if is_64bit { syms.push(buf.pread_with::<Nlist>(header.sym_off as usize + (i * 16), endianess)?); }
                        else { syms.push(Nlist::from(buf.pread_with::<Nlist_32>(header.sym_off as usize + (i * 12), endianess)?)); }
                    }
                    let strs = buf[header.str_off as usize..header.str_off as usize + header.str_sz as usize].to_vec();
                    symtab = Some(Symtab { header: header.clone(), syms, strs });
                    LoadCommand::SymTab(cmd, header)
                },
                LC_DYSYMTAB => {
                    LoadCommand::DySymTab(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_TWOLEVEL_HINTS => {
                    LoadCommand::TwolevelHints(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_PREBIND_CKSUM => {
                    LoadCommand::PrebindCksum(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_UUID => {
                    LoadCommand::UUID(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_RPATH => {
                    LoadCommand::Rpath(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_CODE_SIGNATURE | LC_SEGMENT_SPLIT_INFO | LC_FUNCTION_STARTS |
                LC_DATA_IN_CODE | LC_DYLIB_CODE_SIGN_DRS | LC_LINKER_OPTIMIZATION_HINT => {
                    LoadCommand::LinkeditData(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_ENCRYPTION_INFO => {
                    LoadCommand::EncryptionInfo(cmd, Encryption_info_command::from(
                        buf.pread_with::<Encryption_info_command_32>(*offset, endianess)?))
                },
                LC_ENCRYPTION_INFO_64 => {
                    LoadCommand::EncryptionInfo(cmd, buf.pread_with::<Encryption_info_command>(*offset, endianess)?)
                },
                LC_VERSION_MIN_MACOSX | LC_VERSION_MIN_IPHONEOS |
                LC_VERSION_MIN_WATCHOS | LC_VERSION_MIN_TVOS => {
                    LoadCommand::VersionMin(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_BUILD_VERSION => {
                    LoadCommand::BuildVersion(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_DYLD_INFO | LC_DYLD_INFO_ONLY => {
                    LoadCommand::DyldInfo(cmd, buf.pread_with(*offset, endianess)?)
                },
                // LC_LINKER_OPTION => {
                //     LoadCommand::LinkerOption(cmd, buf.pread_with(*offset, endianess)?)
                // },
                LC_SYMSEG => {
                    LoadCommand::Symseg(cmd, buf.pread_with(*offset, endianess)?)
                },
                // LC_IDENT => {
                //     LoadCommand::Ident(cmd, buf.pread_with(*offset, endianess)?)
                // }
                LC_FVMFILE => {
                    LoadCommand::FvmFile(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_MAIN => {
                    LoadCommand::EntryPoint(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_SOURCE_VERSION => {
                    LoadCommand::SourceVersion(cmd, buf.pread_with(*offset, endianess)?)
                },
                LC_NOTE => {
                    LoadCommand::Note(cmd, buf.pread_with(*offset, endianess)?)
                }
                _ => return Err(Error::from(Problem::Msg(format!("Invalid/Unsupported command type: {} at: {:#X}", cmd, *offset)))),
            });
            *offset += cmd_sz as usize;
            println!("{:#X?}", commands[i as usize]);
        }

        Ok(MachO {
            opt,

            header,
            commands,

            segments,
            symtab,
        })

    }

    fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;
        use prettytable::Table;

        println!("{:#X?}", self.commands);
        println!("{:#X?}", self.symtab);

        //
        // MACH-O FILE
        //
        fmt_macho(&self.header);
        println!();

        //
        // COMMANDS
        //
        if self.commands.len() >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("LoadCommands"),
                     self.commands.len());

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Name"]);

            for (i, entry) in self.commands.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i,
                    segment_to_str(entry.get_cmd_type()),
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();

        }

        //
        // SEGMENTS
        //
        if self.segments.len() >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("Segments"),
                     self.commands.len());

            for entry in &self.segments {

                fmt_indentln(format!("{}({})",
                                     Color::Fixed(75).paint(std::str::from_utf8(&entry.header.seg_name)?),
                                     entry.sects.len()));

                if entry.sects.len() >= 1 {

                    let mut trimmed = false;
                    let mut table = Table::new();
                    let format = prettytable::format::FormatBuilder::new()
                        .borders(' ')
                        .column_separator(' ')
                        .padding(3, 0)
                        .build();
                    table.set_format(format);
                    table.add_row(row!["Idx", "Name", "Addr", "Size", "Offset", "Align", "RelOff", "Nreloc", "Flags"]);

                    for (i, entry) in entry.sects.iter().enumerate() {
                        if i == self.opt.trim_lines {
                            trimmed = true;
                            break;
                        }
                        table.add_row(row![
                            i,
                            std::str::from_utf8(&entry.sect_name)?,
                            Fr->format!("{:#X}", entry.addr),
                            Fg->format!("{:#X}", entry.size),
                            Fy->format!("{:#X}", entry.offset),
                            format!("{:#X}", entry.align),
                            format!("{:#X}", entry.reloff),
                            Fmr->entry.n_reloc,
                            format!("{} {}",
                                    Color::Blue.paint(section_type_to_str(entry.flags)),
                                    section_attr_to_str(entry.flags))
                        ]);
                    }
                    table.printstd();
                    if trimmed {
                        fmt_indentln(format!("Output trimmed..."));
                    }
                }
                println!();
            }
        }

        //
        // SYMBOLS
        //
        if let Some(symtab) = &self.symtab {
            println!("{}({})",
                     Color::White.underline().paint("Symbols"),
                     symtab.syms.len());

            if symtab.syms.len() >= 1 {

                let mut trimmed = false;
                let mut table = Table::new();
                let format = prettytable::format::FormatBuilder::new()
                    .borders(' ')
                    .column_separator(' ')
                    .padding(1, 1)
                    .build();
                table.set_format(format);
                table.add_row(row!["Idx", "Name"]);

                for (i, entry) in symtab.syms.iter().enumerate() {
                    if i == self.opt.trim_lines {
                        trimmed = true;
                        break;
                    }
                    table.add_row(row![
                        i,
                        Fy->symtab.strs.pread::<&str>(entry.n_un as usize)?,
                    ]);
                }
                table.printstd();
                if trimmed {
                    fmt_indentln(format!("Output trimmed..."));
                }
                println!();
            }
        }

        Ok(())
    }

}
