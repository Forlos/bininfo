// The section should not be padded to the next boundary. This flag is obsolete and is replaced
// by `IMAGE_SCN_ALIGN_1BYTES`. This is valid only for object files.
pub const IMAGE_SCN_TYPE_NO_PAD: u32 = 0x00000008;
// The section contains executable code.
pub const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;
// The section contains initialized data.
pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x00000040;
//  The section contains uninitialized data.
pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;
pub const IMAGE_SCN_LNK_OTHER: u32 = 0x00000100;
// The section contains comments or other information. The .drectve section has this type.
// This is valid for object files only.
pub const IMAGE_SCN_LNK_INFO: u32 = 0x00000200;
// The section will not become part of the image. This is valid only for object files.
pub const IMAGE_SCN_LNK_REMOVE: u32 = 0x00000800;
// The section contains COMDAT data. This is valid only for object files.
pub const IMAGE_SCN_LNK_COMDAT: u32 = 0x00001000;
// The section contains data referenced through the global pointer (GP).
pub const IMAGE_SCN_GPREL: u32 = 0x00008000;
pub const IMAGE_SCN_MEM_PURGEABLE: u32 = 0x00020000;
pub const IMAGE_SCN_MEM_16BIT: u32 = 0x00020000;
pub const IMAGE_SCN_MEM_LOCKED: u32 = 0x00040000;
pub const IMAGE_SCN_MEM_PRELOAD: u32 = 0x00080000;

pub const IMAGE_SCN_ALIGN_1BYTES: u32 = 0x00100000;
pub const IMAGE_SCN_ALIGN_2BYTES: u32 = 0x00200000;
pub const IMAGE_SCN_ALIGN_4BYTES: u32 = 0x00300000;
pub const IMAGE_SCN_ALIGN_8BYTES: u32 = 0x00400000;
pub const IMAGE_SCN_ALIGN_16BYTES: u32 = 0x00500000;
pub const IMAGE_SCN_ALIGN_32BYTES: u32 = 0x00600000;
pub const IMAGE_SCN_ALIGN_64BYTES: u32 = 0x00700000;
pub const IMAGE_SCN_ALIGN_128BYTES: u32 = 0x00800000;
pub const IMAGE_SCN_ALIGN_256BYTES: u32 = 0x00900000;
pub const IMAGE_SCN_ALIGN_512BYTES: u32 = 0x00A00000;
pub const IMAGE_SCN_ALIGN_1024BYTES: u32 = 0x00B00000;
pub const IMAGE_SCN_ALIGN_2048BYTES: u32 = 0x00C00000;
pub const IMAGE_SCN_ALIGN_4096BYTES: u32 = 0x00D00000;
pub const IMAGE_SCN_ALIGN_8192BYTES: u32 = 0x00E00000;
pub const IMAGE_SCN_ALIGN_MASK: u32 = 0x00F00000;

// The section contains extended relocations.
pub const IMAGE_SCN_LNK_NRELOC_OVFL: u32 = 0x01000000;
// The section can be discarded as needed.
pub const IMAGE_SCN_MEM_DISCARDABLE: u32 = 0x02000000;
// The section cannot be cached.
pub const IMAGE_SCN_MEM_NOT_CACHED: u32 = 0x04000000;
// The section is not pageable.
pub const IMAGE_SCN_MEM_NOT_PAGED: u32 = 0x08000000;
// The section can be shared in memory.
pub const IMAGE_SCN_MEM_SHARED: u32 = 0x10000000;
// The section can be executed as code.
pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
// The section can be read.
pub const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
// The section can be written to.
pub const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;


pub const IMAGE_DEBUG_TYPE_UNKNOWN: u32 = 0;
pub const IMAGE_DEBUG_TYPE_COFF: u32 = 1;
pub const IMAGE_DEBUG_TYPE_CODEVIEW: u32 = 2;
pub const IMAGE_DEBUG_TYPE_FPO: u32 = 3;
pub const IMAGE_DEBUG_TYPE_MISC: u32 = 4;
pub const IMAGE_DEBUG_TYPE_EXCEPTION: u32 = 5;
pub const IMAGE_DEBUG_TYPE_FIXUP: u32 = 6;
pub const IMAGE_DEBUG_TYPE_BORLAND: u32 = 9;

pub const CODEVIEW_PDB70_MAGIC: u32 = 0x53445352;
pub const CODEVIEW_PDB20_MAGIC: u32 = 0x3031424e;
pub const CODEVIEW_CV50_MAGIC: u32 = 0x3131424e;
pub const CODEVIEW_CV41_MAGIC: u32 = 0x3930424e;

// Characteristics

// The Characteristics field contains flags that indicate attributes of the object or image file. The following flags are currently defined:

