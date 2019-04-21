#![allow(non_camel_case_types)]
use crate::Opt;
use failure::{Error};
use scroll::{self, Pread};

pub const LUA_MAGIC: &'static [u8; LUA_MAGIC_SIZE] = b"\x1BLua";
pub const LUA_MAGIC_SIZE: usize = 4;

#[derive(Debug, Pread)]
struct Lua_header {
    magic:         u32,
    /// High hex digit is major version number
    /// Low hex digit is minor version number
    ver:           u8,
}

// struct Function_header {
//     line_start: u32,
//     line_end:   u32,
//     n_params:   u8,
//     varag_flag: u8,
//     n_regs:     u8,
//     n_instr:    u32,
// }

// struct Constants {
//     n_consts: u32,
//     consts:   Vec<Constant>,
// }

// struct Constant {
//     const_type: u8,
//     const_len:  u32,
//     const_data: Vec<u8>,
// }

#[derive(Debug)]
/// Each version of lua has a different header !???
pub struct Lua {
    opt: Opt,

    lua_header: Lua_header,
}

impl super::FileFormat for Lua {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        let offset = &mut 0;

        let lua_header = buf.gread(offset)?;

        Ok(Lua {
            opt,

            lua_header,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("Lua Bytecode {}.{}",
                 self.lua_header.ver >> 4,
                 self.lua_header.ver << 4 >> 4);
        println!("{:#X?}", self);

        Ok(())
    }


}
