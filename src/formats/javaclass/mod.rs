#![allow(non_camel_case_types)]
include!("./class_constants.rs");

use failure::{Error};
use scroll::{self, Pread};

use crate::Opt;
use crate::Problem;
use crate::format::{fmt_indentln};

pub const CLASS_MAGIC: &'static [u8; CLASS_MAGIC_SIZE] = b"\xCA\xFE\xBA\xBE";
pub const CLASS_MAGIC_SIZE: usize = 4;

#[derive(Debug)]
struct Class_header {
    magic:            u32,
    minor_ver:        u16,
    major_ver:        u16,
    const_pool_count: u16,
    const_pool_tab:   Vec<Constants>,
    access_flags:     u16,
    this_class:       u16,
    super_class:      u16,
    interface_count:  u16,
    interface_tab:    Vec<u16>,
    field_count:      u16,
    field_tab:        Vec<Field_info>,
    method_count:     u16,
    method_tab:       Vec<Method_info>,
    attribute_count:  u16,
    attribute_tab:    Vec<Attribute_info>,
}

#[derive(Debug, AsRefStr)]
enum Constants {
    UTF8{len: u16, text: String},
    Integer(u32),
    Float(f32),
    Long(u64),
    Double(f64),
    ClassRef{name_idx: u16},
    StringRef{string_idx: u16},
    FieldRef{class_idx: u16, nametype_idx: u16},
    MethodRef{class_idx: u16, nametype_idx: u16},
    InterfaceMethodRef{class_idx: u16, nametype_idx: u16},
    NameAndType{name_idx: u16, desc_idx: u16},
    MethodHandle{kind: u8, idx: u16},
    MethodType{desc_idx: u16},
    Dynamic{bootstrap_method_attr_idx: u16, nametype_idx: u16},
    InvokeDynamic{bootstrap_method_attr_idx: u16, nametype_idx: u16},
    Module{name_idx: u16},
    Package{name_idx: u16},
    Ghost,
}