pub const IMAGE_FILE_RELOCS_STRIPPED:         u16 = 0x0001;
pub const IMAGE_FILE_EXECUTABLE_IMAGE:        u16 = 0x0002;
pub const IMAGE_FILE_LINE_NUMS_STRIPPED:      u16 = 0x0004;
pub const IMAGE_FILE_LOCAL_SYMS_STRIPPED:     u16 = 0x0008;
pub const IMAGE_FILE_AGGRESSIVE_WS_TRIM:      u16 = 0x0010;
pub const IMAGE_FILE_LARGE_ADDRESS_AWARE:     u16 = 0x0020;
pub const RESERVED:                           u16 = 0x0040;
pub const IMAGE_FILE_BYTES_REVERSED_LO:       u16 = 0x0080;
pub const IMAGE_FILE_32BIT_MACHINE:           u16 = 0x0100;
pub const IMAGE_FILE_DEBUG_STRIPPED:          u16 = 0x0200;
pub const IMAGE_FILE_REMOVABLE_RUN_FROM_SWAP: u16 = 0x0400;
pub const IMAGE_FILE_NET_RUN_FROM_SWAP:       u16 = 0x0800;
pub const IMAGE_FILE_SYSTEM:                  u16 = 0x1000;
pub const IMAGE_FILE_DLL:                     u16 = 0x2000;
pub const IMAGE_FILE_UP_SYSTEM_ONLY:          u16 = 0x4000;
pub const IMAGE_FILE_BYTES_REVERSED_HI:       u16 = 0x8000;

pub fn is_dll(characteristics: u16) -> bool {
    characteristics & IMAGE_FILE_DLL == IMAGE_FILE_DLL
}

pub fn is_exe(characteristics: u16) -> bool {
    characteristics & IMAGE_FILE_EXECUTABLE_IMAGE == IMAGE_FILE_EXECUTABLE_IMAGE
}

pub fn characteristics_to_str(characteristics: u16) -> String {

    let mut chara = String::new();

    if characteristics & IMAGE_FILE_RELOCS_STRIPPED         == 0x0001 { chara += "RELOC_STRIPPED "; }
    if characteristics & IMAGE_FILE_EXECUTABLE_IMAGE        == 0x0002 { chara += "EXECUTABLE "; }
    if characteristics & IMAGE_FILE_LINE_NUMS_STRIPPED      == 0x0004 { chara += "NUMS_STRIPPED "; }
    if characteristics & IMAGE_FILE_LOCAL_SYMS_STRIPPED     == 0x0008 { chara += "SYMS_STRIPPED "; }
    if characteristics & IMAGE_FILE_AGGRESSIVE_WS_TRIM      == 0x0010 { chara += "WS_TRIM "; }
    if characteristics & IMAGE_FILE_LARGE_ADDRESS_AWARE     == 0x0020 { chara += "LARGE_ADDR "; }
    if characteristics & RESERVED                           == 0x0040 { chara += "RESERVED "; }
    if characteristics & IMAGE_FILE_BYTES_REVERSED_LO       == 0x0080 { chara += "LITTLE_ENDIAN "; }
    if characteristics & IMAGE_FILE_32BIT_MACHINE           == 0x0100 { chara += "32BIT_MACHINE "; }
    if characteristics & IMAGE_FILE_DEBUG_STRIPPED          == 0x0200 { chara += "DEBUG_STRIPPED "; }
    if characteristics & IMAGE_FILE_REMOVABLE_RUN_FROM_SWAP == 0x0400 { chara += "REMOVABLE_SWAP "; }
    if characteristics & IMAGE_FILE_NET_RUN_FROM_SWAP       == 0x0800 { chara += "NET_SWAP "; }
    if characteristics & IMAGE_FILE_SYSTEM                  == 0x1000 { chara += "FILE_SYSTEM "; }
    if characteristics & IMAGE_FILE_DLL                     == 0x2000 { chara += "DLL "; }
    if characteristics & IMAGE_FILE_UP_SYSTEM_ONLY          == 0x4000 { chara += "UP_SYSTEM_ONLY "; }
    if characteristics & IMAGE_FILE_BYTES_REVERSED_HI       == 0x8000 { chara += "BIG_ENDIAN "; }

    chara

}

// Machine Types

// The Machine field has one of the following values that specifies its CPU type. An image file can be run only on the specified machine or on a system that emulates the specified machine.

