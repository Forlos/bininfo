
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
