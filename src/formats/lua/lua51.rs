use super::*;
use crate::Problem;
use crate::format::{fmt_indentln, fmt_with_indentln};

const LUA_TNIL:     u8 = 0;
const LUA_TBOOLEAN: u8 = 1;
const LUA_TNUMBER:  u8 = 3;
const LUA_TSTRING:  u8 = 4;

const VARARG_HASARG:   u8 = 1;
const VARARG_ISVARARG: u8 = 2;
const VARARG_NEEDSARG: u8 = 4;

fn vararg_to_str(vararg: u8) -> String {

    let mut arg = String::new();

    if vararg & VARARG_HASARG   == 1 { arg += "VARARG_HASARG " }
    if vararg & VARARG_ISVARARG == 1 { arg += "VARARG_ISVARARG " }
    if vararg & VARARG_NEEDSARG == 1 { arg += "VARARG_NEEDSARG " }

    arg

}

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

#[derive(Debug, AsRefStr)]
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
    is_vararg:     u8,
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
    is_vararg:     u8,
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
        let is_vararg      = buf.gread::<u8>(offset)?;
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
            is_vararg,
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

    fn print(&self, trim_lines: usize, indent: usize) {
        use ansi_term::Color;
        use prettytable::Table;

        let mut idt = String::with_capacity(indent);
        for _ in 0..indent {
            idt.push(' ');
        }
        fmt_with_indentln(format!("{}", Color::White.paint("---Inner function---")), &idt);
        fmt_with_indentln(format!("Number of upvalues: {}", Color::Purple.paint(self.n_upvalues.to_string())), &idt);
        fmt_with_indentln(format!("Number of params:   {}", Color::Purple.paint(self.n_params.to_string())), &idt);
        fmt_with_indentln(format!("Vararg: {}", vararg_to_str(self.is_vararg)), &idt);
        fmt_with_indentln(format!("Max stack size: {}", Color::Green.paint(self.max_stack_sz.to_string())), &idt);
        fmt_with_indentln(format!("Code size: {}", Color::Green.paint(self.code_sz.to_string())), &idt);
        println!();

        //
        // CONSTANTS
        //
        if self.consts_sz >= 1 {
            fmt_with_indentln(format!("{}({})",
                                      Color::White.underline().paint("Constants"),
                                      self.consts_sz), &idt);

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![idt, "Idx", "Type", "Value"]);

            for (i, entry) in self.consts.iter().enumerate() {
                if i == trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    idt,
                    i,
                    entry.constant.as_ref(),
                    match &entry.constant {
                        Const::Nil        => Color::Cyan.paint("Nil"),
                        Const::Boolean(b) => if *b { Color::Green.paint(b.to_string()) } else { Color::Red.paint(b.to_string()) },
                        Const::Number(n)  => Color::Blue.paint(n.to_string()),
                        Const::String(s)  => Color::Yellow.paint(s),
                    },
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_with_indentln(format!("Output trimmed..."), &idt);
            }
            println!();
        }

        //
        // FUNCTIONS
        //
        if self.funcs_sz >= 1 {
            fmt_with_indentln(format!("{}({})",
                                      Color::White.underline().paint("Functions"),
                                      self.funcs_sz), &idt);
            for func in &self.funcs {
                func.print(trim_lines, indent + 2);
            }
            println!();
        }

        //
        // SOURCE LINES
        //
        if self.source.source_sz >= 1 {
            fmt_with_indentln(format!("{}({})",
                                      Color::White.underline().paint("SourceLines"),
                                      self.source.source_sz), &idt);
            println!();
        }
        //
        // LOCALS
        //
        if self.locals.locals_sz >= 1 {
            fmt_with_indentln(format!("{}({})",
                                      Color::White.underline().paint("Locals"),
                                      self.locals.locals_sz), &idt);
            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![idt, "Idx", "Name", "Start_pc", "End_pc"]);

            for (i, entry) in self.locals.locals.iter().enumerate() {
                if i == trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    idt,
                    i,
                    Fy->entry.var_name,
                    Fg->entry.start_pc,
                    Fr->entry.end_pc,
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }

        //
        // UPVALUES
        //
        if self.upvalues.upvalues_sz >= 1 {
            fmt_with_indentln(format!("{}({})",
                                      Color::White.underline().paint("Upvalues"),
                                      self.upvalues.upvalues_sz), &idt);
            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![idt, "Idx", "Name"]);

            for (i, entry) in self.upvalues.upvalues.iter().enumerate() {
                if i == trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    idt,
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

    pub fn print(&self, trim_lines: usize) {
        use ansi_term::Color;
        use prettytable::Table;

        println!("Format: {}", self.header.format);
        println!("Endian: {}", if self.header.endianness == 0 { "Big-endian" } else { "Little-endian" });
        println!("Size of integer:     {}", Color::Green.paint(self.header.int_sz.to_string()));
        println!("Size of size_t:      {}", Color::Green.paint(self.header.size_t_sz.to_string()));
        println!("Size of instruction: {}", Color::Green.paint(self.header.intr_sz.to_string()));
        println!("Size of lua Number:  {}", Color::Green.paint(self.header.lua_number_sz.to_string()));
        println!("Integral flag: {}", self.header.integral_flag);
        println!();

        println!("Name: {}", Color::Yellow.paint(&self.main_func.name));
        println!("Number of upvalues: {}", Color::Purple.paint(self.main_func.n_upvalues.to_string()));
        println!("Number of params:   {}", Color::Purple.paint(self.main_func.n_params.to_string()));
        println!("Varag: {}", vararg_to_str(self.main_func.is_vararg));
        println!("Max stack size: {}", Color::Green.paint(self.main_func.max_stack_sz.to_string()));
        println!("Code size: {}", Color::Green.paint(self.main_func.code_sz.to_string()));
        println!();

        //
        // CONSTANTS
        //
        if self.main_func.consts_sz >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("Constants"),
                     self.main_func.consts_sz);

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Type", "Value"]);

            for (i, entry) in self.main_func.consts.iter().enumerate() {
                if i == trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i,
                    entry.constant.as_ref(),
                    match &entry.constant {
                        Const::Nil        => Color::Cyan.paint("Nil"),
                        Const::Boolean(b) => if *b { Color::Green.paint(b.to_string()) } else { Color::Red.paint(b.to_string()) },
                        Const::Number(n)  => Color::Blue.paint(n.to_string()),
                        Const::String(s)  => Color::Yellow.paint(s),
                    },
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }

        //
        // FUNCTIONS
        //
        if self.main_func.funcs_sz >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("Functions"),
                     self.main_func.funcs_sz);
            for func in &self.main_func.funcs {
                func.print(trim_lines, 2);
            }
            println!();
        }

        //
        // SOURCE LINES
        //
        if self.main_func.source.source_sz >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("SourceLines"),
                     self.main_func.source.source_sz);
            println!();
        }
        //
        // LOCALS
        //
        if self.main_func.locals.locals_sz >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("Locals"),
                     self.main_func.locals.locals_sz);
            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Name", "Start_pc", "End_pc"]);

            for (i, entry) in self.main_func.locals.locals.iter().enumerate() {
                if i == trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
                    i,
                    Fy->entry.var_name,
                    Fg->entry.start_pc,
                    Fr->entry.end_pc,
                ]);
            }
            table.printstd();
            if trimmed {
                fmt_indentln(format!("Output trimmed..."));
            }
            println!();
        }

        //
        // UPVALUES
        //
        if self.main_func.upvalues.upvalues_sz >= 1 {
            println!("{}({})",
                     Color::White.underline().paint("Upvalues"),
                     self.main_func.upvalues.upvalues_sz);
            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .borders(' ')
                .column_separator(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Name"]);

            for (i, entry) in self.main_func.upvalues.upvalues.iter().enumerate() {
                if i == trim_lines {
                    trimmed = true;
                    break;
                }
                table.add_row(row![
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
    let is_vararg      = buf.gread::<u8>(offset)?;
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
        is_vararg,
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