pub const IMAGE_FILE_MACHINE_UNKNOWN:   u16 = 0x0; // The contents of this field are assumed to be applicable to any machine type
pub const IMAGE_FILE_MACHINE_AM33:      u16 = 0x1d3; // Matsushita AM33
pub const IMAGE_FILE_MACHINE_AMD64:     u16 = 0x8664; // x64
pub const IMAGE_FILE_MACHINE_ARM:       u16 = 0x1c0; // ARM little endian
pub const IMAGE_FILE_MACHINE_ARM64:     u16 = 0xaa64; // ARM64 little endian
pub const IMAGE_FILE_MACHINE_ARMNT:     u16 = 0x1c4; // ARM Thumb-2 little endian
pub const IMAGE_FILE_MACHINE_EBC:       u16 = 0xebc; // EFI byte code
pub const IMAGE_FILE_MACHINE_I386:      u16 = 0x14c; // Intel 386 or later processors and compatible processors
pub const IMAGE_FILE_MACHINE_IA64:      u16 = 0x200; // Intel Itanium processor family
pub const IMAGE_FILE_MACHINE_M32R:      u16 = 0x9041; // Mitsubishi M32R little endian
pub const IMAGE_FILE_MACHINE_MIPS16:    u16 = 0x266; // MIPS16
pub const IMAGE_FILE_MACHINE_MIPSFPU:   u16 = 0x366; // MIPS with FPU
pub const IMAGE_FILE_MACHINE_MIPSFPU16: u16 = 0x466; // MIPS16 with FPU
pub const IMAGE_FILE_MACHINE_POWERPC:   u16 = 0x1f0; // Power PC little endian
pub const IMAGE_FILE_MACHINE_POWERPCFP: u16 = 0x1f1; // Power PC with floating point support
pub const IMAGE_FILE_MACHINE_R4000:     u16 = 0x166; // MIPS little endian
pub const IMAGE_FILE_MACHINE_RISCV32:   u16 = 0x5032; // RISC-V 32-bit address space
pub const IMAGE_FILE_MACHINE_RISCV64:   u16 = 0x5064; // RISC-V 64-bit address space
pub const IMAGE_FILE_MACHINE_RISCV128:  u16 = 0x5128; // RISC-V 128-bit address space
pub const IMAGE_FILE_MACHINE_SH3:       u16 = 0x1a2; // Hitachi SH3
pub const IMAGE_FILE_MACHINE_SH3DSP:    u16 = 0x1a3; // Hitachi SH3 DSP
pub const IMAGE_FILE_MACHINE_SH4:       u16 = 0x1a6; // Hitachi SH4
pub const IMAGE_FILE_MACHINE_SH5:       u16 = 0x1a8; // Hitachi SH5
pub const IMAGE_FILE_MACHINE_THUMB:     u16 = 0x1c2; // Thumb
pub const IMAGE_FILE_MACHINE_WCEMIPSV2: u16 = 0x169; // MIPS little-endian WCE v2

#[inline]
pub fn machine_to_str(machine: u16) -> &'static str {

    match machine {
        IMAGE_FILE_MACHINE_UNKNOWN   => "ANY_MACHINE",
        IMAGE_FILE_MACHINE_AM33      => "Matsushita AM33",
        IMAGE_FILE_MACHINE_AMD64     => "x64",
        IMAGE_FILE_MACHINE_ARM       => "ARM little endian",
        IMAGE_FILE_MACHINE_ARM64     => "ARM64 little endian",
        IMAGE_FILE_MACHINE_ARMNT     => "ARM Thumb-2 little endian",
        IMAGE_FILE_MACHINE_EBC       => "EFI byte code",
        IMAGE_FILE_MACHINE_I386      => "Intel 386",
        IMAGE_FILE_MACHINE_IA64      => "Intel Itanium processor family",
        IMAGE_FILE_MACHINE_M32R      => "Mitsubishi M32R little endian",
        IMAGE_FILE_MACHINE_MIPS16    => "MIPS16",
        IMAGE_FILE_MACHINE_MIPSFPU   => "MIPS with FPU",
        IMAGE_FILE_MACHINE_MIPSFPU16 => "MIPS16 with FPU",
        IMAGE_FILE_MACHINE_POWERPC   => "Power PC little endian",
        IMAGE_FILE_MACHINE_POWERPCFP => "Power PC with floating point support",
        IMAGE_FILE_MACHINE_R4000     => "MIPS little endian",
        IMAGE_FILE_MACHINE_RISCV32   => "RISC-V 32-bit address space",
        IMAGE_FILE_MACHINE_RISCV64   => "RISC-V 64-bit address space",
        IMAGE_FILE_MACHINE_RISCV128  => "RISC-V 128-bit address space",
        IMAGE_FILE_MACHINE_SH3       => "Hitachi SH3",
        IMAGE_FILE_MACHINE_SH3DSP    => "Hitachi SH3 DSP",
        IMAGE_FILE_MACHINE_SH4       => "Hitachi SH3 DSP",
        IMAGE_FILE_MACHINE_SH5       => "Hitachi SH5",
        IMAGE_FILE_MACHINE_THUMB     => "Thumb",
        IMAGE_FILE_MACHINE_WCEMIPSV2 => "MIPS little-endian WCE v2",
        _ => "UNKNOWN_MACHINE",
    }
}
