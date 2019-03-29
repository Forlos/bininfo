use failure::{Error};
use scroll::{self, Pread};

use crate::Problem;
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

const IMAGE_DESCRIPTOR_SEPARATOR: u8 = 0x2c;
const EXTENSION_INTRODUCER: u8       = 0x21;

const GRAPHIC_CONTROL_LABEL: u8 = 0xF9;
const COMMENT_LABEL: u8         = 0xFE;
const PLAIN_TEXT_LABEL: u8      = 0x01;
const APPLICATION_LABEL: u8     = 0xFF;

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
    comment_data:     Vec<String>,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct PT_Ext_header {
    block_size:       u8,
    tg_left_pos:      u16,
    tg_top_pos:       u16,
    tg_width:         u16,
    tg_height:        u16,
    char_cell_width:  u8,
    char_cell_height: u8,
    tf_color_idx:     u8,
    tb_color_idx:     u8,
}

#[derive(Debug)]
#[repr(C)]
struct PT_Ext {
    header:     PT_Ext_header,
    plain_text: Vec<String>,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct App_Ext {
    block_size:       u8,
    app_identifier:   [u8; 8],
    app_auth_code:    [u8; 3],
}

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
    header:   Gif_header,
    lsd:      LSD,
    gct:      Option<GCT>,
    img_desc: Img_desc,
    lct:      Option<LCT>,

    gc_ext:      Option<GC_Ext>,
    comment_ext: Option<Comment_Ext>,
    pt_ext:      Option<PT_Ext>,
    app_ext:     Option<App_Ext>,

    // trailer: u8,
}

impl Gif {

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        // Error there should't ever happen because we have read 16 bytes when parsing magic.
        let header = buf.pread_with(0, scroll::BE)?;
        // Error there should't ever happen because we have read 16 bytes when parsing magic.
        let lsd    = buf.pread_with::<LSD>(GIF_MAGIC_SIZE, scroll::LE)
            .map_err(|e| Problem::Msg(format!("Cannot read logical screen descriptor: {}", e)))?;

        // Global Color Table Flag
        let gctf      = lsd.packed_fields >> 7;
        // let color_res = lsd.packed_fields << 1 >> 5;
        // let sort_flag = lsd.packed_fields << 4 >> 7;
        // Size of Global Color Table
        let sz_gct    = lsd.packed_fields << 5 >> 5;

        let mut gct = None;
        let mut lct = None;

        let mut gc_ext      = None;
        let mut comment_ext = None;
        let mut pt_ext      = None;
        let mut app_ext     = None;

        let mut index = GIF_MAGIC_SIZE + LSD_SIZE;

        if gctf == 1 {
            let size = 3 * 2_usize.pow(sz_gct as u32 + 1) / 3;
            let mut table = Vec::with_capacity(size);
            for i in 0..size {
                table.push(buf.pread(index + i * 3)?)
            }
            gct = Some(GCT {
                table,
            });
            index += size * 3;
        }

        let mut identifier: u8 = buf.pread(index)
            .map_err(|e| Problem::Msg(format!("Cannot read identifier: {}", e)))?;

        while identifier != IMAGE_DESCRIPTOR_SEPARATOR {

            match identifier {

                EXTENSION_INTRODUCER => {

                    match buf.pread::<u8>(index + 1)? {

                        GRAPHIC_CONTROL_LABEL => {
                            index += 8;
                        },
                        COMMENT_LABEL => {
                            index += 2;
                            let mut comment_data = Vec::new();
                            while buf.pread::<u8>(index)? != 0x00 {
                                let size = buf.pread::<u8>(index + 1)? as usize;
                                let data = std::str::from_utf8(&buf[index + 2..index + 2 + size])?.to_string();
                                comment_data.push(data);
                                index += size + 1;
                            }
                            index += 1;
                            // If this chunk is malformed this prevent program from crashing
                            while buf.pread::<u8>(index)? == 0x00 {
                                index += 1;
                            }

                            comment_ext = Some(Comment_Ext {
                                comment_data,
                            });
                        },
                        PLAIN_TEXT_LABEL => {
                            index += 2;
                            let header = buf.pread_with(index, scroll::LE)?;
                            index += 13;
                            let mut plain_text = Vec::new();
                            while buf.pread::<u8>(index)? != 0x00 {
                                let size = buf.pread::<u8>(index + 1)? as usize;
                                let data = std::str::from_utf8(&buf[index + 2..index + 2 + size])?.to_string();
                                plain_text.push(data);
                                index += size + 1;
                            }
                            index += 1;
                            // If this chunk is malformed this prevent program from crashing
                            while buf.pread::<u8>(index)? == 0x00 {
                                index += 1;
                            }
                            pt_ext = Some(PT_Ext {
                                header,
                                plain_text,
                            });
                        },
                        APPLICATION_LABEL => {
                            index += 2;
                            app_ext = Some(buf.pread_with(index, scroll::LE)?);
                            index += 12;
                            while buf.pread::<u8>(index)? != 0x00 {
                                index += buf.pread::<u8>(index)? as usize + 1;
                            }
                            index += 1;
                            // If this chunk is malformed this prevent program from crashing
                            while buf.pread::<u8>(index)? == 0x00 {
                                index += 1;
                            }
                        },
                        _ => {
                            return Err(Error::from(Problem::Msg(format!("Unsupported extension at: {:#X}", index + 1))));
                        }

                    }

                },
                _ => {
                    return Err(Error::from(Problem::Msg(format!("Unsupported extension at: {:#X}", index))));
                }

            }

            identifier = buf.pread(index)
                .map_err(|e| Problem::Msg(format!("Cannot read identifier: {}", e)))?;

        }

