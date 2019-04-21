use crate::Opt;
use failure::{Error};

pub const LUA_MAGIC: &'static [u8; LUA_MAGIC_SIZE] = b"\x1BLua";
pub const LUA_MAGIC_SIZE: usize = 4;

struct Header {
    magic:         u32,
    /// High hex digit is major version number
    /// Low hex digit is minor version number
    ver:           u8,
    format:        u8,
    endianness:    u8,
    int_sz:        u8,
    size_t_sz:     u8,
    intr_sz:       u8,
    lua_number_sz: u8,
    integral_flag: u8,
    // conv_data:     [u8; 6],
}

struct Function_header {
    line_start: u32,
    line_end:   u32,
    n_params:   u8,
    varag_flag: u8,
    n_regs:     u8,
    n_instr:    u32,
}

struct Constants {
    n_consts: u32,
    consts:   Vec<Constant>,
}

struct Constant {
    const_type: u8,
    const_len:  u32,
    const_data: Vec<u8>,
}

#[derive(Debug)]
/// Each version of lua has a different header !???
pub struct Lua {
    opt: Opt,
}

impl super::FileFormat for Lua {
    type Item = Self;

    fn parse(opt: Opt, _buf: &[u8]) -> Result<Self, Error> {

        Ok(Lua {
            opt,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("Lua Bytecode");
        println!("{:#X?}", self);

        Ok(())
    }


}
