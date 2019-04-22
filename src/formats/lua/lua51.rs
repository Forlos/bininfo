use super::*;
use crate::Problem;

const LUA_TNIL:     u8 = 0;
const LUA_TBOOLEAN: u8 = 1;
const LUA_TNUMBER:  u8 = 3;
const LUA_TSTRING:  u8 = 4;


#[derive(Debug, Pread)]
pub struct Header {
    pub format:        u8,
    pub endianness:    u8,
    pub int_sz:        u8,
    pub size_t_sz:     u8,
    pub intr_sz:       u8,
    pub lua_number_sz: u8,
    pub integral_flag: u8,
}

#[derive(Debug)]
pub struct Constant {
    const_type: u8,
    constant:   Const,
}

#[derive(Debug)]
pub enum Const {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Debug)]
pub struct Local {
    var_name: String,
    start_pc: u64,
    end_pc:   u64,
}

#[derive(Debug)]
pub struct Function_block {
    /// This  source name  is  specified  only in  the top-level function; in other functions, this field consists only of a Size_t with the value 0.
    name_sz:       u64,
    name:          String,
    line_def:      u64,
    last_line_def: u64,
    n_upvalues:    u8,
    n_params:      u8,
    is_varag:      u8,
    max_stack_sz:  u8,

    code_sz:       u64,

    consts_sz:     u64,
    consts:        Vec<Constant>,

    funcs_sz:      u64,
    funcs:         Vec<Inner_function_block>,

    source:        Source_line,
    locals:        Locals,
    upvalues:      Upvalues,
}

#[derive(Debug)]
struct Inner_function_block {
    name_sz:       u64,
    line_def:      u64,
    last_line_def: u64,
    n_upvalues:    u8,
    n_params:      u8,
    is_varag:      u8,
    max_stack_sz:  u8,

    code_sz:       u64,

    consts_sz:     u64,
    consts:        Vec<Constant>,

    funcs_sz:      u64,
    funcs:         Vec<Inner_function_block>,

    source:        Source_line,
    locals:        Locals,
    upvalues:      Upvalues,
}

