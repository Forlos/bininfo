#![allow(non_camel_case_types, dead_code)]

use failure::{
    Error,
};

use scroll::{self, Pread};

use crate::format::{
    fmt_indent, fmt_indentln,
};

// BMP in windows
pub const BMP_MAGIC: &'static [u8; BMP_MAGIC_SIZE] = b"BM";
pub const BMP_MAGIC_SIZE: usize = 2;

// Other BMPs
// OS/2 struct bitmap array
pub const OS2_BMP_MAGIC_BITMAP_ARRAY:  &'static [u8; BMP_MAGIC_SIZE] = b"BA";
// OS/2 struct color icon
pub const OS2_BMP_MAGIC_COLOR_ICON:    &'static [u8; BMP_MAGIC_SIZE] = b"CI";
// OS/2 const color pointer
pub const OS2_BMP_MAGIC_COLOR_POINTER: &'static [u8; BMP_MAGIC_SIZE] = b"CP";
// OS/2 struct icon
pub const OS2_BMP_MAGIC_ICON:          &'static [u8; BMP_MAGIC_SIZE] = b"IC";
// OS/2 pointer
pub const OS2_BMP_MAGIC_POINTER:       &'static [u8; BMP_MAGIC_SIZE] = b"PT";

const BMP_HEADER_SIZE: usize = 14;

const INFO_V1: usize = 40;
const INFO_V2: usize = 52;
const INFO_V3: usize = 56;
const INFO_V4: usize = 108;
const INFO_V5: usize = 124;

