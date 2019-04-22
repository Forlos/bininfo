#![allow(non_camel_case_types)]
use crate::Opt;
use crate::Problem;
use failure::{Error};
use scroll::{self, Pread};

pub mod lua51;

pub const LUA_MAGIC: &'static [u8; LUA_MAGIC_SIZE] = b"\x1BLua";
pub const LUA_MAGIC_SIZE: usize = 4;

#[derive(Debug, Pread)]
struct Lua_header {
    magic:         u32,
    // High hex digit is major version number
    /// Low hex digit is minor version number
    ver:           u8,
}

#[derive(Debug)]
enum Info {
    Lua51(lua51::Lua51_info),
}

#[derive(Debug)]
/// Each version of lua has a different header !???
pub struct Lua {
    opt: Opt,

    lua_header: Lua_header,
    info:       Info,
}

impl super::FileFormat for Lua {
    type Item = Self;

    fn parse(opt: Opt, buf: &[u8]) -> Result<Self, Error> {

        let offset = &mut 0;

        let lua_header = buf.gread::<Lua_header>(offset)?;
        let info = match lua_header.ver {
            0x51 => Info::Lua51(lua51::parse(buf, offset)?),
            _ => return Err(Error::from(Problem::Msg(format!("Unsupported lua version")))),
        };

        Ok(Lua {
            opt,

            lua_header,
            info,
        })

    }

    fn print(&self) -> Result<(), Error> {

        println!("{:#X?}", self);
        println!("Lua Bytecode {}.{}",
                 self.lua_header.ver >> 4,
                 self.lua_header.ver << 4 >> 4);
        println!();

        match &self.info {
            Info::Lua51(info) => info.print(),
        }

        Ok(())
    }


}
