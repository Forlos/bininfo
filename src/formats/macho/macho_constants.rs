
// * Constants for the filetype field of the mach_header

pub const	MH_OBJECT: u32      = 0x1;
pub const	MH_EXECUTE: u32     = 0x2;
pub const	MH_FVMLIB: u32      = 0x3;
pub const	MH_CORE: u32        = 0x4;
pub const	MH_PRELOAD: u32     = 0x5;
pub const	MH_DYLIB: u32       = 0x6;
pub const	MH_DYLINKER: u32    = 0x7;
pub const	MH_BUNDLE: u32      = 0x8;
pub const	MH_DYLIB_STUB: u32  = 0x9;
pub const	MH_DSYM: u32        = 0xa;
pub const	MH_KEXT_BUNDLE: u32 = 0xb;

pub fn mach_is_exe(filetype: u32) -> bool {
    filetype == MH_EXECUTE
}

pub fn mach_is_lib(filetype: u32) -> bool {
    filetype == MH_DYLIB
}

// * Constants for the flags field of the mach_header

pub const	MH_NOUNDEFS: u32                = 0x1;
pub const	MH_INCRLINK: u32                = 0x2;
pub const MH_DYLDLINK: u32                = 0x4;
pub const MH_BINDATLOAD: u32              = 0x8;
pub const MH_PREBOUND: u32                = 0x10;
pub const MH_SPLIT_SEGS: u32              = 0x20;
pub const MH_LAZY_INIT: u32               = 0x40;
pub const MH_TWOLEVEL: u32                = 0x80;
pub const MH_FORCE_FLAT: u32              = 0x100;
pub const MH_NOMULTIDEFS: u32             = 0x200;
pub const MH_NOFIXPREBINDING: u32         = 0x400;
pub const MH_PREBINDABLE: u32             = 0x800;
pub const MH_ALLMODSBOUND: u32            = 0x1000;
pub const MH_SUBSECTIONS_VIA_SYMBOLS: u32 = 0x2000;
pub const MH_CANONICAL: u32               = 0x4000;
pub const MH_WEAK_DEFINES: u32            = 0x8000;
pub const MH_BINDS_TO_WEAK: u32           = 0x10000;
pub const MH_ALLOW_STACK_EXECUTION: u32   = 0x20000;
pub const MH_ROOT_SAFE: u32               = 0x40000;
pub const MH_SETUID_SAFE: u32             = 0x80000;
pub const MH_NO_REEXPORTED_DYLIBS: u32    = 0x100000;
pub const	MH_PIE: u32                     = 0x200000;
pub const	MH_DEAD_STRIPPABLE_DYLIB: u32   = 0x400000;
pub const MH_HAS_TLV_DESCRIPTORS: u32     = 0x800000;
pub const MH_NO_HEAP_EXECUTION: u32       = 0x1000000;
pub const MH_APP_EXTENSION_SAFE: u32      = 0x2000000;


// Constants for the cmd field of all load commands, the type
pub const LC_REQ_DYLD: u32 = 0x80000000;

pub const	LC_SEGMENT: u32        = 0x1;
pub const	LC_SYMTAB: u32         = 0x2;
pub const	LC_SYMSEG: u32         = 0x3;
pub const	LC_THREAD: u32         = 0x4;
pub const	LC_UNIXTHREAD: u32     = 0x5;
pub const	LC_LOADFVMLIB: u32     = 0x6;
pub const	LC_IDFVMLIB: u32       = 0x7;
pub const	LC_IDENT: u32          = 0x8;
pub const LC_FVMFILE: u32        = 0x9;
pub const LC_PREPAGE: u32        = 0xa;
pub const	LC_DYSYMTAB: u32       = 0xb;
pub const	LC_LOAD_DYLIB: u32     = 0xc;
pub const	LC_ID_DYLIB: u32       = 0xd;
pub const LC_LOAD_DYLINKER: u32  = 0xe;
pub const LC_ID_DYLINKER: u32    = 0xf;
pub const	LC_PREBOUND_DYLIB: u32 = 0x10;
pub const	LC_ROUTINES: u32       = 0x11;
pub const	LC_SUB_FRAMEWORK: u32  = 0x12;
pub const	LC_SUB_UMBRELLA: u32   = 0x13;
pub const	LC_SUB_CLIENT: u32     = 0x14;
pub const	LC_SUB_LIBRARY: u32    = 0x15;
pub const	LC_TWOLEVEL_HINTS: u32 = 0x16;
pub const	LC_PREBIND_CKSUM: u32  = 0x17;