#[derive(Debug, Pread)]
#[repr(C)]
struct Bmp_header {
    magic:     u16,
    file_size: u32,
    reserved1: u16,
    reserved2: u16,
    offset:    u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Bitmap_info_header {
    size:         u32,
    width:        i32,
    height:       i32,
    planes:       u16,
    bpp:          u16,
    compression:  u32,
    image_size:   u32,
    h_resolution: i32,
    v_resolution: i32,
    palette:      u32,
    imp_colors:   u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct RGB_bitmask {
    red_bitmask:   u32,
    green_bitmask: u32,
    blue_bitmask:  u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Alpha_bitmask {
    alpha_bitmask: u32
}

#[derive(Pread)]
#[repr(C)]
struct Color_space_gamma {
    color_space:   [u8; 36],
    gamma_red:     u32,
    gamma_green:   u32,
    gamma_blue:    u32,
}

#[derive(Pread)]
#[repr(C)]
struct ICC_color_prof {
    intent:        u32,
    icc_data:      u32,
    icc_size:      u32,
    reserved:      u32,
}

#[derive(Debug)]
#[repr(u32)]
enum Compression_method {
    BI_RGB            = 0,
    BI_RLE8           = 1,
    BI_RLE4           =	2,
    BI_BITFIELDS      = 3,
    BI_JPEG           = 4,
    BI_PNG            = 5,
    BI_ALPHABITFIELDS = 6,
    BI_CMYK           = 11,
    BI_CMYKRLE8       = 12,
    BI_CMYKRLE4       = 13,
}

#[repr(C)]
pub struct Bmp {
    bmp_header:        Bmp_header,
    dib_header:        Bitmap_info_header,
    rgb_bitmask:       Option<RGB_bitmask>,
    alpha_bitmask:     Option<Alpha_bitmask>,
    color_space_gamma: Option<Color_space_gamma>,
    icc_color:         Option<ICC_color_prof>,
}

impl Bmp {

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        let bmp_header = buf.pread_with(0, scroll::LE)?;
        let size       = buf.pread_with::<u32>(BMP_HEADER_SIZE, scroll::LE)? as usize;
        let dib_header = buf.pread_with(BMP_HEADER_SIZE, scroll::LE)?;

        let mut rgb_bitmask       = None;
        let mut alpha_bitmask     = None;
        let mut color_space_gamma = None;
        let mut icc_color         = None;

        if size >= INFO_V2 {
            rgb_bitmask = Some(buf
                .pread_with::<RGB_bitmask>(BMP_HEADER_SIZE + INFO_V1, scroll::BE)?);
        }
        if size >= INFO_V3 {
            alpha_bitmask = Some(buf
                .pread_with::<Alpha_bitmask>(BMP_HEADER_SIZE + INFO_V2, scroll::BE)?);
        }
        if size >= INFO_V4 {
            color_space_gamma = Some(buf
                .pread_with::<Color_space_gamma>(BMP_HEADER_SIZE + INFO_V3, scroll::LE)?);
        }
        if size == INFO_V5 {
            icc_color = Some(buf
                .pread_with::<ICC_color_prof>(BMP_HEADER_SIZE + INFO_V4, scroll::LE)?);
        }

        Ok(Bmp {
            bmp_header,
            dib_header,
            rgb_bitmask,
            alpha_bitmask,
            color_space_gamma,
            icc_color,
        })

    }

    pub fn print(&self) -> Result<(), Error> {
        use ansi_term::Color;

        println!("BMP width: {}px height: {}px",
                 self.dib_header.width,
                 self.dib_header.height);

        println!();

        println!("{}:", Color::White.underline().paint("BITMAPFILEHEADER"));
        fmt_indentln(format!
                     ("File size: {:}\n  Reserved1: {:}\n  Reserved2: {:}\n  Offset to pixels: {:}",
                 self.bmp_header.file_size,
                 self.bmp_header.reserved1,
                 self.bmp_header.reserved2,
                 self.bmp_header.offset));
        println!();

        match self.dib_header.size as usize {

            INFO_V1 => {
                println!("{}:", Color::White.underline().paint("BITMAPINFOHEADER"));
            },
            INFO_V2 => {
                println!("{}{}{}:",
                         Color::White.underline().paint("BITMAP"),
                         Color::Yellow.underline().paint("V2"),
                         Color::White.underline().paint("INFOHEADER"))
            },
            INFO_V3 => {
                println!("{}{}{}:",
                         Color::White.underline().paint("BITMAP"),
                         Color::Yellow.underline().paint("V3"),
                         Color::White.underline().paint("INFOHEADER"))
            },
            INFO_V4 => {
                println!("{}{}{}:",
                         Color::White.underline().paint("BITMAP"),
                         Color::Yellow.underline().paint("V4"),
                         Color::White.underline().paint("INFOHEADER"))
            },
            INFO_V5 => {
                println!("{}{}{}:",
                         Color::White.underline().paint("BITMAP"),
                         Color::Yellow.underline().paint("V5"),
                         Color::White.underline().paint("INFOHEADER"))
            },
            _ => panic!("Invalid/Unsupported header"),

        }

        //
        // BITMAPINFOHEADER
        //
        fmt_indentln(format!("Bitmap width: {} height: {}",
                             self.dib_header.width,
                             self.dib_header.height));
        fmt_indentln(format!("Color planes: {}", self.dib_header.planes));
        fmt_indentln(format!("Bits per pixel: {}", self.dib_header.bpp));
        fmt_indentln(format!("Compression method: {}", self.dib_header.compression));
        fmt_indentln(format!("Image size: {}", self.dib_header.image_size));
        fmt_indentln(format!("Print resolution: {}x{}",
                             self.dib_header.h_resolution,
                             self.dib_header.v_resolution));

        fmt_indentln(format!("Number of colors in palette: {}", self.dib_header.palette));
        fmt_indentln(format!("Important colors: {}", self.dib_header.imp_colors));

        //
        // RGB bitmask
        //
        if let Some(rgb_bitmask) = &self.rgb_bitmask {
            println!();
            fmt_indentln(format!("Red channel bitmask: {:}",
                         Color::Red.paint(format!("  {:#010X}",rgb_bitmask.red_bitmask))));
            fmt_indentln(format!("Green channel bitmask: {}",
                         Color::Green.paint(format!("{:#010X}",rgb_bitmask.green_bitmask))));
            fmt_indentln(format!("Blue channel bitmask: {}",
                         Color::Blue.paint(format!(" {:#010X}",rgb_bitmask.blue_bitmask))));
        }

        //
        // Alpha bitmask
        //
        if let Some(alpha_bitmask) = &self.alpha_bitmask {
            fmt_indentln(format!("Alpha channel bitmask: {}",
                         Color::White.paint(format!("{:#010X}", alpha_bitmask.alpha_bitmask))));
        }

        //
        // Color space gamma
        //
        if let Some(csg) = &self.color_space_gamma {
            println!();
            fmt_indent(format!("Color space: "));
            for (i, b) in csg.color_space.iter().enumerate() {
                if i % 16 == 0 {
                    println!("");
                    fmt_indent(format!("{:02X} ", b));
                    // print!("\n{:#04X} ", b);
                }
                else {
                    print!("{:02X} ", b);
                }
            }
            println!();
            fmt_indentln(format!("Red channel gamma: {}",
                                 Color::Red.paint(format!("  {:#010X}", csg.gamma_red))));
            fmt_indentln(format!("Green channel gamma: {}",
                                 Color::Green.paint(format!("{:#010X}", csg.gamma_green))));
            fmt_indentln(format!("Blue channel gamma: {}",
                                 Color::Blue.paint(format!(" {:#010X}", csg.gamma_blue))));
        }

        //
        // ICC
        //
        if let Some(icc) = &self.icc_color {
            println!();
            fmt_indentln(format!("Intent:   {}",format!("{}", icc.intent)));
            fmt_indentln(format!("ICC data: {}",format!("{}", icc.icc_data)));
            fmt_indentln(format!("ICC size: {}",format!("{}", icc.icc_size)));
            fmt_indentln(format!("Reserved: {}",format!("{}", icc.reserved)));
        }


        Ok(())

    }

}