        let img_desc = buf.pread_with::<Img_desc>(index, scroll::LE)
            .map_err(|e| Problem::Msg(format!("Cannot read image descriptor: {}", e)))?;
        index += 10;

        if (img_desc.packed_fields >> 7) == 1 {
            let sz_lct = img_desc.packed_fields << 5 >> 5;
            let size = 3 * 2_usize.pow(sz_lct as u32 + 1) / 3;
            let mut table = Vec::with_capacity(size);
            for i in 0..size {
                table.push(buf.pread(index + i * 3)?);
            }
            lct = Some(LCT {
                table,
            });
            // index += size * 3;
        }

        Ok(Gif {
            header,
            lsd,
            gct,
            lct,
            img_desc,

            gc_ext,
            comment_ext,
            pt_ext,
            app_ext,
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
        fmt_indentln(format!("Logical screen width:  {}px", self.lsd.logic_width));
        fmt_indentln(format!("Logical screen height: {}px", self.lsd.logic_height));
        fmt_indentln(format!("Global color table flag:    {}", gctf));
        fmt_indentln(format!("Color resolution:           {}", color_res));
        fmt_indentln(format!("Sort flag:                  {}", sort_flag));
        fmt_indentln(format!("Size of global color table: {}", sz_gct));
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

        //
        // Graphic Control Extension
        //
        if let Some(gc_ext) = &self.gc_ext {
            println!("{}", Color::White.underline().paint("Graphic Control Extension"));
            fmt_indentln(format!("Block size: {}", gc_ext.block_size));
            fmt_indentln(format!("Reserved: {}", gc_ext.packet_fields >> 5));
            fmt_indentln(format!("Disposal method: {}", gc_ext.packet_fields << 3 >> 5));
            fmt_indentln(format!("User input flag: {}", gc_ext.packet_fields & 0b00000010));
            fmt_indentln(format!("Transparent color flag: {}", gc_ext.packet_fields & 0b00000001));
            fmt_indentln(format!("Transparent color idx: {}", gc_ext.transp_color_idx));
            println!();
        }

        //
        // Comment Extension
        //
        if let Some(comment_ext) = &self.comment_ext {
            println!("{}", Color::White.underline().paint("Comment Extension"));
            for com in comment_ext.comment_data.iter() {
                println!("{}", com);
            }
            println!();
        }

        //
        // Plain Text Extension
        //
        if let Some(pt_ext) = &self.pt_ext {
            println!("{}", Color::White.underline().paint("Plain Text Extension"));
            fmt_indentln(format!("Block size: {}", pt_ext.header.block_size));
            fmt_indentln(format!("Text grid left position: {}px", pt_ext.header.tg_left_pos));
            fmt_indentln(format!("Text grid top position:  {}px", pt_ext.header.tg_top_pos));
            fmt_indentln(format!("Text grid width:  {}px", pt_ext.header.tg_width));
            fmt_indentln(format!("Text grid height: {}px", pt_ext.header.tg_height));
            fmt_indentln(format!("Character cell width:  {}px", pt_ext.header.char_cell_width));
            fmt_indentln(format!("Character cell height: {}px", pt_ext.header.char_cell_height));
            fmt_indentln(format!("Text foreground color idx: {}", pt_ext.header.tf_color_idx));
            fmt_indentln(format!("Text background color idx: {}", pt_ext.header.tb_color_idx));
            for text in pt_ext.plain_text.iter() {
                fmt_indentln(format!("{}", text));
            }
            println!();
        }

        //
        // Application Extension
        //
        if let Some(app_ext) = &self.app_ext {
            println!("{}", Color::White.underline().paint("Application Extension"));
            fmt_indentln(format!("Block size: {}", app_ext.block_size));
            fmt_indentln(format!("Application identifier: {}",
                                 Color::Blue.paint(std::str::from_utf8(&app_ext.app_identifier)?)));
            fmt_indentln(format!("Application auth code:  {}",
                                 Color::Blue.paint(std::str::from_utf8(&app_ext.app_auth_code)?)));
            println!();
        }

        //
        // Image Descriptor
        //
        println!("{}", Color::White.underline().paint("Image Descriptor"));
        fmt_indentln(format!("Image left position: {}px", self.img_desc.left_pos));
        fmt_indentln(format!("Image top position:  {}px", self.img_desc.top_pos));
        fmt_indentln(format!("Image width:  {}px", self.img_desc.width));
        fmt_indentln(format!("Image height: {}px", self.img_desc.height));
        fmt_indentln(format!("Local color table flag:    {}", self.img_desc.packed_fields >> 7));
        fmt_indentln(format!("Interlace flag:            {}", self.img_desc.packed_fields << 1 >> 7));
        fmt_indentln(format!("Sort flag:                 {}", self.img_desc.packed_fields << 2 >> 7));
        fmt_indentln(format!("Reserved:                  {}", self.img_desc.packed_fields << 3 >> 6));
        fmt_indentln(format!("Size of local color table: {}", self.img_desc.packed_fields & 0b00000111));
        println!();

        //
        // Local Color Table
        //
        if let Some(lct) = &self.lct {
            println!("{}", Color::White.underline().paint("Local Color Table"));
            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![r->"Idx", rFr->"Red", rFg->"Green", rFb->"Blue"]);
            for (i, rgb) in lct.table.iter().enumerate() {
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