// * load a dynamically linked shared library that is allowed to be missing

pub const	LC_LOAD_WEAK_DYLIB: u32          = (0x18 | LC_REQ_DYLD);
pub const	LC_SEGMENT_64: u32               = 0x19;
pub const	LC_ROUTINES_64: u32              = 0x1a;
pub const LC_UUID: u32                     = 0x1b;
pub const LC_RPATH: u32                    = (0x1c | LC_REQ_DYLD);
pub const LC_CODE_SIGNATURE: u32           = 0x1d;
pub const LC_SEGMENT_SPLIT_INFO: u32       = 0x1e;
pub const LC_REEXPORT_DYLIB: u32           = (0x1f | LC_REQ_DYLD);
pub const	LC_LAZY_LOAD_DYLIB: u32          = 0x20;
pub const	LC_ENCRYPTION_INFO: u32          = 0x21;
pub const	LC_DYLD_INFO: u32                = 0x22;
pub const	LC_DYLD_INFO_ONLY: u32           = (0x22 | LC_REQ_DYLD);
pub const	LC_LOAD_UPWARD_DYLIB: u32        = (0x23 | LC_REQ_DYLD);
pub const LC_VERSION_MIN_MACOSX: u32       = 0x24;
pub const LC_VERSION_MIN_IPHONEOS: u32     = 0x25;
pub const LC_FUNCTION_STARTS: u32          = 0x26;
pub const LC_DYLD_ENVIRONMENT: u32         = 0x27;
pub const LC_MAIN: u32                     = (0x28 | LC_REQ_DYLD);
pub const LC_DATA_IN_CODE: u32             = 0x29;
pub const LC_SOURCE_VERSION: u32           = 0x2A;
pub const LC_DYLIB_CODE_SIGN_DRS: u32      = 0x2B;
pub const	LC_ENCRYPTION_INFO_64: u32       = 0x2C;
pub const LC_LINKER_OPTION: u32            = 0x2D;
pub const LC_LINKER_OPTIMIZATION_HINT: u32 = 0x2E;
pub const LC_VERSION_MIN_TVOS: u32         = 0x2F;
pub const LC_VERSION_MIN_WATCHOS: u32      = 0x30;
pub const LC_NOTE: u32                     = 0x31;
pub const LC_BUILD_VERSION: u32            = 0x32;