impl Constants {
    pub fn values_to_string(&self, const_tab: &Vec<Constants>) -> String {
        use Constants::*;
        use ansi_term::Color;

        match self {
            UTF8{len: _len, text}                                  => format!("{}", text),
            Integer(int)                                           => format!("{}", Color::Fixed(120).paint(format!("{}", int))),
            Float(float)                                           => format!("{}", Color::Fixed(120).paint(format!("{}", float))),
            Long(long)                                             => format!("{}", Color::Fixed(120).paint(format!("{}", long))),
            Double(double)                                         => format!("{}", Color::Fixed(120).paint(format!("{}", double))),
            ClassRef{name_idx}                                     => format!("{}({})",
                                                                              Color::Yellow.paint(const_tab[*name_idx as usize - 1].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", name_idx))),
            StringRef{string_idx}                                  => format!("\"{}\"({})",
                                                                              Color::Purple.paint(const_tab[*string_idx as usize - 1].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", string_idx))),
            FieldRef{class_idx, nametype_idx}                      => format!("{}, {}",
                                                                              const_tab[*class_idx as usize - 1].values_to_string(const_tab),
                                                                              const_tab[*nametype_idx as usize - 1].values_to_string(const_tab)),
            MethodRef{class_idx, nametype_idx}                     => format!("{}, {}",
                                                                              const_tab[*class_idx as usize - 1].values_to_string(const_tab),
                                                                              const_tab[*nametype_idx as usize - 1].values_to_string(const_tab)),
            InterfaceMethodRef{class_idx, nametype_idx}            => format!("{}, {}",
                                                                              const_tab[*class_idx as usize - 1].values_to_string(const_tab),
                                                                              const_tab[*nametype_idx as usize - 1].values_to_string(const_tab)),
            NameAndType{name_idx, desc_idx}                        => format!("{}({}), {}({})",
                                                                              Color::Blue.paint(const_tab[*name_idx as usize - 1].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", name_idx)),
                                                                              Color::Green.paint(const_tab[*desc_idx as usize - 1].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", desc_idx))),
            MethodHandle{kind, idx}                                => format!("{}, {}({})",
                                                                              ref_kind_to_str(*kind),
                                                                              const_tab[*idx as usize - 1].values_to_string(const_tab),
                                                                              Color::Red.paint(format!("{}", idx))),
            MethodType{desc_idx}                                   => format!("{}({})",
                                                                              const_tab[*desc_idx as usize - 1].values_to_string(const_tab),
                                                                              Color::Red.paint(format!("{}", desc_idx))),
            Dynamic{bootstrap_method_attr_idx, nametype_idx}       => format!("Bootstrap: {} {}",
                                                                              bootstrap_method_attr_idx,
                                                                              const_tab[*nametype_idx as usize - 1].values_to_string(const_tab)),
            InvokeDynamic{bootstrap_method_attr_idx, nametype_idx} => format!("Bootstrap: {} {}",
                                                                              bootstrap_method_attr_idx,
                                                                              const_tab[*nametype_idx as usize - 1].values_to_string(const_tab)),
            Module{name_idx}                                       => format!("{}({})",
                                                                              const_tab[*name_idx as usize - 1].values_to_string(const_tab),
                                                                              Color::Red.paint(format!("{}", name_idx))),
            Package{name_idx}                                      => format!("{}({})",
                                                                              const_tab[*name_idx as usize - 1].values_to_string(const_tab),
                                                                              Color::Red.paint(format!("{}", name_idx))),
            Ghost                                                  => format!(""),
        }

    }
}

#[derive(Debug)]
struct Field_info {
    access_flags: u16,
    name_idx:     u16,
    desc_idx:     u16,
    attr_count:   u16,
    attributes:   Vec<Attribute_info>,
}

#[derive(Debug)]
struct Method_info {
    access_flags: u16,
    name_idx:     u16,
    desc_idx:     u16,
    attr_count:   u16,
    attributes:   Vec<Attribute_info>,
}

#[derive(Debug)]
struct Attribute_info {
    attr_name_idx: u16,
    attr_len:      u32,
    info:          Vec<u8>,
}

#[derive(Debug)]
enum Attributes {
    ConstantValue {
        name_idx:  u16,
        attr_len:  u32,
        const_idx: u16,
    },
    Code {
        name_idx:    u16,
        attr_len:    u32,
        max_stack:   u16,
        max_locals:  u16,
        code_length: u32,
        code:        Vec<u8>,
        ex_tab_len:  u16,
        ex_tab:      Vec<Exception>,
        attr_count:  u16,
        attributes:  Vec<Attribute_info>,
    },
    StackMapTable {
        name_idx:  u16,
        attr_len:  u32,
        n_entries: u16,
        entries:   Vec<u8>
    },
    Exceptions {
        name_idx:   u16,
        attr_len:   u32,
        n_of_ex:    u16,
        ex_idx_tab: Vec<u16>,
    },
    InnerClasses {
        name_idx:     u16,
        attr_len:     u32,
        n_of_classes: u16,
        classes:      Vec<Inner_class>,
    },
    EnclosingMethod {
        name_idx:   u16,
        attr_len:   u32,
        class_idx:  u16,
        method_idx: u16,
    },
    Synthetic {
        name_idx: u16,
        attr_len: u32,
    },
    Signature {
        name_idx: u16,
        attr_len: u32,
        sig_idx: u16,
    },
    SourceFile {
        name_idx: u16,
        attr_len: u32,
        source_idx: u16,
    },
    SourceDebugExtension {
        name_idx: u16,
        attr_len: u32,
        debug_ext: Vec<u8>,
    },
    LineNumberTable {
        name_idx:         u16,
        attr_len:         u32,
        line_num_tab_len: u16,
        line_num_tab:     Vec<Line_number>,
    },
    LocalVariableTable {
        name_idx:          u16,
        attr_len:          u32,
        local_var_tab_len: u16,
        local_var_tab:     Vec<Local_variable>,
    },
    LocalVariableTypeTable {
        name_idx:               u16,
        attr_len:               u32,
        local_var_type_tab_len: u16,
        local_var_type_tab:     Vec<Local_variable_type>,
    },
    Deprecated {
        name_idx: u16,
        attr_len: u32,
    },
    RuntimeVisibleAnnotations {
        name_idx: u16,
        attr_len: u32,
        num_anno: u16,
        anno:     Vec<Annotation>,
    },
    RuntimeInvisibleAnnotations {
        name_idx: u16,
        attr_len: u32,
        num_anno: u16,
        anno:     Vec<Annotation>,
    },
    RuntimeVisibleParameterAnnotations {
        name_idx:   u16,
        attr_len:   u32,
        num_params: u8,
        param_anno: Vec<Parameter_annotations>,
    },
    RuntimeInvisibleParameterAnnotations {
        name_idx:   u16,
        attr_len:   u32,
        num_params: u8,
        param_anno: Vec<Parameter_annotations>,
    },
    RuntimeVisibleTypeAnnotations {
        name_idx: u16,
        attr_len: u32,
        num_anno: u16,
        anno: Vec<Type_annotation>,
    },
    RuntimeInvisibleTypeAnnotations {
        name_idx: u16,
        attr_len: u32,
        num_anno: u16,
        anno: Vec<Type_annotation>,
    },
    AnnotationDefault {
        name_idx: u16,
        attr_len: u32,
        default_value: Element_value,
    },
    BootstrapMethods {
        name_idx:            u16,
        attr_len:            u32,
        n_bootstrap_methods: u16,
        bootstrap_methods:   Vec<Bootstrap_method>,
    },
    MethodParameters {
        name_idx:     u16,
        attr_len:     u32,
        params_count: u8,
        params: Vec<Parameter>,
    },
    Module {
        name_idx:     u16,
        attr_len:     u32,
        mod_name_idx: u16,
        mod_flags:    u16,
        mod_ver_idx:  u16,
        required_cnt: u16,
        required:     Vec<Require>,
        exports_cnt:  u16,
        exports:      Vec<Export>,
        opens_cnt:    u16,
        opens:        Vec<Open>,
        uses_cnt:     u16,
        uses_idx:     Vec<u16>,
        provides_cnt: u16,
        provides:     Vec<Provide>,
    },
    ModulePackages {
        name_idx: u16,
        attr_len: u32,
        package_cnt: u16,
        package_idx: Vec<u16>,
    },
    ModuleMainClass {
        name_idx: u16,
        attr_len: u32,
        main_class_idx: u16,
    },
    NestHost {
        name_idx: u16,
        attr_len: u32,
        host_class_idx: u16,
    },
    NestMembers {
        name_idx: u16,
        attr_len: u32,
        n_of_classes: u16,
        classes: Vec<u16>,
    }
}

#[derive(Debug)]
struct Exception {
    start_pc:   u16,
    end_pc:     u16,
    handler_pc: u16,
    catch_type: u16,
}

#[derive(Debug)]
struct Inner_class {
    inner_class_info_idx:     u16,
    outer_class_info_idx:     u16,
    inner_name_idx:           u16,
    inner_class_access_flags: u16,
}

#[derive(Debug)]
struct Line_number {
    start_pc: u16,
    line_num: u16,
}

#[derive(Debug)]
struct Local_variable {
    start_pc: u16,
    len:      u16,
    name_idx: u16,
    desc_idx: u16,
    idx:      u16,
}

#[derive(Debug)]
struct Local_variable_type {
    start_pc: u16,
    len:      u16,
    name_idx: u16,
    sig_idx:  u16,
    idx:      u16,
}

#[derive(Debug)]
struct Annotation {
    type_idx:        u16,
    n_ele_val_pairs: u16,
    ele_val_pairs:   Vec<Element_value>,
}

#[derive(Debug)]
enum Value {
    ConstValueIdx(u16),
    EnumConstValue(u16,u16),
    ClassInfoIdx(u16),
    AnnotationValue(Annotation),
    ArrayValue(u16, Vec<Element_value>),
}

#[derive(Debug)]
struct Element_value {
    tag:   u8,
    value: Value,
}

#[derive(Debug)]
struct Parameter_annotations {
    num_anno: u16,
    anno:     Vec<Annotation>,
}

#[derive(Debug)]
struct Type_annotation {
    target_type: u8,
    target_info: Vec<TargetInfo>,
    target_path: Type_path,
    type_idx: u16,
    n_ele_var_pairs: u16,
    ele_var_pairs: Vec<Name_element_value>,
}

#[derive(Debug)]
enum TargetInfo {
    TypeParameter(u8),
    Supertype(u16),
    TypeParameterBound(u8,u8),
    Empty,
    FormatParameter(u8),
    Throws(u16),
    LocalVar(u16, Vec<Local_var>),
    Catch(u16),
    Offset(u16),
    TypeArgument(u16,u8),
}

#[derive(Debug)]
struct Local_var {
    start_pc: u16,
    len:      u16,
    idx:      u16,
}

#[derive(Debug)]
struct Type_path {
    path_len: u8,
    path:     Vec<Path>,
}

#[derive(Debug)]
struct Path {
    kind: u8,
    idx:  u8,
}

#[derive(Debug)]
struct Name_element_value {
    name_idx: u16,
    value:    Element_value,
}

#[derive(Debug)]
struct Bootstrap_method {
    bootstrap_method_ref: u16,
    n_bootstrap_args: u16,
    bootstrap_args: Vec<u16>,
}

#[derive(Debug)]
struct Parameter {
    name_idx: u16,
    access_flags: u16,
}


#[derive(Debug)]
struct Require {
    idx:     u16,
    flags:   u16,
    ver_idx: u16,
}

#[derive(Debug)]
struct Export {
    idx:    u16,
    flags:  u16,
    to_cnt: u16,
    to_idx: Vec<u16>,
}

#[derive(Debug)]
struct Open {
    idx:    u16,
    flags:  u16,
    to_cnt: u16,
    to_idx: Vec<u16>,
}

#[derive(Debug)]
struct Provide {
    idx:      u16,
    with_cnt: u16,
    with_idx: Vec<u16>,
}

#[derive(Debug)]
pub struct JavaClass {
    opt:     Opt,

    class_header: Class_header,
}

impl super::FileFormat for JavaClass {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        let offset = &mut 0;

        let magic: u32 = buf.gread_with(offset, scroll::BE)?;
        let minor_ver: u16 = buf.gread_with(offset, scroll::BE)?;
        let major_ver: u16 = buf.gread_with(offset, scroll::BE)?;
        let const_pool_count: u16 = buf.gread_with(offset, scroll::BE)?;
        let mut const_pool_tab = Vec::with_capacity(const_pool_count as usize);

        let mut i = 1;
        while i < const_pool_count {
            let tag: u8 = buf.gread(offset)?;

            match tag {

                CONSTANT_UTF8 => {
                    let len: u16 = buf.gread_with(offset, scroll::BE)?;
                    let text = std::str::from_utf8(&buf[*offset..*offset + len as usize])?
                        .to_string();
                    const_pool_tab.push(Constants::UTF8{len, text});
                    *offset += len as usize;
                },
                CONSTANT_INTEGER => {
                    const_pool_tab.push(Constants::Integer(buf.gread_with(offset, scroll::BE)?));
                },
                CONSTANT_FLOAT => {
                    const_pool_tab.push(Constants::Float(buf.gread_with(offset, scroll::BE)?));
                },
                CONSTANT_LONG => {
                    // All 8-byte constants take up two entries in the constant_pool table of the class file.
                    const_pool_tab.push(Constants::Long(buf.gread_with::<u64>(offset, scroll::BE)?));
                    const_pool_tab.push(Constants::Ghost);
                    i += 1;
                },
                CONSTANT_DOUBLE => {
                    // All 8-byte constants take up two entries in the constant_pool table of the class file.
                    const_pool_tab.push(Constants::Double(buf.gread_with::<f64>(offset, scroll::BE)?));
                    const_pool_tab.push(Constants::Ghost);
                    i += 1;
                },
                CONSTANT_CLASS => {
                    const_pool_tab.push(Constants::ClassRef{name_idx: buf.gread_with(offset, scroll::BE)?});
                },
                CONSTANT_STRING => {
                    const_pool_tab.push(Constants::StringRef{string_idx: buf.gread_with(offset, scroll::BE)?});
                },
                CONSTANT_FIELDREF => {
                    const_pool_tab.push(Constants::FieldRef {
                        class_idx: buf.gread_with(offset, scroll::BE)?,
                        nametype_idx: buf.gread_with(offset, scroll::BE)?,
                    });
                },
                CONSTANT_METHODREF => {
                    const_pool_tab.push(Constants::MethodRef {
                        class_idx: buf.gread_with(offset, scroll::BE)?,
                        nametype_idx: buf.gread_with(offset, scroll::BE)?,
                    });
                },
                CONSTANT_INTERFACEMETHODREF => {
                    const_pool_tab.push(Constants::InterfaceMethodRef {
                        class_idx: buf.gread_with(offset, scroll::BE)?,
                        nametype_idx: buf.gread_with(offset, scroll::BE)?,
                    });
                },
                CONSTANT_NAMEANDTYPE => {
                    const_pool_tab.push(Constants::NameAndType {
                        name_idx: buf.gread_with(offset, scroll::BE)?,
                        desc_idx: buf.gread_with(offset, scroll::BE)?,
                    });
                },
                CONSTANT_METHODHANDLE => {
                    const_pool_tab.push(Constants::MethodHandle {
                        kind: buf.gread_with(offset, scroll::BE)?,
                        idx: buf.gread_with(offset, scroll::BE)?,
                    });
                },
                CONSTANT_METHODTYPE => {
                    const_pool_tab.push(Constants::MethodType {desc_idx: buf.gread_with(offset, scroll::BE)?});
                },
                CONSTANT_DYNAMIC => {
                    const_pool_tab.push(Constants::Dynamic {
                        bootstrap_method_attr_idx: buf.gread_with(offset, scroll::BE)?,
                        nametype_idx: buf.gread_with(offset, scroll::BE)?,
                    });
                },
                CONSTANT_INVOKEDYNAMIC => {
                    const_pool_tab.push(Constants::InvokeDynamic {
                        bootstrap_method_attr_idx: buf.gread_with(offset, scroll::BE)?,
                        nametype_idx: buf.gread_with(offset, scroll::BE)?,
                    });
                },
                CONSTANT_MODULE => {
                    const_pool_tab.push(Constants::Module{name_idx: buf.gread_with(offset, scroll::BE)?});
                },
                CONSTANT_PACKAGE => {
                    const_pool_tab.push(Constants::Package{name_idx: buf.gread_with(offset, scroll::BE)?});
                },
                _ => return Err(Error::from(Problem::Msg(format!("Invalid constant tag: {} at: {:#X}, {:#X}", tag, offset, const_pool_tab.len())))),

            }

            i += 1;

        }

        let access_flags:    u16 = buf.gread_with(offset, scroll::BE)?;
        let this_class:      u16 = buf.gread_with(offset, scroll::BE)?;
        let super_class:     u16 = buf.gread_with(offset, scroll::BE)?;
        let interface_count: u16 = buf.gread_with(offset, scroll::BE)?;
        let mut interface_tab = Vec::with_capacity(interface_count as usize);

        for _ in 0..interface_count as usize {
            interface_tab.push(buf.gread_with(offset, scroll::BE)?);
        }

        let field_count: u16 = buf.gread_with(offset, scroll::BE)?;
        let mut field_tab = Vec::with_capacity(field_count as usize);
        for _ in 0..field_count as usize {
            let access_flags: u16    = buf.gread_with(offset, scroll::BE)?;
            let name_idx: u16        = buf.gread_with(offset, scroll::BE)?;
            let desc_idx: u16  = buf.gread_with(offset, scroll::BE)?;
            let attr_count: u16 = buf.gread_with(offset, scroll::BE)?;
            let mut attributes       = Vec::with_capacity(attr_count as usize);
            for _ in 0..attr_count as usize {
                let attr_name_idx: u16 = buf.gread_with(offset, scroll::BE)?;
                let attr_len: u32      = buf.gread_with(offset, scroll::BE)?;
                let info                    = buf[*offset..*offset + attr_len as usize].to_vec();
                *offset += attr_len as usize;
                attributes.push(Attribute_info {attr_name_idx, attr_len, info});
            }
            let field = Field_info {
                access_flags,
                name_idx,
                desc_idx,
                attr_count,
                attributes,
            };
            field_tab.push(field);
        }

        let method_count: u16 = buf.gread_with(offset, scroll::BE)?;
        let mut method_tab = Vec::with_capacity(method_count as usize);
        for _ in 0..method_count as usize {
            let access_flags: u16    = buf.gread_with(offset, scroll::BE)?;
            let name_idx: u16        = buf.gread_with(offset, scroll::BE)?;
            let desc_idx: u16  = buf.gread_with(offset, scroll::BE)?;
            let attr_count: u16 = buf.gread_with(offset, scroll::BE)?;
            let mut attributes       = Vec::with_capacity(attr_count as usize);
            for _ in 0..attr_count as usize {
                let attr_name_idx: u16 = buf.gread_with(offset, scroll::BE)?;
                let attr_len: u32      = buf.gread_with(offset, scroll::BE)?;
                let info                    = buf[*offset..*offset + attr_len as usize].to_vec();
                *offset += attr_len as usize;
                attributes.push(Attribute_info {attr_name_idx, attr_len, info});
            }
            let method = Method_info {
                access_flags,
                name_idx,
                desc_idx,
                attr_count,
                attributes,
            };
            method_tab.push(method);
        }

        let attribute_count: u16 = buf.gread_with(offset, scroll::BE)?;
        let mut attribute_tab = Vec::with_capacity(attribute_count as usize);
        for _ in 0..attribute_count as usize {
            let attr_name_idx: u16 = buf.gread_with(offset, scroll::BE)?;
            let attr_len: u32      = buf.gread_with(offset, scroll::BE)?;
            let info                    = buf[*offset..*offset + attr_len as usize].to_vec();
            *offset += attr_len as usize;
            attribute_tab.push(Attribute_info {attr_name_idx, attr_len, info});
        }


        let class_header = Class_header {
            magic,
            minor_ver,
            major_ver,
            const_pool_count,
            const_pool_tab,

            access_flags,
            this_class,
            super_class,
            interface_count,
            interface_tab,

            field_count,
            field_tab,

            method_count,
            method_tab,

            attribute_count,
            attribute_tab,

        };

        Ok(JavaClass {
            opt,

            class_header,
        })
    }

    fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;
        use prettytable::Table;
        use textwrap::fill;

        println!("{:#X?}", self);

        //
        // JAVA CLASS FILE
        //
        print!("JAVA_CLASS ");
        println!("{}", Color::Blue.paint(java_version_to_str(
            self.class_header.minor_ver,
            self.class_header.major_ver)));
        println!("{}", access_flags_to_str(self.class_header.access_flags));
        println!("Class: {}",
                 self.class_header.const_pool_tab[self.class_header.this_class as usize - 1].values_to_string(&self.class_header.const_pool_tab));
        println!("Super Class: {}",
                 self.class_header.const_pool_tab[self.class_header.super_class as usize - 1].values_to_string(&self.class_header.const_pool_tab));
        println!();

        //
        // CONSTANTS
        //
        if self.class_header.const_pool_tab.len() >= 1 {
            println!("{}", Color::White.underline().paint("Constants"));

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Type", "Value"]);

            for (i, entry) in self.class_header.const_pool_tab.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i + 1,
                    format!("{}", entry.as_ref()),
                    fill(&entry.values_to_string(&self.class_header.const_pool_tab), self.opt.wrap_chars),
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();

        }

        //
        // INTERFACES
        //
        if self.class_header.interface_tab.len() >= 1 {
            println!("{}", Color::White.underline().paint("Interfaces"));

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Name"]);

            for (i, entry) in self.class_header.interface_tab.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i + 1,
                    self.class_header.const_pool_tab[*entry as usize - 1].values_to_string(&self.class_header.const_pool_tab),
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }

        //
        // FIELDS
        //
        if self.class_header.field_tab.len() >= 1 {
            println!("{}", Color::White.underline().paint("Fields"));

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Name", "Flags", "Descriptor", "Attributes"]);

            for (i, entry) in self.class_header.field_tab.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i + 1,
                    self.class_header.const_pool_tab[entry.name_idx as usize - 1].values_to_string(&self.class_header.const_pool_tab),
                    access_flags_to_str(entry.access_flags),
                    self.class_header.const_pool_tab[entry.desc_idx as usize - 1].values_to_string(&self.class_header.const_pool_tab),
                    format!("{:X?}", entry.attributes),
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }

        //
        // ATTRIBUTES
        //
        if self.class_header.attribute_tab.len() >= 1 {
            println!("{}", Color::White.underline().paint("Attributes"));

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Name", "Info"]);

            for (i, entry) in self.class_header.attribute_tab.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i + 1,
                    self.class_header.const_pool_tab[entry.attr_name_idx as usize - 1].values_to_string(&self.class_header.const_pool_tab),
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }


        Ok(())

    }

}
