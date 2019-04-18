use crate::Opt;
use failure::{Error};

pub const LUA_MAGIC: &'static [u8; LUA_MAGIC_SIZE] = b"\x1BLua";
pub const LUA_MAGIC_SIZE: usize = 4;

#[derive(Debug)]
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
