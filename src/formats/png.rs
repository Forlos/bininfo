use scroll::{self, Pread};

use failure::{
    Error,
};

use crate::format::{fmt_indentln};

pub const PNG_HEADER: &'static [u8; PNG_HEADER_SIZE] = b"\x89PNG\x0D\x0A\x1A\x0A";
pub const PNG_HEADER_SIZE: usize = 8;

// const IHDR_SIZE: usize = PNG_HEADER_SIZE + 25;


#[derive(Debug, Pread)]
#[repr(C)]
struct Ihdr {
    size:        u32,
    id:          u32,
    width:       i32,
    height:      i32,
    bpp:         u8,
    color:       u8,
    compression: u8,
    filter:      u8,
    interlace:   u8,
    checksum:    u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Plte {
    size:     u32,
    id:       u32,
    red:      u8,
    green:    u8,
    blue:     u8,
    checksum: u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
// https://tools.ietf.org/html/rfc1950#page-5
struct Zlib {
    // bits 0 to 3  CM     Compression method
    // bits 4 to 7  CINFO  Compression info
    cmf: u8,

    // bits 0 to 4  FCHECK  (check bits for CMF and FLG)
    // bit  5       FDICT   (preset dictionary)
    // bits 6 to 7  FLEVEL  (compression level)
    flg: u8,
    deflate: Deflate,
    adler: u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
// https://tools.ietf.org/html/rfc1951#page-11
struct Deflate {
    // first bit       BFINAL
    // next 2 bits     BTYPE
    flag: u8,
    // 3.2.4. Non-compressed blocks (BTYPE=00)

    //     Any bits of input up to the next byte boundary are ignored.
    //     The rest of the block consists of the following information:

    //     0   1   2   3   4...
    //     +---+---+---+---+================================+
    //     |  LEN  | NLEN  |... LEN bytes of literal data...|
    //     +---+---+---+---+================================+

    //     LEN is the number of data bytes in the block.  NLEN is the
    //     one's complement of LEN.
    len: u16,
    nlen: u16,
}

#[derive(Debug)]
#[repr(C)]
struct Idat {
    size:     u32,
    checksum: u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Iend {
    size:     u32,
    id:       u32,
    checksum: u32,
}

#[derive(Debug)]
pub struct Png {
    ihdr: Ihdr,
    plte: Option<Plte>,
    idat: Vec<Idat>,
    zlib: Zlib,
    iend: Option<Iend>,
}

impl Png {

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        const CHUNK_SIZE: usize = 12;

        let mut plte = None;
        let mut idat = Vec::new();
        let mut data = Vec::new();
        let mut iend = None;

        let ihdr: Ihdr = buf.pread_with(PNG_HEADER_SIZE, scroll::BE)?;

        let mut index = PNG_HEADER_SIZE;

        while index < buf.len() {

            let size = buf.pread_with::<u32>(index, scroll::BE)? as usize;
            let id   = &buf[index + 4..index + 8];
            let s    = std::str::from_utf8(id)?;

            match s {
                "IHDR" => {
                    ()
                },
                "PLTE" => {
                    plte = Some(buf.pread_with(index, scroll::BE)?);
                },
                "IDAT" => {
                    let mut inner_data = buf[index + 8..index + 8 + size].to_vec();
                    data.append(&mut inner_data);

                    let dat = Idat {
                        size: size as u32,
                        checksum: buf.pread_with(index + size + 8, scroll::BE)?,
                    };

                    idat.push(dat);

                },
                "IEND" => {
                    iend = Some(buf.pread_with(index, scroll::BE)?);
                },
                _ => (),
            }

            index += size + CHUNK_SIZE;

        }

        let flag = data.pread_with::<u8>(2, scroll::BE)?;
        let btype = (flag << 5) >> 6;
        let zlib;

        if btype == 0 {

            zlib = Zlib {
                cmf: data.pread_with(0, scroll::BE)?,
                flg: data.pread_with(1, scroll::BE)?,
                deflate: data.pread_with(2, scroll::BE)?,
                adler: data.pread_with(data.len() - 4, scroll::BE)?,
            };

        }
        else {

            zlib = Zlib {
                cmf: data.pread_with(0, scroll::BE)?,
                flg: data.pread_with(1, scroll::BE)?,
                deflate: Deflate {
                    flag,
                    len: 0,
                    nlen: 0,
                },
                adler: data.pread_with(data.len() - 4, scroll::BE)?,
            };

        }

        Ok(Png {
            ihdr,
            plte,
            idat,
            zlib,
            iend,
        })

    }

    pub fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;
        use prettytable::Table;

        //
        // PNG file
        //
        println!("PNG width: {}px height: {}px",
                 self.ihdr.width,
                 self.ihdr.height);
        println!();

        //
        // IHDR
        //
        println!("{}:", Color::White.underline().paint("IHDR"));
        fmt_indentln(format!("Image width: {} height: {}", self.ihdr.width, self.ihdr.height));
        fmt_indentln(format!("Bits per pixel: {}", self.ihdr.bpp));
        fmt_indentln(format!("Color type: {}", self.ihdr.color));
        fmt_indentln(format!("Compression method: {}", self.ihdr.compression));
        fmt_indentln(format!("Filter method: {}", self.ihdr.filter));
        fmt_indentln(format!("Interlace method: {}", self.ihdr.interlace));
        fmt_indentln(format!("Checksum: {:#010X}", self.ihdr.checksum));
        println!();

        //
        // PLTE
        //
        if let Some(plte) = &self.plte {
            println!("{}:", Color::White.underline().paint("PLTE"));
            fmt_indentln(format!("Red:   {}",
                                 Color::Red.paint(format!("{:#04X}",plte.red))));
            fmt_indentln(format!("Green: {}",
                                 Color::Green.paint(format!("{:#04X}", plte.green))));
            fmt_indentln(format!("Blue:  {}",
                                 Color::Blue.paint(format!("{:#04X}", plte.blue))));
            fmt_indentln(format!("Checksum: {:#010X}", plte.checksum));
            println!();
        }

        //
        // IDAT
        //
        println!("{}({}) {} {} {} {} {}",
            Color::White.underline().paint("IDAT"),
            self.idat.len(),
            Color::Purple.paint(format!("Compression method: {}",self.zlib.cmf << 4 >> 4)),
            Color::Cyan.paint(format!("Compression info: {}",self.zlib.cmf >> 4)),
            Color::Green.paint(format!("Checksum: {:#07b}", self.zlib.flg << 3 >> 3)),
            Color::Red.paint(format!("Dict: {:#03b}", self.zlib.flg << 2 >> 7)),
            Color::Yellow.paint(format!("Compression level: {:#04b}", self.zlib.flg >> 6)),);

        let mut idx = 0;
        let mut table = Table::new();
        let format = prettytable::format::FormatBuilder::new()
            .column_separator(' ')
            .borders(' ')
            .padding(1, 1)
            .build();
        table.set_format(format);
        table.add_row(row![l->"Idx", l->"Size", l->"Checksum"]);

        for idat in &self.idat {
            table.add_row(row![r->idx,
                               rFr->idat.size,
                               rFb->format!("{:#010X}", idat.checksum)]);
            idx += 1;
        }
        table.printstd();
        println!();

        //
        // IEND
        //
        if let Some(iend) = &self.iend {
            println!("{}:", Color::White.underline().paint("IEND"));
            fmt_indentln(format!("Checksum: {:#010X}", iend.checksum));
        }

        Ok(())

    }

}
