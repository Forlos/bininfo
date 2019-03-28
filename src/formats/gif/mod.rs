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
    // <Packed Fields>  =
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

// Local Color Table.
#[derive(Debug)]
#[repr(C)]
struct LCT {
    table: Vec<RGB>,
}

const EXTENSION_INTRODUCER:  u8 = 0x21;

const GRAPHIC_CONTROL_LABEL: u8 = 0xF9;
const COMMENT_LABEL: u8         = 0xFE;
const PLAIN_TEXT_LABEL: u8      = 0x01;
const APPLICATION_LABEL: u8 = 0xFF;

// Graphic Control Extension.
#[derive(Debug, Pread)]
#[repr(C)]
struct GC_Ext {
    ext_intro:        u8,
    ctrl_label:       u8,
    block_size:       u8,
    // <Packed Fields>  =
    //   Reserved                      3 Bits
    //   Disposal Method               3 Bits
    //   User Input Flag               1 Bit
    //   Transparent Color Flag        1 Bit
    packet_fields:    u8,
    delay:            u16,
    transp_color_idx: u8,
    block_terminator: u8,

}

#[derive(Debug)]
#[repr(C)]
struct Comment_Ext {
    ext_intro:        u8,
    ctrl_label:       u8,
    comment_data:     Vec<u8>,
    block_terminator: u8,
}

#[derive(Debug)]
#[repr(C)]
struct PT_Ext {
    ext_intro:        u8,
    ctrl_label:       u8,
    block_size:       u8,
    tg_left_pos:      u16,
    tg_top_pos:       u16,
    tg_width:         u16,
    th_height:        u16,
    char_cell_width:  u8,
    char_cell_height: u8,
    tf_color_idx:     u8,
    tb_color_idx:     u8,
    plain_text:       Vec<u8>,
    block_terminator: u8,
}

#[derive(Debug)]
#[repr(C)]
struct App_Ext {
    ext_intro:        u8,
    ctrl_label:       u8,
    block_size: u8,
    app_identifier: [u8; 8],
    app_auth_code: [u8; 3],
    app_data: Vec<u8>,
    block_terminator: u8,
}

const IMAGE_DESCRIPTOR_SEPARATOR: u8 = 0x2c;

#[derive(Debug, Pread)]
#[repr(C)]
struct Img_desc {
    separator:     u8,
    left_pos:      u16,
    top_pos:       u16,
    width:         u16,
    height:        u16,
    // <Packed Fields>  =
    //   Local Color Table Flag        1 Bit
    //   Interlace Flag                1 Bit
    //   Sort Flag                     1 Bit
    //   Reserved                      2 Bits
    //   Size of Local Color Table     3 Bits
    packed_fields: u8,
}

pub struct Gif {
    header: Gif_header,
    lsd:    LSD,
    gct:    Option<GCT>,
    lct:    Option<LCT>,
    // img_desc: Img_desc,

    // trailer: u8,
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
        let mut lct = None;

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
            lct,
        })

    }

    pub fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;
        use prettytable::Table;

        //
        // GIF file
        //
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

        //
        // Logical Screen Descriptor
        //
        println!("{}", Color::White.underline().paint("Logical Screen Descriptor"));
        fmt_indentln(format!("Logical Screen width:  {}px", self.lsd.logic_width));
        fmt_indentln(format!("Logical Screen height: {}px", self.lsd.logic_height));
        fmt_indentln(format!("Global Color Table Flag:    {}", gctf));
        fmt_indentln(format!("Color Resolution:           {}", color_res));
        fmt_indentln(format!("Sort Flag:                  {}", sort_flag));
        fmt_indentln(format!("Size of Global Color Table: {}", sz_gct));
        println!();

        //
        // Global Color Table
        //
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
