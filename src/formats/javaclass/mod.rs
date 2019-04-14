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
    attribute_tab:    Vec<Attributes>,
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
                                                                              Color::Yellow.paint(const_tab[*name_idx as usize].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", name_idx))),
            StringRef{string_idx}                                  => format!("\"{}\"({})",
                                                                              Color::Purple.paint(const_tab[*string_idx as usize].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", string_idx))),
            FieldRef{class_idx, nametype_idx}                      => format!("{}, {}",
                                                                              const_tab[*class_idx as usize].values_to_string(const_tab),
                                                                              const_tab[*nametype_idx as usize].values_to_string(const_tab)),
            MethodRef{class_idx, nametype_idx}                     => format!("{}, {}",
                                                                              const_tab[*class_idx as usize].values_to_string(const_tab),
                                                                              const_tab[*nametype_idx as usize].values_to_string(const_tab)),
            InterfaceMethodRef{class_idx, nametype_idx}            => format!("{}, {}",
                                                                              const_tab[*class_idx as usize].values_to_string(const_tab),
                                                                              const_tab[*nametype_idx as usize].values_to_string(const_tab)),
            NameAndType{name_idx, desc_idx}                        => format!("{}({}), {}({})",
                                                                              Color::Blue.paint(const_tab[*name_idx as usize].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", name_idx)),
                                                                              Color::Green.paint(const_tab[*desc_idx as usize].values_to_string(const_tab)),
                                                                              Color::Red.paint(format!("{}", desc_idx))),
            MethodHandle{kind, idx}                                => format!("{}, {}({})",
                                                                              ref_kind_to_str(*kind),
                                                                              const_tab[*idx as usize].values_to_string(const_tab),
                                                                              Color::Red.paint(format!("{}", idx))),
            MethodType{desc_idx}                                   => format!("{}({})",
                                                                              const_tab[*desc_idx as usize].values_to_string(const_tab),
                                                                              Color::Red.paint(format!("{}", desc_idx))),
            Dynamic{bootstrap_method_attr_idx, nametype_idx}       => format!("Bootstrap: {} {}",
                                                                              bootstrap_method_attr_idx,
                                                                              const_tab[*nametype_idx as usize].values_to_string(const_tab)),
            InvokeDynamic{bootstrap_method_attr_idx, nametype_idx} => format!("Bootstrap: {} {}",
                                                                              bootstrap_method_attr_idx,
                                                                              const_tab[*nametype_idx as usize].values_to_string(const_tab)),
            Module{name_idx}                                       => format!("{}({})",
                                                                              const_tab[*name_idx as usize].values_to_string(const_tab),
                                                                              Color::Red.paint(format!("{}", name_idx))),
            Package{name_idx}                                      => format!("{}({})",
                                                                              const_tab[*name_idx as usize].values_to_string(const_tab),
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
    attributes:   Vec<Attributes>,
}

#[derive(Debug)]
struct Method_info {
    access_flags: u16,
    name_idx:     u16,
    desc_idx:     u16,
    attr_count:   u16,
    attributes:   Vec<Attributes>,
}

// #[derive(Debug)]
// struct Attribute_info {
//     attr_name_idx: u16,
//     attr_len:      u32,
//     info:          Vec<u8>,
// }

#[derive(Debug, AsRefStr)]
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
        attributes:  Vec<Attributes>,
    },
    StackMapTable {
        name_idx:  u16,
        attr_len:  u32,
        n_entries: u16,
        entries:   Vec<StackMapFrame>
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
        params:       Vec<Parameter>,
    },
    Module {
        name_idx:     u16,
        attr_len:     u32,
        mod_name_idx: u16,
        mod_flags:    u16,
        mod_ver_idx:  u16,
        requires_cnt: u16,
        requires:     Vec<Require>,
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

impl Attributes {

    pub fn parse(buf: &[u8], offset: &mut usize, const_tab: &Vec<Constants>) -> Result<Attributes, Error> {
        use Attributes::*;

        let name_idx = buf.gread_with::<u16>(offset, scroll::BE)?;
        let attr_len = buf.gread_with::<u32>(offset, scroll::BE)?;
        let name = const_tab[name_idx as usize].values_to_string(const_tab);
        match name.as_str() {

            "ConstantValue" => Ok(ConstantValue{name_idx,attr_len,
                                                const_idx: buf.gread_with(offset, scroll::BE)?}),
            "Code" => {

                let max_stack   = buf.gread_with::<u16>(offset, scroll::BE)?;
                let max_locals  = buf.gread_with::<u16>(offset, scroll::BE)?;
                let code_length = buf.gread_with::<u32>(offset, scroll::BE)?;
                // Code is pretty useless there since we parse headers
                let code        = Vec::new();
                *offset += code_length as usize;

                let ex_tab_len = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut ex_tab = Vec::with_capacity(ex_tab_len as usize);
                for _ in 0..ex_tab_len as usize {
                    ex_tab.push(buf.gread_with(offset, scroll::BE)?);
                }
                let attr_count     = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut attributes = Vec::with_capacity(attr_count as usize);
                for _ in 0..attr_count as usize {
                    attributes.push(Attributes::parse(buf, offset, const_tab)?);
                }

                Ok(Code{name_idx,attr_len,
                        max_stack,
                        max_locals,
                        code_length,
                        code,
                        ex_tab_len,
                        ex_tab,
                        attr_count,
                        attributes,
                })
            },
            "StackMapTable" => {
                use StackMapFrame::*;

                let n_entries = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut entries = Vec::with_capacity(n_entries as usize);
                for _ in 0..n_entries as usize {
                    let frame_type = buf.gread_with::<u8>(offset, scroll::BE)?;
                    match frame_type {
                        // Tags in the range [128-246] are reserved for future use.
                        0...63 => {
                            entries.push(SameFrame(frame_type));
                        },
                        64...127 => {
                            let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
                            let stack = VerificationTypeInfo::parse(tag, buf, offset)?;
                            entries.push(SameLocals1StackItemFrame(frame_type, stack));
                        },
                        247 => {
                            let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
                            let offset_delta = buf.gread_with::<u16>(offset, scroll::BE)?;
                            let stack = VerificationTypeInfo::parse(tag, buf, offset)?;
                            entries.push(SameLocals1StackItemFrameExt(frame_type, offset_delta, stack));
                        },
                        248...250 => {
                            let offset_delta = buf.gread_with::<u16>(offset, scroll::BE)?;
                            entries.push(ChopFrame(frame_type, offset_delta));
                        },
                        251 => {
                            let offset_delta = buf.gread_with::<u16>(offset, scroll::BE)?;
                            entries.push(SameFrameExt(frame_type, offset_delta));
                        },
                        252...254 => {
                            let offset_delta = buf.gread_with::<u16>(offset, scroll::BE)?;
                            let mut locals = Vec::with_capacity(frame_type as usize - 251);
                            for _ in 0..frame_type as usize - 251 {
                                let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
                                locals.push(VerificationTypeInfo::parse(tag, buf, offset)?);
                            }
                            entries.push(AppendFrame(frame_type, offset_delta, locals));
                        },
                        255 => {
                            let offset_delta  = buf.gread_with::<u16>(offset, scroll::BE)?;
                            let n_locals      = buf.gread_with::<u16>(offset, scroll::BE)?;
                            let mut locals    = Vec::with_capacity(n_locals as usize);
                            for _ in 0..n_locals {
                                let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
                                locals.push(VerificationTypeInfo::parse(tag, buf, offset)?);
                            }
                            let n_stack_items = buf.gread_with::<u16>(offset, scroll::BE)?;
                            let mut stack_items = Vec::with_capacity(n_stack_items as usize);
                            for _ in 0..n_stack_items {
                                let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
                                stack_items.push(VerificationTypeInfo::parse(tag, buf, offset)?);
                            }
                            entries.push(FullFrame(frame_type, offset_delta, n_locals, locals, n_stack_items, stack_items));
                        },
                        _ => return Err(Error::from(Problem::Msg(format!("Invalid/Unsupported stack map frame")))),
                    }
                }

                Ok(StackMapTable{name_idx,attr_len,
                                 n_entries,
                                 entries,
                })

            },
            "Exceptions" => {

                let n_of_ex        = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut ex_idx_tab = Vec::with_capacity(n_of_ex as usize);
                for _ in 0..n_of_ex {
                    ex_idx_tab.push(buf.gread_with(offset, scroll::BE)?)
                }

                Ok(Exceptions{name_idx,attr_len,
                              n_of_ex,
                              ex_idx_tab,
                })
            },
            "InnerClasses" => {

                let n_of_classes = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut classes  = Vec::with_capacity(n_of_classes as usize);
                for _ in 0..n_of_classes {
                    classes.push(buf.gread_with(offset, scroll::BE)?)
                }

                Ok(InnerClasses{name_idx,attr_len,
                                n_of_classes,
                                classes,
                })
            },
            "EnclosingMethod" => {

                let class_idx  = buf.gread_with::<u16>(offset, scroll::BE)?;
                let method_idx = buf.gread_with::<u16>(offset, scroll::BE)?;

                Ok(EnclosingMethod{name_idx,attr_len,
                                   class_idx,
                                   method_idx,
                })
            },
            "Synthetic" => {
                Ok(Synthetic{name_idx,attr_len})
            },
            "Signature" => {

                let sig_idx = buf.gread_with::<u16>(offset, scroll::BE)?;

                Ok(Signature{name_idx,attr_len,
                             sig_idx,
                })
            },
            "SourceFile" => {

                let source_idx = buf.gread_with::<u16>(offset, scroll::BE)?;

                Ok(SourceFile{name_idx,attr_len,
                              source_idx,
                })
            },
            "SourceDebugExtension" => {

                let debug_ext = buf[*offset..*offset + attr_len as usize].to_vec();

                Ok(SourceDebugExtension{name_idx,attr_len,
                                        debug_ext,
                })
            },
            "LineNumberTable" => {

                let line_num_tab_len = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut line_num_tab = Vec::with_capacity(line_num_tab_len as usize);
                for _ in 0..line_num_tab_len {
                    line_num_tab.push(buf.gread_with(offset, scroll::BE)?);
                }

                Ok(LineNumberTable{name_idx,attr_len,
                                   line_num_tab_len,
                                   line_num_tab,
                })
            },
            "LocalVariableTable" => {

                let local_var_tab_len = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut local_var_tab = Vec::with_capacity(local_var_tab_len as usize);
                for _ in 0..local_var_tab_len {
                    local_var_tab.push(buf.gread_with(offset, scroll::BE)?);
                }

                Ok(LocalVariableTable{name_idx,attr_len,
                                      local_var_tab_len,
                                      local_var_tab,
                })
            },
            "LocalVariableTypeTable" => {

                let local_var_type_tab_len = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut local_var_type_tab = Vec::with_capacity(local_var_type_tab_len as usize);
                for _ in 0..local_var_type_tab_len {
                    local_var_type_tab.push(buf.gread_with(offset, scroll::BE)?);
                }

                Ok(LocalVariableTypeTable{name_idx,attr_len,
                                          local_var_type_tab_len,
                                          local_var_type_tab,
                })
            },
            "Deprecated" => {
                Ok(Deprecated{name_idx,attr_len})
            },
            "RuntimeVisibleAnnotations" => {

                let num_anno = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut anno = Vec::with_capacity(num_anno as usize);
                for _ in 0..num_anno {
                    anno.push(Annotation::parse(buf, offset)?);
                }

                Ok(RuntimeVisibleAnnotations{name_idx,attr_len,
                                             num_anno,
                                             anno,
                })
            },
            "RuntimeInvisibleAnnotations" => {

                let num_anno = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut anno = Vec::with_capacity(num_anno as usize);
                for _ in 0..num_anno {
                    anno.push(Annotation::parse(buf, offset)?);
                }

                Ok(RuntimeInvisibleAnnotations{name_idx,attr_len,
                                               num_anno,
                                               anno,
                })
            },
            "RuntimeVisibleParameterAnnotations" => {

                let num_params     = buf.gread_with::<u8>(offset, scroll::BE)?;
                let mut param_anno = Vec::with_capacity(num_params as usize);
                for _ in 0..num_params {
                    let num_anno = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let mut anno = Vec::with_capacity(num_anno as usize);
                    for _ in 0..num_anno {
                        anno.push(Annotation::parse(buf, offset)?);
                    }
                    let param = Parameter_annotations {
                        num_anno,
                        anno,
                    };
                    param_anno.push(param);
                }

                Ok(RuntimeVisibleParameterAnnotations{name_idx,attr_len,
                                                      num_params,
                                                      param_anno,
                })
            },
            "RuntimeInvisibleParameterAnnotations" => {

                let num_params     = buf.gread_with::<u8>(offset, scroll::BE)?;
                let mut param_anno = Vec::with_capacity(num_params as usize);
                for _ in 0..num_params {
                    let num_anno = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let mut anno = Vec::with_capacity(num_anno as usize);
                    for _ in 0..num_anno {
                        anno.push(Annotation::parse(buf, offset)?);
                    }
                    let param = Parameter_annotations {
                        num_anno,
                        anno,
                    };
                    param_anno.push(param);
                }

                Ok(RuntimeInvisibleParameterAnnotations{name_idx,attr_len,
                                                        num_params,
                                                        param_anno,
                })
            },
            "RuntimeVisibleTypeAnnotations" => {

                let num_anno = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut anno = Vec::with_capacity(num_anno as usize);
                for _ in 0..num_anno {
                    let type_anno = Type_annotation::parse(buf, offset)?;
                    anno.push(type_anno);
                }

                Ok(RuntimeVisibleTypeAnnotations{name_idx,attr_len,
                                                 num_anno,
                                                 anno,
                })
            },
            "RuntimeInvisibleTypeAnnotations" => {

                let num_anno = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut anno = Vec::with_capacity(num_anno as usize);
                for _ in 0..num_anno {
                    let type_anno = Type_annotation::parse(buf, offset)?;
                    anno.push(type_anno);
                }

                Ok(RuntimeInvisibleTypeAnnotations{name_idx,attr_len,
                                                   num_anno,
                                                   anno,
                })
            },
            "AnnotationDefault" => {

                let tag = buf.gread_with(offset, scroll::BE)?;
                let value = Value::parse(tag, buf, offset)?;
                let default_value = Element_value {
                    tag,
                    value,
                };

                Ok(AnnotationDefault{name_idx,attr_len,
                                     default_value,
                })
            },
            "BootstrapMethods" => {

                let n_bootstrap_methods   = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut bootstrap_methods = Vec::with_capacity(n_bootstrap_methods as usize);
                for _ in 0..n_bootstrap_methods {
                    let bootstrap_method_ref = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let n_bootstrap_args     = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let mut bootstrap_args   = Vec::with_capacity(n_bootstrap_args as usize);
                    for _ in 0..n_bootstrap_args {
                        bootstrap_args.push(buf.gread_with::<u16>(offset, scroll::BE)?);
                    }
                    bootstrap_methods.push(Bootstrap_method {bootstrap_method_ref, n_bootstrap_args, bootstrap_args});
                }

                Ok(BootstrapMethods{name_idx,attr_len,
                                    n_bootstrap_methods,
                                    bootstrap_methods,
                })
            },
            "MethodParameters" => {

                let params_count = buf.gread_with::<u8>(offset, scroll::BE)?;
                let mut params   = Vec::with_capacity(params_count as usize);
                for _ in 0..params_count {
                    params.push(buf.gread_with(offset, scroll::BE)?);
                }

                Ok(MethodParameters{name_idx,attr_len,
                                    params_count,
                                    params,
                })
            },
            "Module" => {

                let mod_name_idx = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mod_flags    = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mod_ver_idx  = buf.gread_with::<u16>(offset, scroll::BE)?;

                let requires_cnt = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut requires = Vec::with_capacity(requires_cnt as usize);
                for _ in 0..requires_cnt {
                    requires.push(buf.gread_with(offset, scroll::BE)?);
                }

                let exports_cnt  = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut exports  = Vec::with_capacity(exports_cnt as usize);
                for _ in 0..exports_cnt {
                    let idx        = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let flags      = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let to_cnt     = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let mut to_idx = Vec::with_capacity(to_cnt as usize);
                    for _ in 0..to_cnt {
                        to_idx.push(buf.gread_with::<u16>(offset, scroll::BE)?);
                    }
                    exports.push(Export {
                        idx,
                        flags,
                        to_cnt,
                        to_idx,
                    });
                }

                let opens_cnt    = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut opens    = Vec::with_capacity(opens_cnt as usize);
                for _ in 0..opens_cnt {
                    let idx        = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let flags      = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let to_cnt     = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let mut to_idx = Vec::with_capacity(to_cnt as usize);
                    for _ in 0..to_cnt {
                        to_idx.push(buf.gread_with::<u16>(offset, scroll::BE)?);
                    }
                    opens.push(Open {
                        idx,
                        flags,
                        to_cnt,
                        to_idx,
                    });
                }

                let uses_cnt     = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut uses_idx = Vec::with_capacity(uses_cnt as usize);
                for _ in 0..uses_cnt {
                    uses_idx.push(buf.gread_with::<u16>(offset, scroll::BE)?);
                }

                let provides_cnt = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut provides = Vec::with_capacity(provides_cnt as usize);
                for _ in 0..provides_cnt {
                    let idx          = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let with_cnt     = buf.gread_with::<u16>(offset, scroll::BE)?;
                    let mut with_idx = Vec::with_capacity(with_cnt as usize);
                    for _ in 0..with_cnt {
                        with_idx.push(buf.gread_with::<u16>(offset, scroll::BE)?);
                    }
                    provides.push(Provide {
                        idx,
                        with_cnt,
                        with_idx,
                    });
                }

                Ok(Module{name_idx,attr_len,
                          mod_name_idx,
                          mod_flags,
                          mod_ver_idx,
                          requires_cnt,
                          requires,
                          exports_cnt,
                          exports,
                          opens_cnt,
                          opens,
                          uses_cnt,
                          uses_idx,
                          provides_cnt,
                          provides,
                })
            },
            "ModulePackages" => {

                let package_cnt     = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut package_idx = Vec::with_capacity(package_cnt as usize);
                for _ in 0..package_cnt {
                    package_idx.push(buf.gread_with::<u16>(offset, scroll::BE)?);
                }

                Ok(ModulePackages{name_idx,attr_len,
                                  package_cnt,
                                  package_idx,
                })
            },
            "ModuleMainClass" => {

                let main_class_idx = buf.gread_with(offset, scroll::BE)?;

                Ok(ModuleMainClass{name_idx,attr_len,
                                   main_class_idx,
                })
            },
            "NestHost" => {

                let host_class_idx = buf.gread_with::<u16>(offset, scroll::BE)?;

                Ok(NestHost{name_idx,attr_len,
                            host_class_idx,
                })
            }
            "NestMembers" => {

                let n_of_classes = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut classes  = Vec::with_capacity(n_of_classes as usize);
                for _ in 0..n_of_classes {
                    classes.push(buf.gread_with::<u16>(offset, scroll::BE)?);
                }

                Ok(NestMembers{name_idx,attr_len,
                               n_of_classes,
                               classes,
                })
            }


            _ => return Err(Error::from(Problem::Msg(format!("Invalid/Unsupported attribute: {}", name)))),

        }

    }

    pub fn values_to_string(&self, const_tab: &Vec<Constants>) -> String {
        use Attributes::*;
        use ansi_term::Color;

        match self {

            ConstantValue{name_idx, attr_len: _, const_idx} => format!("{}: {}",
                                                                    const_tab[*name_idx  as usize].values_to_string(const_tab),
                                                                    const_tab[*const_idx as usize].values_to_string(const_tab)),
            Code{name_idx, attr_len, max_stack, max_locals, code_length, code, ex_tab_len, ex_tab, attr_count, attributes} => {
                format!("{}: ",
                        const_tab[*name_idx  as usize].values_to_string(const_tab))
            },
            StackMapTable{name_idx, attr_len: _, n_entries: _, entries} => {
                let mut format = String::new();
                for entry in entries {
                    format += &entry.value_to_string();
                }
                format!("{}: {}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            Exceptions{name_idx, attr_len: _, n_of_ex: _, ex_idx_tab} => {
                let mut format = String::new();
                for ex in ex_idx_tab {
                    format += &const_tab[*ex as usize].values_to_string(const_tab)
                }

                format!("{}: ",
                        const_tab[*name_idx  as usize].values_to_string(const_tab))
            },
            InnerClasses{name_idx: _, attr_len: _, n_of_classes: _, classes} => {
                let mut format = String::new();
                for class in classes {
                    format += &format!("Inner: {}, ", const_tab[class.inner_class_info_idx as usize].values_to_string(const_tab));
                    format += &format!("Outer: {}, ", const_tab[class.outer_class_info_idx as usize].values_to_string(const_tab));
                    format += &format!("Inner name: {}, ", Color::Yellow.paint(const_tab[class.inner_name_idx as usize].values_to_string(const_tab)));
                    format += &format!("Flags: {}\n", access_flags_to_str(class.inner_class_access_flags));
                }

                format!("{}",format)
            },
            EnclosingMethod{name_idx, attr_len: _, class_idx, method_idx} => {
                format!("{}: {} {}",
                        const_tab[*name_idx   as usize].values_to_string(const_tab),
                        const_tab[*class_idx  as usize].values_to_string(const_tab),
                        const_tab[*method_idx as usize].values_to_string(const_tab))
            },
            Synthetic{name_idx, attr_len: _, } => {
                format!("{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab))
            },
            Signature{name_idx, attr_len: _, sig_idx} => {
                format!("{}: {}",
                        const_tab[*name_idx as usize].values_to_string(const_tab),
                        Color::Yellow.paint(const_tab[*sig_idx  as usize].values_to_string(const_tab)))
            },
            SourceFile{name_idx, attr_len: _, source_idx} => {
                format!("{}: {}",
                        const_tab[*name_idx   as usize].values_to_string(const_tab),
                        Color::Yellow.paint(const_tab[*source_idx as usize].values_to_string(const_tab)))
            },
            SourceDebugExtension{name_idx, attr_len: _, debug_ext} => {
                format!("{}: {}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        std::str::from_utf8(debug_ext).unwrap())
            },
            LineNumberTable{name_idx, attr_len: _, line_num_tab_len: _, line_num_tab: _} => {
                format!("{}: ",
                        const_tab[*name_idx  as usize].values_to_string(const_tab))
            },
            LocalVariableTable{name_idx, attr_len: _, local_var_tab_len: _, local_var_tab} => {
                format!("{}: ",
                        const_tab[*name_idx  as usize].values_to_string(const_tab))
            },
            LocalVariableTypeTable{name_idx, attr_len: _, local_var_type_tab_len: _, local_var_type_tab} => {
                format!("{}: ",
                        const_tab[*name_idx  as usize].values_to_string(const_tab))
            },
            Deprecated{name_idx, attr_len: _, } => {
                format!("{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab))
            },
            RuntimeVisibleAnnotations{name_idx, attr_len: _, num_anno: _, anno} => {
                let mut format = String::new();
                for ann in anno {
                    format += &format!("{:?}", ann);
                }

                format!("{}:\n{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            RuntimeInvisibleAnnotations{name_idx, attr_len: _, num_anno: _, anno} => {
                let mut format = String::new();
                for ann in anno {
                    format += &format!("{:?}", ann);
                }

                format!("{}:\n{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            RuntimeVisibleParameterAnnotations{name_idx, attr_len: _, num_params: _, param_anno} => {
                let mut format = String::new();
                for ann in param_anno {
                    format += &format!("{:?}", ann);
                }

                format!("{}:\n{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            RuntimeInvisibleParameterAnnotations{name_idx, attr_len: _, num_params: _, param_anno} => {
                let mut format = String::new();
                for ann in param_anno {
                    format += &format!("{:?}", ann);
                }

                format!("{}:\n{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            RuntimeVisibleTypeAnnotations{name_idx, attr_len: _, num_anno: _, anno} => {
                let mut format = String::new();
                for ann in anno {
                    format += &format!("{:?}", ann);
                }

                format!("{}:\n{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            RuntimeInvisibleTypeAnnotations{name_idx, attr_len: _, num_anno: _, anno} => {
                let mut format = String::new();
                for ann in anno {
                    format += &format!("{:?}", ann);
                }

                format!("{}:\n{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            AnnotationDefault{name_idx, attr_len: _, default_value} => {
                format!("{}: {:?}",
                        const_tab[*name_idx as usize].values_to_string(const_tab),
                        default_value)
            },
            BootstrapMethods{name_idx: _, attr_len: _, n_bootstrap_methods: _, bootstrap_methods} => {
                let mut format = String::new();
                for boot in bootstrap_methods {
                    format += &format!("{}:\n", const_tab[boot.bootstrap_method_ref as usize].values_to_string(const_tab));
                    for arg in &boot.bootstrap_args {
                        format += &format!("{}\n", const_tab[*arg as usize].values_to_string(const_tab))
                    }
                    format.push('\n');
                }

                format!("{}",
                        format)
            },
            MethodParameters{name_idx, attr_len: _, params_count: _, params} => {
                let mut format = String::new();
                for param in params {
                    format += &format!("{}, {}\n",
                                       const_tab[param.name_idx as usize].values_to_string(const_tab),
                                       access_flags_to_str(param.access_flags))
                }

                format!("{}:\n{}",
                        const_tab[*name_idx  as usize].values_to_string(const_tab),
                        format)
            },
            Module{name_idx: _, attr_len: _, mod_name_idx, mod_flags, mod_ver_idx, requires_cnt: _, requires,
                   exports_cnt: _, exports, opens_cnt: _, opens, uses_cnt: _, uses_idx, provides_cnt: _, provides} => {
                let mut require = String::new();
                for req in requires {
                    require += &format!("  {} {} {}\n",
                                        const_tab[req.idx as usize].values_to_string(const_tab),
                                        access_flags_to_str(req.flags),
                                        Color::Blue.paint(const_tab[req.ver_idx as usize].values_to_string(const_tab)));
                }
                let mut export = String::new();
                for exp in exports {
                    let mut exps = String::new();
                    for id in &exp.to_idx {
                        exps += &format!("{}",
                                         const_tab[*id as usize].values_to_string(const_tab));
                    }
                    export += &format!("  {} {} {}\n",
                                       const_tab[exp.idx as usize].values_to_string(const_tab),
                                       access_flags_to_str(exp.flags),
                                       exps);
                }
                let mut open = String::new();
                for op in opens {
                    let mut ops = String::new();
                    for id in &op.to_idx {
                        ops += &format!("{}",
                                        const_tab[*id as usize].values_to_string(const_tab));
                    }
                    open += &format!("  {} {} {}\n",
                                     const_tab[op.idx as usize].values_to_string(const_tab),
                                     access_flags_to_str(op.flags),
                                     ops);
                }
                let mut used = String::new();
                for us in uses_idx {
                    used += &format!("  {}\n",
                                     const_tab[*us as usize].values_to_string(const_tab));
                }
                let mut provide = String::new();
                for prov in provides {
                    let mut provs = String::new();
                    for id in &prov.with_idx {
                        provs += &format!("{}",
                                          const_tab[*id as usize].values_to_string(const_tab));
                    }
                    provide += &format!("  {} {}\n",
                                        const_tab[prov.idx as usize].values_to_string(const_tab),
                                        provs);
                }

                format!("{}: {} {}\nRequire:\n{}Exports:\n{}Opens:\n{}Uses:\n{}Provides:\n{}",
                        const_tab[*mod_name_idx as usize].values_to_string(const_tab),
                        access_flags_to_str(*mod_flags),
                        const_tab[*mod_ver_idx as usize].values_to_string(const_tab),
                        require, export, open, used, provide)
            },
            ModulePackages{name_idx, attr_len: _, package_cnt: _, package_idx} => {
                let mut format = String::new();
                for package in package_idx {
                    format += &format!("{}", const_tab[*package as usize].values_to_string(const_tab))
                }

                format!("{}: ",
                        const_tab[*name_idx as usize].values_to_string(const_tab))
            },
            ModuleMainClass{name_idx, attr_len: _, main_class_idx} => {
                format!("{}: {}",
                        const_tab[*name_idx as usize].values_to_string(const_tab),
                        const_tab[*main_class_idx as usize].values_to_string(const_tab))
            },
            NestHost{name_idx: _, attr_len: _, host_class_idx} => {
                format!("{}",
                        const_tab[*host_class_idx as usize].values_to_string(const_tab))
            },
            NestMembers{name_idx, attr_len: _, n_of_classes: _, classes} => {
                let mut format = String::new();
                for class in classes {
                    format += &format!("{}\n",
                                      const_tab[*class as usize].values_to_string(const_tab))
                }

                format!("{}:\n{}",
                        const_tab[*name_idx as usize].values_to_string(const_tab),
                        format)
            },

        }

    }

}

#[derive(Debug)]
enum StackMapFrame {
    SameFrame(u8),
    SameLocals1StackItemFrame(u8, VerificationTypeInfo),
    SameLocals1StackItemFrameExt(u8, u16, VerificationTypeInfo),
    ChopFrame(u8, u16),
    SameFrameExt(u8, u16),
    AppendFrame(u8, u16, Vec<VerificationTypeInfo>),
    FullFrame(u8, u16, u16, Vec<VerificationTypeInfo>, u16, Vec<VerificationTypeInfo>),
}

impl StackMapFrame {

    pub fn value_to_string(&self) -> String {
        use StackMapFrame::*;

        match self {
            SameFrame(frame_type) => format!("SAME({})", frame_type),
            SameLocals1StackItemFrame(frame_type, stack) => {
                format!("SAME_LOCALS_1_STACK_ITEM({}) {:?}",
                        frame_type,
                        stack)
            },
            SameLocals1StackItemFrameExt(frame_type, offset_delta, stack) => {
                format!("SAME_LOCALS_1_STACK_ITEM_EXTENDED({}) {} {:?}",
                        frame_type,
                        offset_delta,
                        stack)
            },
            ChopFrame(frame_type, offset_delta) => {
                format!("CHOP({}) {}",
                        frame_type,
                        offset_delta)
            },
            SameFrameExt(frame_type, offset_delta) => {
                format!("SAME_FRAME_EXTENDED({}) {}",
                        frame_type,
                        offset_delta)
            },
            AppendFrame(frame_type, offset_delta, locals) => {
                format!("APPEND({}) {} {:?}",
                        frame_type,
                        offset_delta,
                        locals)
            },
            FullFrame(frame_type, offset_delta, n_locals, locals, n_items, items) => {
                format!("FULL_FRAME({}) {} {} {:?} {} {:?}",
                        frame_type,
                        offset_delta,
                        n_locals,locals,
                        n_items,items)
            }
        }
    }

}

#[derive(Debug)]
enum VerificationTypeInfo {
    TopVariable(u8),
    IntegerVariable(u8),
    FloatVariable(u8),
    LongVariable(u8),
    DoubleVariable(u8),
    NullVariable(u8),
    UninitializedThisVariable(u8),
    ObjectVariable(u8, u16),
    UninitializedVariable(u8, u16),
}

impl VerificationTypeInfo {

    pub fn parse(tag: u8, buf: &[u8], offset: &mut usize) -> Result<Self, Error> {
        use VerificationTypeInfo::*;

        match tag {
            0 => Ok(TopVariable(tag)),
            1 => Ok(IntegerVariable(tag)),
            2 => Ok(FloatVariable(tag)),
            3 => Ok(DoubleVariable(tag)),
            4 => Ok(LongVariable(tag)),
            5 => Ok(NullVariable(tag)),
            6 => Ok(UninitializedThisVariable(tag)),
            7 => Ok(ObjectVariable(tag, buf.gread_with(offset, scroll::BE)?)),
            8 => Ok(UninitializedVariable(tag, buf.gread_with(offset, scroll::BE)?)),
            _ => Err(Error::from(Problem::Msg(format!("Invalid tag for VerificationTypeInfo")))),
        }

    }

}

#[derive(Debug, Pread)]
struct Exception {
    start_pc:   u16,
    end_pc:     u16,
    handler_pc: u16,
    catch_type: u16,
}

#[derive(Debug, Pread)]
struct Inner_class {
    inner_class_info_idx:     u16,
    outer_class_info_idx:     u16,
    inner_name_idx:           u16,
    inner_class_access_flags: u16,
}

#[derive(Debug, Pread)]
struct Line_number {
    start_pc: u16,
    line_num: u16,
}

#[derive(Debug, Pread)]
struct Local_variable {
    start_pc: u16,
    len:      u16,
    name_idx: u16,
    desc_idx: u16,
    idx:      u16,
}

#[derive(Debug, Pread)]
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

impl Annotation {

    pub fn parse(buf: &[u8], offset: &mut usize) -> Result<Self, Error> {
        let type_idx = buf.gread_with::<u16>(offset, scroll::BE)?;
        let n_ele_val_pairs = buf.gread_with::<u16>(offset, scroll::BE)?;
        let mut ele_val_pairs = Vec::with_capacity(n_ele_val_pairs as usize);
        for _ in 0..n_ele_val_pairs {
            let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
            ele_val_pairs.push(Element_value {tag, value: Value::parse(tag, buf, offset)?})
        }
        Ok(Annotation {
            type_idx,
            n_ele_val_pairs,
            ele_val_pairs,
        })
    }

}

#[derive(Debug)]
enum Value {
    ConstValueIdx(u16),
    EnumConstValue(u16,u16),
    ClassInfoIdx(u16),
    AnnotationValue(Annotation),
    ArrayValue(u16, Vec<Element_value>),
}

impl Value {

    pub fn parse(tag: u8, buf: &[u8], offset: &mut usize) -> Result<Self, Error> {
        use Value::*;

        match tag {

            b'B' | b'C' | b'D' | b'F' | b'I' | b'J' | b'S' | b'Z' | b's' => Ok(ConstValueIdx(buf.gread_with(offset, scroll::BE)?)),
            b'e' => Ok(EnumConstValue(buf.gread_with(offset, scroll::BE)?, buf.gread_with(offset, scroll::BE)?)),
            b'c' => Ok(ClassInfoIdx(buf.gread_with(offset, scroll::BE)?)),
            b'@' => {
                Ok(AnnotationValue(Annotation::parse(buf, offset)?))
            },
            b'[' => {
                let num_values = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut values = Vec::with_capacity(num_values as usize);
                for _ in 0..num_values {
                    let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
                    values.push(Element_value { tag, value: Value::parse(tag, buf, offset)? })
                }
                Ok(ArrayValue(num_values, values))
            },
            _ => Err(Error::from(Problem::Msg(format!("Invalid/unsupported tag for Value: {}", tag))))

        }

    }

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
    target_type:     u8,
    target_info:     TargetInfo,
    target_path:     Type_path,
    type_idx:        u16,
    n_ele_var_pairs: u16,
    ele_var_pairs:   Vec<Name_element_value>,
}

impl Type_annotation {

    pub fn parse(buf: &[u8], offset: &mut usize) -> Result<Self, Error> {
        let target_type = buf.gread_with::<u8>(offset, scroll::BE)?;
        let target_info = TargetInfo::parse(target_type, buf, offset)?;
        let path_len    = buf.gread_with::<u8>(offset, scroll::BE)?;
        let mut path    = Vec::with_capacity(path_len as usize);
        for _ in 0..path_len {
            path.push(buf.gread_with(offset, scroll::BE)?);
        }
        let target_path       = Type_path {path_len, path};
        let type_idx          = buf.gread_with::<u16>(offset, scroll::BE)?;
        let n_ele_var_pairs   = buf.gread_with::<u16>(offset, scroll::BE)?;
        let mut ele_var_pairs = Vec::with_capacity(n_ele_var_pairs as usize);
        for _ in 0..n_ele_var_pairs {
            let name_idx = buf.gread_with::<u16>(offset, scroll::BE)?;
            let tag = buf.gread_with::<u8>(offset, scroll::BE)?;
            ele_var_pairs.push(Name_element_value{name_idx , value: Element_value { tag, value: Value::parse(tag, buf, offset)? }});
        }

        Ok(Type_annotation {
            target_type,
            target_info,
            target_path,
            type_idx,
            n_ele_var_pairs,
            ele_var_pairs,
        })
    }

}

#[derive(Debug)]
enum TargetInfo {
    TypeParameter(u8),
    Supertype(u16),
    TypeParameterBound(u8,u8),
    Empty,
    FormalParameter(u8),
    Throws(u16),
    LocalVar(u16, Vec<Local_var>),
    Catch(u16),
    Offset(u16),
    TypeArgument(u16,u8),
}

impl TargetInfo {

    pub fn parse(target_type: u8, buf: &[u8], offset: &mut usize) -> Result<Self, Error> {
        use TargetInfo::*;

        match target_type {

            0x00 | 0x01 => Ok(TypeParameter(buf.gread_with(offset, scroll::BE)?)),
            0x10 => Ok(Supertype(buf.gread_with(offset, scroll::BE)?)),
            0x11 | 0x12 => Ok(TypeParameterBound(buf.gread_with(offset, scroll::BE)?, buf.gread_with(offset, scroll::BE)?)),
            0x13 | 0x14 | 0x15 => Ok(Empty),
            0x16 => Ok(FormalParameter(buf.gread_with(offset, scroll::BE)?)),
            0x17 => Ok(Throws(buf.gread_with(offset, scroll::BE)?)),
            0x40 | 0x41 => {
                let tab_len = buf.gread_with::<u16>(offset, scroll::BE)?;
                let mut tab = Vec::with_capacity(tab_len as usize);
                for _ in 0..tab_len {
                    tab.push(buf.gread_with(offset, scroll::BE)?);
                }
                Ok(LocalVar(tab_len, tab))
            },
            0x42 => Ok(Catch(buf.gread_with(offset, scroll::BE)?)),
            0x43...0x46 => Ok(Offset(buf.gread_with(offset, scroll::BE)?)),
            0x48...0x4B => Ok(TypeArgument(buf.gread_with(offset, scroll::BE)?, buf.gread_with(offset, scroll::BE)?)),
            _ => Err(Error::from(Problem::Msg(format!("Invalid/unsupported target type for target info: {}", target_type)))),

        }

    }

}

#[derive(Debug, Pread)]
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

#[derive(Debug, Pread)]
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

#[derive(Debug, Pread)]
struct Parameter {
    name_idx: u16,
    access_flags: u16,
}


#[derive(Debug, Pread)]
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
        const_pool_tab.push(Constants::Ghost);

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
                // let attr_name_idx: u16 = buf.gread_with(offset, scroll::BE)?;
                // let attr_len: u32      = buf.gread_with(offset, scroll::BE)?;
                // let info                    = buf[*offset..*offset + attr_len as usize].to_vec();
                // *offset += attr_len as usize;
                // attributes.push(Attribute_info {attr_name_idx, attr_len, info});
                attributes.push(Attributes::parse(buf, offset, &const_pool_tab)?);
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
                // let attr_name_idx: u16 = buf.gread_with(offset, scroll::BE)?;
                // let attr_len: u32      = buf.gread_with(offset, scroll::BE)?;
                // let info                    = buf[*offset..*offset + attr_len as usize].to_vec();
                // *offset += attr_len as usize;
                // attributes.push(Attribute_info {attr_name_idx, attr_len, info});
                attributes.push(Attributes::parse(buf, offset, &const_pool_tab)?);
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
            // let attr_name_idx: u16 = buf.gread_with(offset, scroll::BE)?;
            // let attr_len: u32      = buf.gread_with(offset, scroll::BE)?;
            // let info                    = buf[*offset..*offset + attr_len as usize].to_vec();
            // *offset += attr_len as usize;
            // attribute_tab.push(Attribute_info {attr_name_idx, attr_len, info});
            attribute_tab.push(Attributes::parse(buf, offset, &const_pool_tab)?);
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
                 self.class_header.const_pool_tab[self.class_header.this_class as usize].values_to_string(&self.class_header.const_pool_tab));
        if self.class_header.super_class > 0 {
            println!("Super Class: {}",
                     self.class_header.const_pool_tab[self.class_header.super_class as usize].values_to_string(&self.class_header.const_pool_tab));
        }
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
                    i,
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
                    i,
                    self.class_header.const_pool_tab[*entry as usize].values_to_string(&self.class_header.const_pool_tab),
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
                let mut values = String::new();
                for attr in &entry.attributes {
                    values.push_str(&attr.values_to_string(&self.class_header.const_pool_tab));
                };
                table.add_row(row![
                    i,
                    self.class_header.const_pool_tab[entry.name_idx as usize].values_to_string(&self.class_header.const_pool_tab),
                    access_flags_to_str(entry.access_flags),
                    self.class_header.const_pool_tab[entry.desc_idx as usize].values_to_string(&self.class_header.const_pool_tab),
                    // format!("{:X?}", entry.attributes),
                    fill(&values, self.opt.wrap_chars),
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }

        //
        // METHODS
        //
        if self.class_header.method_tab.len() >= 1 {
            println!("{}", Color::White.underline().paint("Methods"));

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Name", "Flags", "Desc", "Attributes"]);

            for (i, entry) in self.class_header.method_tab.iter().enumerate() {
                if i == self.opt.trim_lines {
                    trimmed = true;
                    break;
                }
                let mut values = String::new();
                for attr in &entry.attributes {
                    values.push_str(&attr.values_to_string(&self.class_header.const_pool_tab));
                };
                table.add_row(row![
                    i,
                    self.class_header.const_pool_tab[entry.name_idx as usize].values_to_string(&self.class_header.const_pool_tab),
                    access_flags_to_str(entry.access_flags),
                    self.class_header.const_pool_tab[entry.desc_idx as usize].values_to_string(&self.class_header.const_pool_tab),
                    fill(&values, self.opt.wrap_chars),
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
                    i,
                    entry.as_ref(),
                    fill(&entry.values_to_string(&self.class_header.const_pool_tab), self.opt.wrap_chars),
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
