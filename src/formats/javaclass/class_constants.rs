pub const CONSTANT_UTF8: u8               =	1;
pub const CONSTANT_INTEGER: u8            =	3;
pub const CONSTANT_FLOAT: u8              =	4;
pub const CONSTANT_LONG: u8               =	5;
pub const CONSTANT_DOUBLE: u8             =	6;
pub const CONSTANT_CLASS: u8              =	7;
pub const CONSTANT_STRING: u8             = 8;
pub const CONSTANT_FIELDREF: u8           =	9;
pub const CONSTANT_METHODREF: u8          =	10;
pub const CONSTANT_INTERFACEMETHODREF: u8 =	11;
pub const CONSTANT_NAMEANDTYPE: u8        =	12;
pub const CONSTANT_METHODHANDLE: u8       = 15; // JAVA 7
pub const CONSTANT_METHODTYPE: u8         = 16; // JAVA 7
pub const CONSTANT_DYNAMIC: u8            = 17; // JAVA 11
pub const CONSTANT_INVOKEDYNAMIC: u8      = 18; // JAVA 7
pub const CONSTANT_MODULE: u8             = 19; // JAVA 9
pub const CONSTANT_PACKAGE: u8            = 20; // JAVA 9

pub fn java_version_to_str(minor: u16, major: u16) -> &'static str {

    if major == 45 && minor <= 3 {
        "JAVA 1.0.2"
    }
    else if major == 45 && minor > 3 {
        "JAVA 1.1"
    }
    else if major > 45 && major <= 46 {
        "JAVA 1.2"
    }
    else if major > 46 && major <= 47 {
        "JAVA 1.3"
    }
    else if major > 47 && major <= 48 {
        "JAVA 1.4"
    }
    else if major > 48 && major <= 49 {
        "JAVA 5.0"
    }
    else if major > 49 && major <= 50 {
        "JAVA 6"
    }
    else if major > 50 && major <= 51 {
        "JAVA 7"
    }
    else if major > 51 && major <= 52 {
        "JAVA 8"
    }
    else if major > 52 && major <= 53 {
        "JAVA 9"
    }
    else if major > 53 && major <= 54 {
        "JAVA 10"
    }
    else if major > 54 && major <= 55 {
        "JAVA 11"
    }
    else {
        "Unknown/Invalid java version"
    }

}

pub const ACC_PUBLIC: u16     = 0x0001; // Declared public; may be accessed from outside its package.
pub const ACC_PRIVATE: u16    =	0x0002; // Declared private; accessible only within the defining class and other classes belonging to the same nest
pub const ACC_PROTECTED: u16  = 0x0004; // Declared protected; may be accessed within subclasses.
pub const ACC_STATIC: u16     = 0x0008; // Declared static.
pub const ACC_FINAL: u16      = 0x0010; // Declared final; no subclasses allowed.
pub const ACC_SUPER: u16      = 0x0020; // Treat superclass methods specially when invoked by the invokespecial instruction.
pub const ACC_BRIDGE: u16     =	0x0040; // A bridge method, generated by the compiler.
pub const ACC_VARARGS: u16    =	0x0080; // Declared with variable number of arguments.
pub const ACC_NATIVE: u16     =	0x0100; //	Declared native; implemented in a language other than the Java programming language.
pub const ACC_INTERFACE: u16  = 0x0200; // Is an interface, not a class.
pub const ACC_ABSTRACT: u16   = 0x0400; // Declared abstract; must not be instantiated.
pub const ACC_STRICT: u16     = 0x0800; // Declared strictfp; floating-point mode is FP-strict. 
pub const ACC_SYNTHETIC:  u16 = 0x1000; // Declared synthetic; not present in the source code.
pub const ACC_ANNOTATION: u16 = 0x2000; // Declared as an annotation type.
pub const ACC_ENUM: u16       = 0x4000; // Declared as an enum type.
pub const ACC_MODULE: u16     = 0x8000; // Is a module, not a class or interface.

pub fn access_flags_to_str(access_flags: u16) -> String {

    let mut access = String::new();

    if access_flags & ACC_PUBLIC     == 0x0001 { access += "ACC_PUBLIC " }
    if access_flags & ACC_PRIVATE    == 0x0002 { access += "ACC_PRIVATE " }
    if access_flags & ACC_PROTECTED  == 0x0004 { access += "ACC_PROTECTED " }
    if access_flags & ACC_STATIC     == 0x0008 { access += "ACC_STATIC " }

    if access_flags & ACC_FINAL      == 0x0010 { access += "ACC_FINAL " }
    if access_flags & ACC_SUPER      == 0x0020 { access += "ACC_SUPER " }
    if access_flags & ACC_BRIDGE     == 0x0040 { access += "ACC_BRIDGE " }
    if access_flags & ACC_VARARGS    == 0x0080 { access += "ACC_VARARGS " }

    if access_flags & ACC_NATIVE     == 0x0100 { access += "ACC_NATIVE " }
    if access_flags & ACC_INTERFACE  == 0x0200 { access += "ACC_INTERFACE " }
    if access_flags & ACC_ABSTRACT   == 0x0400 { access += "ACC_ABSTRACT " }
    if access_flags & ACC_STRICT     == 0x0800 { access += "ACC_STRICT " }

    if access_flags & ACC_SYNTHETIC  == 0x1000 { access += "ACC_SYNTHETIC " }
    if access_flags & ACC_ANNOTATION == 0x2000 { access += "ACC_ANNOTATION " }
    if access_flags & ACC_ENUM       == 0x4000 { access += "ACC_ENUM " }
    if access_flags & ACC_MODULE     == 0x8000 { access += "ACC_MODULE " }

    access

}