impl Inner_function_block {
    fn parse(buf: &[u8], offset: &mut usize, int_sz: u8, lua_number_sz: u8, size_t_sz: u8, intr_sz: u8, endian: scroll::Endian) -> Result<Self, Error> {

        let mut name_sz = 0;
        if size_t_sz == 4 {
            name_sz = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        if size_t_sz == 8 {
            name_sz = buf.gread_with::<u64>(offset, endian)?;
        }
        println!("Name: {:#X} {:#X}", name_sz, *offset);

        let mut line_def = 0;
        let mut last_line_def = 0;
        if int_sz == 4 {
            line_def      = buf.gread_with::<u32>(offset, endian)? as u64;
            last_line_def = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if int_sz == 8 {
            line_def      = buf.gread_with::<u64>(offset, endian)?;
            last_line_def = buf.gread_with::<u64>(offset, endian)?;
        }

        let n_upvalues    = buf.gread::<u8>(offset)?;
        let n_params      = buf.gread::<u8>(offset)?;
        let is_varag      = buf.gread::<u8>(offset)?;
        let max_stack_sz  = buf.gread::<u8>(offset)?;

        let mut code_sz = 0;
        if int_sz == 4 {
            code_sz = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        if int_sz == 8 {
            code_sz = buf.gread_with::<u64>(offset, endian)?;
        }
        *offset += code_sz as usize * intr_sz as usize;

        let mut consts_sz = 0;
        if int_sz == 4 {
            consts_sz = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        if int_sz == 8 {
            consts_sz = buf.gread_with::<u64>(offset, endian)?;
        }

        let mut consts = Vec::with_capacity(consts_sz as usize);
        for _ in 0..consts_sz as usize {
            let const_type = buf.gread::<u8>(offset)?;
            match const_type {
                LUA_TNIL     => consts.push( Constant { const_type, constant: Const::Nil } ),
                LUA_TBOOLEAN => {
                    let constant = match buf.gread::<u8>(offset)? {
                        0 => false,
                        1 => true,
                        _ => return Err(Error::from(Problem::Msg(format!("Invalid value for LUA_TBOOLEAN")))),
                    };
                    consts.push( Constant { const_type, constant: Const::Boolean(constant) } )
                },
                LUA_TNUMBER  => {
                    let constant = match lua_number_sz {
                        4 => buf.gread_with::<f32>(offset, endian)? as f64,
                        8 => buf.gread_with::<f64>(offset, endian)?,
                        _ => return Err(Error::from(Problem::Msg(format!("Invalid size of Lua Number")))),
                    };
                    consts.push( Constant { const_type, constant: Const::Number(constant) } )
                },
                LUA_TSTRING  => {
                    if size_t_sz == 4 {
                        buf.gread_with::<u32>(offset, endian)? as u64;
                    }
                    else if size_t_sz == 8 {
                        buf.gread_with::<u64>(offset, endian)?;
                    }
                    let tstring = buf.gread::<&str>(offset)?.to_string();
                    consts.push( Constant { const_type, constant: Const::String(tstring) } )
                },
                _ => return Err(Error::from(Problem::Msg(format!("Invalid value for const type: {}", const_type)))),
            }
        }


        let mut funcs_sz = 0;
        if int_sz == 4 {
            funcs_sz = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if int_sz == 8 {
            funcs_sz = buf.gread_with::<u64>(offset, endian)?;
        }
        println!("Func: {:#X} {:#X}", funcs_sz, *offset);

        let mut funcs = Vec::with_capacity(funcs_sz as usize);
        for _ in 0..funcs_sz as usize {
            funcs.push(Inner_function_block::parse(buf, offset, int_sz,  lua_number_sz, size_t_sz, intr_sz, endian)?);
        }

        let mut source_sz = 0;
        if int_sz == 4 {
            source_sz = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if int_sz == 8 {
            source_sz = buf.gread_with::<u64>(offset, endian)?;
        }

        let mut source = Vec::with_capacity(source_sz as usize);
        for _ in 0..source_sz as usize {
            if int_sz == 4 { source.push(buf.gread_with::<u32>(offset, endian)? as u64); }
            else if int_sz == 8 { source.push(buf.gread_with::<u64>(offset, endian)?); }
        }


        let mut locals_sz = 0;
        if int_sz == 4 {
            locals_sz = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if int_sz == 8 {
            locals_sz = buf.gread_with::<u64>(offset, endian)?;
        }

        let mut locals = Vec::with_capacity(locals_sz as usize);
        for _ in 0..locals_sz as usize {
            if size_t_sz == 4 {
                buf.gread_with::<u32>(offset, endian)? as u64;
            }
            else if size_t_sz == 8 {
                buf.gread_with::<u64>(offset, endian)?;
            }
            let var_name = buf.gread::<&str>(offset)?.to_string();
            let mut start_pc = 0;
            let mut end_pc = 0;
            if int_sz == 4 {
                start_pc = buf.gread_with::<u32>(offset, endian)? as u64;
                end_pc   = buf.gread_with::<u32>(offset, endian)? as u64;
            }
            else if int_sz == 8 {
                start_pc = buf.gread_with::<u64>(offset, endian)?;
                end_pc   = buf.gread_with::<u64>(offset, endian)?;
            }
            locals.push( Local { var_name, start_pc, end_pc });
        }

        let mut upvalues_sz = 0;
        if int_sz == 4 {
            upvalues_sz = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if int_sz == 8 {
            upvalues_sz = buf.gread_with::<u64>(offset, endian)?;
        }

        let mut upvalues = Vec::with_capacity(upvalues_sz as usize);
        for _ in 0..upvalues_sz as usize {
            if size_t_sz == 4 {
                buf.gread_with::<u32>(offset, endian)? as u64;
            }
            else if size_t_sz == 8 {
                buf.gread_with::<u64>(offset, endian)?;
            }
            upvalues.push(buf.gread::<&str>(offset)?.to_string());
        }

        Ok(Inner_function_block {
            name_sz,
            line_def,
            last_line_def,
            n_upvalues,
            n_params,
            is_varag,
            max_stack_sz,

            code_sz,

            consts_sz,
            consts,

            funcs_sz,
            funcs,

            source: Source_line { source_sz, source },
            locals: Locals { locals_sz, locals },
            upvalues: Upvalues { upvalues_sz, upvalues },
        })

    }
}

#[derive(Debug)]
struct Source_line {
    source_sz: u64,
    source:    Vec<u64>,
}

#[derive(Debug)]
struct Locals {
    locals_sz: u64,
    locals:    Vec<Local>,
}

#[derive(Debug)]
struct Upvalues {
    upvalues_sz: u64,
    upvalues:    Vec<String>,
}

#[derive(Debug)]
pub struct Lua51_info {
    header: Header,
    main_func: Function_block,
}

impl Lua51_info {

    pub fn print(&self) {
        use ansi_term::Color;

        println!("Format: {}", self.header.format);
        println!("Endian: {}", if self.header.endianness == 0 { "Big-endian" } else { "Little-endian" });
        println!("Size of integer:     {}",     Color::Purple.paint(self.header.int_sz.to_string()));
        println!("Size of size_t:      {}",     Color::Purple.paint(self.header.size_t_sz.to_string()));
        println!("Size of instruction: {}", Color::Purple.paint(self.header.intr_sz.to_string()));
        println!("Size of lua Number:  {}",  Color::Purple.paint(self.header.lua_number_sz.to_string()));
        println!("Integral flag: {}", self.header.integral_flag);
        println!();

        println!("Name: {}", Color::Yellow.paint(&self.main_func.name));

    }

}

pub fn parse(buf: &[u8], offset: &mut usize) -> Result<Lua51_info, Error> {

    let endianness = buf.pread::<u8>(*offset + 1)?;
    let endian = match endianness {
        1 => scroll::LE,
        0 => scroll::BE,
        _ => return Err(Error::from(Problem::Msg(format!("Invalid endian value: {}",
                                                         endianness)))),
    };

    let header = buf.gread_with::<Header>(offset, endian)?;

    let mut name_sz = 0;
    if header.size_t_sz == 4 {
        name_sz = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.size_t_sz == 8 {
        name_sz = buf.gread_with::<u64>(offset, endian)?;
    }
    let name = buf.gread::<&str>(offset)?.to_string();

    let mut line_def = 0;
    let mut last_line_def = 0;
    if header.int_sz == 4 {
        line_def      = buf.gread_with::<u32>(offset, endian)? as u64;
        last_line_def = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.int_sz == 8 {
        line_def      = buf.gread_with::<u64>(offset, endian)?;
        last_line_def = buf.gread_with::<u64>(offset, endian)?;
    }

    let n_upvalues    = buf.gread::<u8>(offset)?;
    let n_params      = buf.gread::<u8>(offset)?;
    let is_varag      = buf.gread::<u8>(offset)?;
    let max_stack_sz  = buf.gread::<u8>(offset)?;

    let mut code_sz = 0;
    if header.int_sz == 4 {
       code_sz = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.int_sz == 8 {
        code_sz = buf.gread_with::<u64>(offset, endian)?;
    }
    *offset += code_sz as usize * header.intr_sz as usize;

    let mut consts_sz = 0;
    if header.int_sz == 4 {
        consts_sz = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.int_sz == 8 {
        consts_sz = buf.gread_with::<u64>(offset, endian)?;
    }

    let mut consts = Vec::with_capacity(consts_sz as usize);
    for _ in 0..consts_sz as usize {
        let const_type = buf.gread::<u8>(offset)?;
        match const_type {
            LUA_TNIL     => consts.push( Constant { const_type, constant: Const::Nil } ),
            LUA_TBOOLEAN => {
                let constant = match buf.gread::<u8>(offset)? {
                    0 => false,
                    1 => true,
                    _ => return Err(Error::from(Problem::Msg(format!("Invalid value for LUA_TBOOLEAN")))),
                };
                consts.push( Constant { const_type, constant: Const::Boolean(constant) } )
            },
            LUA_TNUMBER  => {
                let constant = match header.lua_number_sz {
                    4 => buf.gread_with::<f32>(offset, endian)? as f64,
                    8 => buf.gread_with::<f64>(offset, endian)?,
                    _ => return Err(Error::from(Problem::Msg(format!("Invalid size of Lua Number")))),
                };
                consts.push( Constant { const_type, constant: Const::Number(constant) } )
            },
            LUA_TSTRING  => {
                if header.size_t_sz == 4 {
                    buf.gread_with::<u32>(offset, endian)? as u64;
                }
                else if header.size_t_sz == 8 {
                    buf.gread_with::<u64>(offset, endian)?;
                }
                let tstring = buf.gread::<&str>(offset)?.to_string();
                consts.push( Constant { const_type, constant: Const::String(tstring) } )
            },
            _ => return Err(Error::from(Problem::Msg(format!("Invalid value for const type: {}", const_type)))),
        }
    }

    let mut funcs_sz = 0;
    if header.int_sz == 4 {
        funcs_sz = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.int_sz == 8 {
        funcs_sz = buf.gread_with::<u64>(offset, endian)?;
    }

    let mut funcs = Vec::with_capacity(funcs_sz as usize);
    for _ in 0..funcs_sz as usize {
        funcs.push(Inner_function_block::parse(buf, offset, header.int_sz, header.lua_number_sz, header.size_t_sz, header.intr_sz, endian)?);
    }

    let mut source_sz = 0;
    if header.int_sz == 4 {
        source_sz = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.int_sz == 8 {
        source_sz = buf.gread_with::<u64>(offset, endian)?;
    }

    let mut source = Vec::with_capacity(source_sz as usize);
    for _ in 0..source_sz as usize {
        if header.int_sz == 4 { source.push(buf.gread_with::<u32>(offset, endian)? as u64); }
        else if header.int_sz == 8 { source.push(buf.gread_with::<u64>(offset, endian)?); }
    }


    let mut locals_sz = 0;
    if header.int_sz == 4 {
        locals_sz = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.int_sz == 8 {
        locals_sz = buf.gread_with::<u64>(offset, endian)?;
    }

    let mut locals = Vec::with_capacity(locals_sz as usize);
    for _ in 0..locals_sz as usize {
        if header.size_t_sz == 4 {
            buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if header.size_t_sz == 8 {
            buf.gread_with::<u64>(offset, endian)?;
        }
        let var_name = buf.gread::<&str>(offset)?.to_string();
        let mut start_pc = 0;
        let mut end_pc = 0;
        if header.int_sz == 4 {
            start_pc = buf.gread_with::<u32>(offset, endian)? as u64;
            end_pc   = buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if header.int_sz == 8 {
            start_pc = buf.gread_with::<u64>(offset, endian)?;
            end_pc   = buf.gread_with::<u64>(offset, endian)?;
        }
        locals.push( Local { var_name, start_pc, end_pc });
    }

    let mut upvalues_sz = 0;
    if header.int_sz == 4 {
        upvalues_sz = buf.gread_with::<u32>(offset, endian)? as u64;
    }
    else if header.int_sz == 8 {
        upvalues_sz = buf.gread_with::<u64>(offset, endian)?;
    }

    let mut upvalues = Vec::with_capacity(upvalues_sz as usize);
    for _ in 0..upvalues_sz as usize {
        if header.size_t_sz == 4 {
            buf.gread_with::<u32>(offset, endian)? as u64;
        }
        else if header.size_t_sz == 8 {
            buf.gread_with::<u64>(offset, endian)?;
        }
        upvalues.push(buf.gread::<&str>(offset)?.to_string());
    }

    let main_func = Function_block {
        name_sz,
        name,
        line_def,
        last_line_def,
        n_upvalues,
        n_params,
        is_varag,
        max_stack_sz,

        code_sz,

        consts_sz,
        consts,

        funcs_sz,
        funcs,

        source: Source_line { source_sz, source },
        locals: Locals { locals_sz, locals },
        upvalues: Upvalues { upvalues_sz, upvalues },
    };


    Ok(Lua51_info {
        header,

        main_func
    })

}