pub fn segment_to_str(segment: u32) -> &'static str {
    match segment {
        LC_SEGMENT        =>"LC_SEGMENT",
        LC_SYMTAB         =>"LC_SYMTAB",
        LC_SYMSEG         =>"LC_SYMSEG",
        LC_THREAD         =>"LC_THREAD",
        LC_UNIXTHREAD     =>"LC_UNIXTHREAD",
        LC_LOADFVMLIB     =>"LC_LOADFVMLIB",
        LC_IDFVMLIB       =>"LC_IDFVMLIB",
        LC_IDENT          =>"LC_IDENT",
        LC_FVMFILE        =>"LC_FVMFILE",
        LC_PREPAGE        =>"LC_PREPAGE",
        LC_DYSYMTAB       =>"LC_DYSYMTAB",
        LC_LOAD_DYLIB     =>"LC_LOAD_DYLIB",
        LC_ID_DYLIB       =>"LC_ID_DYLIB",
        LC_LOAD_DYLINKER  =>"LC_LOAD_DYLINKER",
        LC_ID_DYLINKER    =>"LC_ID_DYLINKER",
        LC_PREBOUND_DYLIB =>"LC_PREBOUND_DYLIB",
        LC_ROUTINES       =>"LC_ROUTINES",
        LC_SUB_FRAMEWORK  =>"LC_SUB_FRAMEWORK",
        LC_SUB_UMBRELLA   =>"LC_SUB_UMBRELLA",
        LC_SUB_CLIENT     =>"LC_SUB_CLIENT",
        LC_SUB_LIBRARY    =>"LC_SUB_LIBRARY",
        LC_TWOLEVEL_HINTS =>"LC_TWOLEVEL_HINTS",
        LC_PREBIND_CKSUM  =>"LC_PREBIND_CKSUM",

        LC_LOAD_WEAK_DYLIB          =>"LC_LOAD_WEAK_DYLIB",
        LC_SEGMENT_64               =>"LC_SEGMENT_64",
        LC_ROUTINES_64              =>"LC_ROUTINES_64",
        LC_UUID                     =>"LC_UUID",
        LC_RPATH                    =>"LC_RPATH",
        LC_CODE_SIGNATURE           =>"LC_CODE_SIGNATURE",
        LC_SEGMENT_SPLIT_INFO       =>"LC_SEGMENT_SPLIT_INFO",
        LC_REEXPORT_DYLIB           =>"LC_REEXPORT_DYLIB",
        LC_LAZY_LOAD_DYLIB          =>"LC_LAZY_LOAD_DYLIB",
        LC_ENCRYPTION_INFO          =>"LC_ENCRYPTION_INFO",
        LC_DYLD_INFO                =>"LC_DYLD_INFO",
        LC_DYLD_INFO_ONLY           =>"LC_DYLD_INFO_ONLY",
        LC_LOAD_UPWARD_DYLIB        =>"LC_LOAD_UPWARD_DYLIB",
        LC_VERSION_MIN_MACOSX       =>"LC_VERSION_MIN_MACOSX",
        LC_VERSION_MIN_IPHONEOS     =>"LC_VERSION_MIN_IPHONEOS",
        LC_FUNCTION_STARTS          =>"LC_FUNCTION_STARTS",
        LC_DYLD_ENVIRONMENT         =>"LC_DYLD_ENVIRONMENT",
        LC_MAIN                     =>"LC_MAIN",
        LC_DATA_IN_CODE             =>"LC_DATA_IN_CODE",
        LC_SOURCE_VERSION           =>"LC_SOURCE_VERSION",
        LC_DYLIB_CODE_SIGN_DRS      =>"LC_DYLIB_CODE_SIGN_DRS",
        LC_ENCRYPTION_INFO_64       =>"LC_ENCRYPTION_INFO_64",
        LC_LINKER_OPTION            =>"LC_LINKER_OPTION",
        LC_LINKER_OPTIMIZATION_HINT =>"LC_LINKER_OPTIMIZATION_HINT",
        LC_VERSION_MIN_TVOS         =>"LC_VERSION_MIN_TVOS",
        LC_VERSION_MIN_WATCHOS      =>"LC_VERSION_MIN_WATCHOS",
        LC_NOTE                     =>"LC_NOTE",
        LC_BUILD_VERSION            =>"LC_BUILD_VERSION",

        _ => "INVALID_SEGMENT"
    }
}

// * Constants for the flags field of the segment_command

pub const	SG_HIGHVM: u32 = 0x1;
pub const	SG_FVMLIB: u32 = 0x2;
pub const	SG_NORELOC: u32 = 0x4;
pub const SG_PROTECTED_VERSION_1: u32 = 0x8;

// * Known values for the platform field.

pub const PLATFORM_MACOS:   u32 = 1;
pub const PLATFORM_IOS:     u32 = 2;
pub const PLATFORM_TVOS:    u32 = 3;
pub const PLATFORM_WATCHOS: u32 = 4;

// * Known values for the tool field.

pub const TOOL_CLANG: u32 = 1;
pub const TOOL_SWIFT: u32 = 2;
pub const TOOL_LD:    u32 = 3;

// * The following are used to encode rebasing information

