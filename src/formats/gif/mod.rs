use failure::{Error};
use scroll::{self, Pread};

use crate::format::{fmt_indentln};

pub const GIF87A_MAGIC: &'static [u8; GIF_MAGIC_SIZE] = b"GIF87a";
pub const GIF89A_MAGIC: &'static [u8; GIF_MAGIC_SIZE] = b"GIF89a";
pub const GIF_MAGIC_SIZE: usize = 6;
pub const LSD_SIZE: usize = 7;

// TODO allow to use this as a argument and disable trimming in general.
const TRIM_INDEX: usize = 20;

#[derive(Debug, Pread)]
#[repr(C)]
struct RGB {
    red:     u8,
    green:   u8,
    blue:    u8,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Gif_header {
    magic:   [u8; 3],
    version: [u8; 3],
}

// Logical Screen Descriptor.
#[derive(Debug, Pread)]
#[repr(C)]
struct LSD {
    logic_width:        u16,
    logic_height:       u16,
    //   <Packed Fields>  =
    //   Global Color Table Flag       1 Bit
    //   Color Resolution              3 Bits
    //   Sort Flag                     1 Bit
    //   Size of Global Color Table    3 Bits
    packed_fields:      u8,
    bkg_color_idx:      u8,
    pixel_aspect_ratio: u8,
}

// Global Color Table.
#[derive(Debug)]
#[repr(C)]
struct GCT {
    table: Vec<RGB>,
}

pub struct Gif {
    header: Gif_header,
    lsd:    LSD,
    gct:    Option<GCT>,
}

impl Gif {

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        let header = buf.pread_with(0, scroll::BE)?;
        let lsd    = buf.pread_with::<LSD>(GIF_MAGIC_SIZE, scroll::LE)?;

        // Global Color Table Flag
        let gctf      = lsd.packed_fields >> 7;
        let color_res = lsd.packed_fields << 1 >> 5;
        let sort_flag = lsd.packed_fields << 4 >> 7;
        // Size of Global Color Table
        let sz_gct    = lsd.packed_fields << 5 >> 5;

        let mut gct = None;

        if gctf == 1 {
            let size = 3 * 2_usize.pow(sz_gct as u32 + 1) / 3;
            let mut table = Vec::with_capacity(size);
            for i in 0..size {
                table.push(buf.pread(GIF_MAGIC_SIZE + LSD_SIZE + i * 3)?)
            }
            gct = Some(GCT {
                table,
            });
        }

        Ok(Gif {
            header,
            lsd,
            gct,
        })

    }

    pub fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;
        use prettytable::Table;

        println!("GIF{} width: {}px height: {}px",
                 std::str::from_utf8(&self.header.version)?.to_string(),
                 self.lsd.logic_width,
                 self.lsd.logic_height);
        println!();

        // Global Color Table Flag
        let gctf      = &self.lsd.packed_fields >> 7;
        let color_res = &self.lsd.packed_fields << 1 >> 5;
        let sort_flag = &self.lsd.packed_fields << 4 >> 7;
        // Size of Global Color Table
        let sz_gct    = &self.lsd.packed_fields << 5 >> 5;

        println!("{}", Color::White.underline().paint("Logical Screen Descriptor"));
        fmt_indentln(format!("Logical Screen width:  {}px", self.lsd.logic_width));
        fmt_indentln(format!("Logical Screen height: {}px", self.lsd.logic_height));
        fmt_indentln(format!("Global Color Table Flag:    {}", gctf));
        fmt_indentln(format!("Color Resolution:           {}", color_res));
        fmt_indentln(format!("Sort Flag:                  {}", sort_flag));
        fmt_indentln(format!("Size of Global Color Table: {}", sz_gct));
        println!();

        if let Some(gct) = &self.gct {
            println!("{}", Color::White.underline().paint("Global Color Table"));
            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![r->"Idx", rFr->"Red", rFg->"Green", rFb->"Blue"]);
            for (i, rgb) in gct.table.iter().enumerate() {
                table.add_row(row![r->i,
                                   rFr->format!("{:#04X}", rgb.red),
                                   rFg->format!("{:#04X}", rgb.green),
                                   rFb->format!("{:#04X}", rgb.blue)]);
                if i == TRIM_INDEX {
                    trimmed = true;
                    break;
                }
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
