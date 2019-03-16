use scroll::{self, Pread};

use failure::{
    Error,
};

use crate::format::{fmt_indentln, fmt_png_header};

pub const PNG_HEADER: &'static [u8; PNG_HEADER_SIZE] = b"\x89PNG\x0D\x0A\x1A\x0A";
pub const PNG_HEADER_SIZE: usize = 8;

// const IHDR_SIZE: usize = PNG_HEADER_SIZE + 25;

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Prefix {
    pub size: u32,
    id:   u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Postfix {
    pub checksum: u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Ihdr {
    prefix:      Prefix,
    width:       i32,
    height:      i32,
    bpp:         u8,
    color:       u8,
    compression: u8,
    filter:      u8,
    interlace:   u8,
    postfix:     Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Plte {
    prefix:  Prefix,
    red:     u8,
    green:   u8,
    blue:    u8,
    postfix: Postfix,
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
    flg:     u8,
    deflate: Deflate,
    adler:   u32,
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
    prefix: Prefix,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Iend {
    prefix:  Prefix,
    postfix: Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Bkgd {
    prefix:  Prefix,
    color:   bKGD,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Bkgd3 {
    index: u8,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Bkgd04 {
    gray: u16,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Bkgd26 {
    red:   u16,
    green: u16,
    blue : u16,
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
enum bKGD {
    bKGD3(Bkgd3),
    bKGD04(Bkgd04),
    bKGD26(Bkgd26),
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Chrm {
    prefix:  Prefix,
    white_x: u32,
    white_y: u32,
    red_x:   u32,
    red_y:   u32,
    green_x: u32,
    green_y: u32,
    blue_x:  u32,
    blue_y:  u32,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Gama {
    prefix:  Prefix,
    gamma:   u32,
    postfix: Postfix,
}

#[derive(Debug)]
pub struct Png {
    //
    // Critical chunks
    //
    ihdr: Ihdr,
    plte: Option<Plte>,
    idat: Vec<Idat>,
    zlib: Zlib,
    iend: Option<Iend>,
    //
    // Ancillary chunks
    //
    bkgd: Option<Bkgd>,
    chrm: Option<Chrm>,
    gama: Option<Gama>,
}

impl Png {

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        const CHUNK_SIZE: usize = 12;

        //
        // Critical chunks
        //
        let mut plte = None;
        let mut idat = Vec::new();
        let mut data = Vec::new();
        let mut iend = None;

        //
        // Ancillary chunks
        //
        let mut bkgd = None;
        let mut chrm = None;
        let mut gama = None;

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
                        prefix: buf.pread_with(index, scroll::BE)?,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,

                    };

                    idat.push(dat);

                },
                "IEND" => {
                    iend = Some(buf.pread_with(index, scroll::BE)?);
                },
                "bKGD" => {
                    let color_type = ihdr.color;
                    let color;
                    if color_type == 3 {
                        color = bKGD::bKGD3(buf.pread_with(index + 8, scroll::BE)?);
                    }
                    else if color_type == 0 || color_type == 4 {
                        color = bKGD::bKGD04(buf.pread_with(index + 8, scroll::BE)?);
                    }
                    else if color_type == 2 || color_type == 6 {
                        color = bKGD::bKGD26(buf.pread_with(index + 8, scroll::BE)?);
                    }
                    else {
                        panic!("Invalid color type for bKGD");
                    }
                    bkgd = Some(Bkgd {
                        prefix: buf.pread_with(index, scroll::BE)?,
                        color,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    });
                },
                "cHRM" => {
                    chrm = Some(buf.pread_with(index, scroll::BE)?);
                },
                "gAMA" => {
                    gama = Some(buf.pread_with(index, scroll::BE)?);
                },
                _ => (),
            }

            println!("{}", s);

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

            bkgd,
            chrm,
            gama,
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
        fmt_png_header("IHDR", &self.ihdr.prefix, &self.ihdr.postfix);
        fmt_indentln(format!("Image width: {} height: {}", self.ihdr.width, self.ihdr.height));
        fmt_indentln(format!("Bits per pixel: {}", self.ihdr.bpp));
        fmt_indentln(format!("Color type: {}", self.ihdr.color));
        fmt_indentln(format!("Compression method: {}", self.ihdr.compression));
        fmt_indentln(format!("Filter method: {}", self.ihdr.filter));
        fmt_indentln(format!("Interlace method: {}", self.ihdr.interlace));
        fmt_indentln(format!("Checksum: {:#010X}", self.ihdr.postfix.checksum));
        println!();

        //
        // PLTE
        //
        if let Some(plte) = &self.plte {
            fmt_png_header("PLTE", &plte.prefix, &plte.postfix);
            fmt_indentln(format!("Red:   {}",
                                 Color::Red.paint(format!("{:#04X}",plte.red))));
            fmt_indentln(format!("Green: {}",
                                 Color::Green.paint(format!("{:#04X}", plte.green))));
            fmt_indentln(format!("Blue:  {}",
                                 Color::Blue.paint(format!("{:#04X}", plte.blue))));
            fmt_indentln(format!("Checksum: {:#010X}", plte.postfix.checksum));
            println!();
        }

        //
        // bKGD
        //
        if let Some(background) = &self.bkgd {
            fmt_png_header("bKGD", &background.prefix, &background.postfix);
            match &background.color {
                bKGD::bKGD3(bkgd)  => {
                    fmt_indentln(format!("Palette index: {}", bkgd.index));
                },
                bKGD::bKGD04(bkgd) => {
                    fmt_indentln(format!("Gray: {:#06X}", bkgd.gray));
                },
                bKGD::bKGD26(bkgd) => {
                    fmt_indentln(format!("Red:   {}",
                                         Color::Red.paint(format!("{:#06X}",bkgd.red))));
                    fmt_indentln(format!("Green: {}",
                                         Color::Green.paint(format!("{:#06X}",bkgd.green))));
                    fmt_indentln(format!("Blue:  {}",
                                         Color::Blue.paint(format!("{:#06X}",bkgd.blue))));
                },
            }
            println!();
        }

        //
        // IDAT
        //
        println!("{}({}) {} {} {} {} {} {}",
            Color::White.underline().paint("IDAT"),
            self.idat.len(),
            Color::Purple.paint(format!("Compression method: {}",self.zlib.cmf << 4 >> 4)),
            Color::Cyan.paint(format!("Compression info: {}",self.zlib.cmf >> 4)),
            Color::Green.paint(format!("Checksum: {:#07b}", self.zlib.flg << 3 >> 3)),
            Color::Red.paint(format!("Dict: {:#03b}", self.zlib.flg << 2 >> 7)),
            Color::Yellow.paint(format!("Compression level: {:#04b}", self.zlib.flg >> 6)),
            Color::Fixed(221).paint(format!("Adler: {:#010X}", self.zlib.adler)));

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
                               rFy->idat.prefix.size,
                               rFg->format!("{:#010X}", idat.postfix.checksum)]);
            idx += 1;
        }
        table.printstd();
        println!();

        //
        // cHRM
        //
        if let Some(chrm) = &self.chrm {
            fmt_png_header("cHRM", &chrm.prefix, &chrm.postfix);

            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![r->"Color", r->"X", r->"Y", r->"Point x", r->"Point y"]);
            table.add_row(row![Fwr=>"White",
                               chrm.white_x, chrm.white_y,
                               format!("{:.5}",chrm.white_x as f32 / 100000.0),
                               format!("{:.5}",chrm.white_y as f32 / 100000.0)]);
            table.add_row(row![Frr=>"Red",
                               chrm.red_x, chrm.red_y,
                               format!("{:.5}",chrm.red_x as f32 / 100000.0),
                               format!("{:.5}",chrm.red_y as f32 / 100000.0)]);
            table.add_row(row![Fgl=>"Green",
                               chrm.green_x, chrm.green_y,
                               format!("{:.5}",chrm.green_x as f32 / 100000.0),
                               format!("{:.5}",chrm.green_y as f32 / 100000.0)]);
            table.add_row(row![Fbr=>"Blue",
                               chrm.blue_x, chrm.blue_y,
                               format!("{:.5}",chrm.blue_x as f32 / 100000.0),
                               format!("{:.5}",chrm.blue_y as f32 / 100000.0)]);
            table.printstd();
            println!();
        }

        //
        // gAMA
        //
        if let Some(gama) = &self.gama {
            fmt_png_header("gAMA", &gama.prefix, &gama.postfix);
            fmt_indentln(format!("Gamma: {} ({})",
                                 gama.gamma,
                                 format!("{:.5}", gama.gamma as f32 / 100000.0)));
            println!();
        }

        //
        // IEND
        //
        if let Some(iend) = &self.iend {
            fmt_png_header("IEND", &iend.prefix, &iend.postfix);
        }

        Ok(())

    }

}