pub const REBASE_TYPE_POINTER:         u32 = 1;
pub const REBASE_TYPE_TEXT_ABSOLUTE32: u32 = 2;
pub const REBASE_TYPE_TEXT_PCREL32:    u32 = 3;

pub const REBASE_OPCODE_MASK:                               u32 = 0xF0;
pub const REBASE_IMMEDIATE_MASK:                            u32 = 0x0F;
pub const REBASE_OPCODE_DONE:                               u32 = 0x00;
pub const REBASE_OPCODE_SET_TYPE_IMM:                       u32 = 0x10;
pub const REBASE_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB:        u32 = 0x20;
pub const REBASE_OPCODE_ADD_ADDR_ULEB:                      u32 = 0x30;
pub const REBASE_OPCODE_ADD_ADDR_IMM_SCALED:                u32 = 0x40;
pub const REBASE_OPCODE_DO_REBASE_IMM_TIMES:                u32 = 0x50;
pub const REBASE_OPCODE_DO_REBASE_ULEB_TIMES:               u32 = 0x60;
pub const REBASE_OPCODE_DO_REBASE_ADD_ADDR_ULEB:            u32 = 0x70;
pub const REBASE_OPCODE_DO_REBASE_ULEB_TIMES_SKIPPING_ULEB: u32 = 0x80;

// * The folowing are used to encode binding information

pub const BIND_SPECIAL_DYLIB_SELF:            i32 = 0;
pub const BIND_SPECIAL_DYLIB_MAIN_EXECUTABLE: i32 = -1;
pub const BIND_SPECIAL_DYLIB_FLAT_LOOKUP:     i32 =	-2;

pub const BIND_SYMBOL_FLAGS_WEAK_IMPORT:         u32 = 0x1;
pub const BIND_SYMBOL_FLAGS_NON_WEAK_DEFINITION: u32 = 0x8;

pub const BIND_OPCODE_MASK:                             u32 = 0xF0;
pub const BIND_IMMEDIATE_MASK:                          u32 = 0x0F;
pub const BIND_OPCODE_DONE:                             u32 = 0x00;
pub const BIND_OPCODE_SET_DYLIB_ORDINAL_IMM:            u32 = 0x10;
pub const BIND_OPCODE_SET_DYLIB_ORDINAL_ULEB:           u32 = 0x20;
pub const BIND_OPCODE_SET_DYLIB_SPECIAL_IMM:            u32 = 0x30;
pub const BIND_OPCODE_SET_SYMBOL_TRAILING_FLAGS_IMM:    u32 = 0x40;
pub const BIND_OPCODE_SET_TYPE_IMM:                     u32 = 0x50;
pub const BIND_OPCODE_SET_ADDEND_SLEB:                  u32 = 0x60;
pub const BIND_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB:      u32 = 0x70;
pub const BIND_OPCODE_ADD_ADDR_ULEB:                    u32 = 0x80;
pub const BIND_OPCODE_DO_BIND:                          u32 = 0x90;
pub const BIND_OPCODE_DO_BIND_ADD_ADDR_ULEB:            u32 = 0xA0;
pub const BIND_OPCODE_DO_BIND_ADD_ADDR_IMM_SCALED:      u32 = 0xB0;
pub const BIND_OPCODE_DO_BIND_ULEB_TIMES_SKIPPING_ULEB: u32 = 0xC0;

// * The folowing are used on the flags byte of a terminal node in the export information.

pub const EXPORT_SYMBOL_FLAGS_KIND_MASK:         u32 = 0x03;
pub const EXPORT_SYMBOL_FLAGS_KIND_REGULAR:      u32 = 0x00;
pub const EXPORT_SYMBOL_FLAGS_KIND_THREAD_LOCAL: u32 = 0x01;
pub const EXPORT_SYMBOL_FLAGS_WEAK_DEFINITION:   u32 = 0x04;
pub const EXPORT_SYMBOL_FLAGS_REEXPORT:          u32 = 0x08;
pub const EXPORT_SYMBOL_FLAGS_STUB_AND_RESOLVER: u32 = 0x10;
