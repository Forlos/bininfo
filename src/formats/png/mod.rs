use scroll::{self, Pread};

use failure::{
    Error,
};

use crate::format::{fmt_indent, fmt_indentln, fmt_png_header};

pub const PNG_HEADER: &'static [u8; PNG_HEADER_SIZE] = b"\x89PNG\x0D\x0A\x1A\x0A";
pub const PNG_HEADER_SIZE: usize = 8;

// TODO allow to use this as a argument and disable trimming in general.
const TRIM_INDEX: usize = 20;
// const IHDR_SIZE: usize = PNG_HEADER_SIZE + 25;

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Prefix {
    pub size: u32,
    id:       u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
pub struct Postfix {
    pub checksum: u32,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct RGB {
    red:     u8,
    green:   u8,
    blue:    u8,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct RGB_16 {
    red:     u16,
    green:   u16,
    blue:    u16,
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

#[derive(Debug)]
#[repr(C)]
struct Plte {
    prefix:  Prefix,
    rgb:     Vec<RGB>,
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
#[repr(C)]
struct Hist {
    prefix:  Prefix,
    entries: Vec<u16>,
    // Sum of all entries
    sum:     u64,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Phys {
    prefix:  Prefix,
    ppu_x:   u32,
    ppu_y:   u32,
    //0: unit is unknown
    //1: unit is the meter
    unit:    u8,
    postfix: Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Sbit {
    prefix:  Prefix,
    sbit:    sBIT,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Sbit0 {
    sig: u8,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Sbit2 {
    rgb: RGB,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Sbit3 {
    rgb: RGB,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Sbit4 {
    sig:   u8,
    alpha: u8,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Sbit6 {
    rgb:   RGB,
    alpha: u8,
}


#[derive(Debug)]
#[repr(C)]
#[allow(non_camel_case_types)]
// https://tools.ietf.org/html/rfc2083#page-23
enum sBIT {
    sBIT0(Sbit0),
    sBIT2(Sbit2),
    sBIT3(Sbit3),
    sBIT4(Sbit4),
    sBIT6(Sbit6),
}

#[derive(Debug)]
#[repr(C)]
struct Text {
    prefix:  Prefix,
    keyword: String,
    text:    String,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Time {
    prefix:  Prefix,
    year:    u16,
    month:   u8,
    day:     u8,
    hour:    u8,
    minute:  u8,
    second:  u8,
    postfix: Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Trns {
    prefix:  Prefix,
    data:    tRNS,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Trns0 {
    gray: u16,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Trns2 {
    rgb: RGB_16,
}

#[derive(Debug)]
#[repr(C)]
struct Trns3 {
    alpha: Vec<u8>,
}

#[derive(Debug)]
#[repr(C)]
#[allow(non_camel_case_types)]
enum tRNS {
    tRNS0(Trns0),
    tRNS2(Trns2),
    tRNS3(Trns3),
}

#[derive(Debug)]
#[repr(C)]
struct Ztxt {
    prefix:      Prefix,
    keyword:     String,
    comp_method: u8,
    comp_text:   Vec<u8>,
    postfix:     Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Srgb {
    prefix:    Prefix,
    rendering: u8,
    postfix:   Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Iccp {
    prefix:       Prefix,
    profile:      String,
    comp_method:  u8,
    comp_profile: Vec<u8>,
    postfix:      Postfix,
}


#[derive(Debug)]
#[repr(C)]
struct Itxt {
    prefix:        Prefix,
    keyword:       String,
    comp_flag:     u8,
    comp_method:   u8,
    lang_tag:      String,
    trans_keyword: String,
    comp_text:     Vec<u8>,
    postfix:       Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Splt {
    prefix:  Prefix,
    name:    String,
    depth:   u8,
    plt:     Vec<sPLT>,
    postfix: Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Splt8 {
    rgb:   RGB,
    alpha: u8,
    freq:  u16,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Splt16 {
    rgb:   RGB_16,
    alpha: u16,
    freq:  u16,
}

#[derive(Debug)]
#[repr(C)]
#[allow(non_camel_case_types)]
enum sPLT {
    sPLT8(Splt8),
    sPLT16(Splt16),
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Offs {
    prefix:  Prefix,
    x:       i32,
    y:       i32,
    unit:    i8,
    postfix: Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Pcal {
    prefix:           Prefix,
    name:             String,
    org_zero:         i32,
    org_max:          i32,
    equation:         u8,
    parameters_count: u8,
    unit_name:        String,
    parameters:       Vec<String>,
    postfix:          Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Scal {
    prefix:       Prefix,
    unit:         u8,
    pixel_width:  String,
    pixel_height: String,
    postfix:      Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Gifg {
    prefix:     Prefix,
    disposal:   u8,
    user_input: u8,
    delay:      u16,
    postfix:    Postfix,
}

#[derive(Debug)]
#[repr(C)]
struct Gifx {
    prefix:   Prefix,
    app_id:   [u8; 8],
    app_code: [u8; 3],
    app_data: Vec<u8>,
    postfix:  Postfix,
}

#[derive(Debug, Pread)]
#[repr(C)]
struct Ster {
    prefix:   Prefix,
    mode: u8,
    postfix:  Postfix,
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
    hist: Option<Hist>,
    phys: Option<Phys>,
    sbit: Option<Sbit>,
    text: Vec<Text>,
    time: Option<Time>,
    trns: Option<Trns>,
    ztxt: Vec<Ztxt>,
    srgb: Option<Srgb>,
    iccp: Option<Iccp>,
    itxt: Vec<Itxt>,
    splt: Vec<Splt>,
    //
    // PNGEXT 1.2 chunks https://pmt.sourceforge.io/specs/pngext-1.2.0-pdg-h20.html
    //
    offs: Option<Offs>,
    pcal: Option<Pcal>,
    scal: Option<Scal>,
    gifg: Vec<Gifg>,
    gifx: Vec<Gifx>,
    // frac: Vec<Frac>,
    //
    // PNGEXT 1.3 chunks
    //
    ster: Option<Ster>,
}

impl Png {

    pub fn parse(buf: &[u8]) -> Result<Self, Error> {

        const CHUNK_SIZE: usize = 12;
        let mut data = Vec::new();

        //
        // Critical chunks
        //
        let mut plte = None;
        let mut idat = Vec::new();
        let mut iend = None;

        //
        // Ancillary chunks
        //
        let mut bkgd = None;
        let mut chrm = None;
        let mut gama = None;
        let mut hist = None;
        let mut phys = None;
        let mut sbit = None;
        let mut text = Vec::new();
        let mut time = None;
        let mut trns = None;
        let mut ztxt = Vec::new();
        let mut srgb = None;
        let mut iccp = None;
        let mut itxt = Vec::new();
        let mut splt = Vec::new();


        //
        // PNGEXT 1.2 chunks https://pmt.sourceforge.io/specs/pngext-1.2.0-pdg-h20.html
        //
        let mut offs = None;
        let mut pcal = None;
        let mut scal = None;
        let mut gifg = Vec::new();
        let mut gifx = Vec::new();
        // let mut frac = Vec::new();

        //
        // PNGEXT 1.3 chunks
        //
        let mut ster = None;

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
                    let prefix = buf.pread_with::<Prefix>(index, scroll::BE)?;
                    let size = prefix.size as usize;
                    if size % 3 != 0 {
                        panic!("PLTE Length must be divisible by 3");
                    }
                    let mut rgb = Vec::new();
                    for i in 0..size / 3 {
                        rgb.push(buf.pread_with::<RGB>(index + 8 + i * 3,
                                                       scroll::BE)?)
                    }
                    plte = Some(Plte {
                        prefix,
                        rgb,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    });
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
                "hIST" => {
                    if let Some(plte) = &plte {
                        let hist_size = (plte.prefix.size / 3) as usize;
                        let mut entries = Vec::with_capacity(hist_size);
                        let mut sum = 0;
                        for i in 0..hist_size {
                            entries.push(buf.pread_with(index + 8 + i * 2, scroll::BE)?);
                            sum += entries[i] as u64;
                        }
                        hist = Some(Hist {
                            prefix: buf.pread_with(index, scroll::BE)?,
                            entries,
                            sum,
                            postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                        });
                    }
                    else {
                        panic!("Need PLTE chunk");
                    }
                },
                "pHYs" => {
                    phys = Some(buf.pread_with(index, scroll::BE)?);
                },
                "sBIT" => {
                    let color_type = ihdr.color;
                    let inner_sbit;
                    match color_type {
                        0 => {
                            inner_sbit = sBIT::sBIT0(buf.pread_with(index + 8, scroll::BE)?);
                        },
                        2 => {
                            inner_sbit = sBIT::sBIT2(buf.pread_with(index + 8, scroll::BE)?);
                        },
                        3 => {
                            inner_sbit = sBIT::sBIT3(buf.pread_with(index + 8, scroll::BE)?);
                        },
                        4 => {
                            inner_sbit = sBIT::sBIT4(buf.pread_with(index + 8, scroll::BE)?);
                        },
                        6 => {
                            inner_sbit = sBIT::sBIT6(buf.pread_with(index + 8, scroll::BE)?);
                        },
                        _ => {
                            panic!("Invalid color type for sBIT");
                        }
                    }
                    sbit = Some(Sbit {
                        prefix: buf.pread_with(index, scroll::BE)?,
                        sbit: inner_sbit,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    });
                },
                "tEXt" => {
                    let keyword = buf.pread::<&str>(index + 8)?.to_string();
                    let inner_text = std::str::from_utf8(
                        &buf[index + 9 + keyword.len()..index + size + 8])?.to_string();

                    let text_chunk = Text {
                        prefix: buf.pread_with(index, scroll::BE)?,
                        keyword,
                        text: inner_text,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    };

                    text.push(text_chunk);

                },
                "tIME" => {
                    time = Some(buf.pread_with(index, scroll::BE)?);
                },
                "tRNS" => {
                    let prefix = buf.pread_with::<Prefix>(index, scroll::BE)?;
                    let color_type = ihdr.color;
                    let data;
                    match color_type {
                        0 => {
                            data = tRNS::tRNS0(buf.pread_with(index + 8, scroll::BE)?);
                        },
                        2 => {
                            data = tRNS::tRNS2(buf.pread_with(index + 8, scroll::BE)?);
                        },
                        3 => {
                            let mut alpha = Vec::with_capacity(prefix.size as usize);
                            for i in 0..alpha.capacity() {
                                alpha.push(buf[index + 8 + i]);
                            }
                            data = tRNS::tRNS3(Trns3{ alpha });
                        },
                        _ => {
                            panic!("Invalid color type for tRNS");
                        },
                    }
                    trns = Some(Trns {
                        prefix,
                        data,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    });
                },
                "zTXt" => {
                    let keyword = buf.pread::<&str>(index + 8)?.to_string();
                    let comp_text = buf[index + 10 + keyword.len()..index + size + 8].to_vec();
                    let comp_method: u8 = buf.pread(index + 9 + keyword.len())?;

                    let ztxt_chunk = Ztxt {
                        prefix: buf.pread_with(index, scroll::BE)?,
                        keyword,
                        comp_method,
                        comp_text,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    };

                    ztxt.push(ztxt_chunk);

                },
                "sRGB" => {
                    srgb = Some(buf.pread_with(index, scroll::BE)?);
                },
                "iCCP" => {
                    let profile = buf.pread::<&str>(index + 8)?.to_string();
                    let comp_profile = buf[index + 10 + profile.len()..index + size + 8].to_vec();
                    let comp_method: u8 = buf.pread(index + 9 + profile.len())?;

                    iccp = Some(Iccp {
                        prefix: buf.pread_with(index, scroll::BE)?,
                        profile,
                        comp_method,
                        comp_profile,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    });

                },
                "iTXt" => {
                    let keyword     = buf.pread::<&str>(index + 8)?.to_string();
                    let comp_flag   = buf.pread(index + 9 + keyword.len())?;
                    let comp_method = buf.pread(index + 10 + keyword.len())?;

                    let lang_tag = buf.pread::<&str>(index + 11 + keyword.len())?.to_string();
                    let trans_keyword = buf.pread::<&str>(
                            index + 12 + keyword.len() + lang_tag.len())?.to_string();
                    let comp_text =
                        buf[index + 13 + keyword.len() + lang_tag.len() + trans_keyword.len()
                            ..index + size + 8].to_vec();


                    let itxt_chunk = Itxt {
                        prefix: buf.pread_with(index, scroll::BE)?,
                        keyword,
                        comp_flag,
                        comp_method,
                        lang_tag,
                        trans_keyword,
                        comp_text,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    };

                    itxt.push(itxt_chunk);

                },
                "sPLT" => {
                    let prefix = buf.pread_with::<Prefix>(index, scroll::BE)?;
                    let name   = buf.pread::<&str>(index + 8)?.to_string();
                    let depth  = buf.pread(index + 9 + name.len())?;

                    let length = prefix.size as usize - name.len() - 2;
                    let mut plt;

                    match depth {

                        8 => {
                            if length % 6 != 0 {
                                panic!("sPLT remaining length not divisible by 6");
                            }
                            plt = Vec::with_capacity(length / 6);
                            for _ in 0..length / 6 {
                                plt.push(sPLT::sPLT8(buf.pread_with::<Splt8>
                                         (index + 10 + name.len(), scroll::BE)?));
                            }
                        },
                        16 => {
                            if length % 10 != 0 {
                                panic!("sPLT remaining length not divisible by 10");
                            }
                            plt = Vec::with_capacity(length / 10);
                            for _ in 0..length / 6 {
                                plt.push(sPLT::sPLT16(buf.pread_with::<Splt16>
                                        (index + 10 + name.len(), scroll::BE)?));
                            }
                        },
                        _ => {
                            panic!("Invalid bit depth for sPLT");
                        },

                    }

                    let splt_chunk = Splt {
                        prefix,
                        name,
                        depth,
                        plt,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    };

                    splt.push(splt_chunk);

                },
                "oFFs" => {
                    offs = Some(buf.pread_with(index, scroll::BE)?);
                },
                "pCAL" => {
                    let prefix   = buf.pread_with::<Prefix>(index, scroll::BE)?;
                    let name     = buf.pread::<&str>(index + 8)?.to_string();
                    let org_zero = buf.pread_with(index + 9 + name.len(), scroll::BE)?;
                    let org_max  = buf.pread_with(index + 13 + name.len(), scroll::BE)?;
                    let equation = buf.pread_with(index + 17 + name.len(), scroll::BE)?;
                    let parameters_count = buf.pread_with::<u8>(index + 18 + name.len(), scroll::BE)?;
                    let unit_name = buf.pread::<&str>(index + 19 + name.len())?.to_string();
                    let mut parameters = Vec::with_capacity(parameters_count as usize);

                    let mut param_length = 0;

                    for _ in 0..parameters_count - 1 {
                        let parameter = buf.pread::<&str>(index + 20 + name.len() + unit_name.len())?.to_string();
                        param_length += parameter.len() + 1;
                        parameters.push(parameter);
                    }
                    let parameter = std::str::from_utf8(&buf[index + 20 + name.len() + unit_name.len() + param_length..index + size + 8])?.to_string();
                    parameters.push(parameter);

                    pcal = Some(Pcal {
                        prefix,
                        name,
                        org_zero,
                        org_max,
                        equation,
                        parameters_count,
                        unit_name,
                        parameters,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    });

                },
                "sCAL" => {
                    let unit = buf.pread(index + 8)?;
                    let pixel_width = buf.pread::<&str>(index + 9)?.to_string();
                    let pixel_height = std::str::from_utf8(&buf[index + 10 + pixel_width.len()..index + size + 8])?.to_string();
                    scal = Some(Scal {
                        prefix: buf.pread_with(index, scroll::BE)?,
                        unit,
                        pixel_width,
                        pixel_height,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    });
                },
                "gIFg" => {
                    let gifg_chunk = buf.pread_with(index, scroll::BE)?;
                    gifg.push(gifg_chunk);
                },
                "gIFx" => {
                    let prefix = buf.pread_with(index, scroll::BE)?;
                    let mut app_id = [0; 8];
                    app_id.copy_from_slice(&buf[index + 8..index + 16]);
                    let mut app_code = [0; 3];
                    app_code.copy_from_slice(&buf[index + 16..index + 19]);
                    let app_data = buf[index + 19..index + size + 8].to_vec();
                    let gifx_chunk = Gifx {
                        prefix,
                        app_id,
                        app_code,
                        app_data,
                        postfix: buf.pread_with(index + size + 8, scroll::BE)?,
                    };
                    gifx.push(gifx_chunk);
                },
                "sTER" => {
                    ster = Some(buf.pread_with(index, scroll::BE)?);
                }
                _ => {
                    use ansi_term::Color;
                    eprintln!("Error: {} ",
                              Color::Red.underline().paint(format!("Unsupported chunk: {}", s)));
                },
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

            bkgd,
            chrm,
            gama,
            hist,
            phys,
            sbit,
            text,
            time,
            trns,
            ztxt,
            srgb,
            iccp,
            itxt,
            splt,

            offs,
            pcal,
            scal,
            gifg,
            gifx,
            // frac,

            ster,
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

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![r->"Idx", rFr->"Red", rFg->"Green", rFb->"Blue"]);
            for (i, rgb) in plte.rgb.iter().enumerate() {
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
        // hIST
        //
        if let Some(hist) = &self.hist {
            fmt_png_header("hIST", &hist.prefix, &hist.postfix);

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![r->"Idx", rFb->"Frequency"]);
            for (i, entry) in hist.entries.iter().enumerate() {
                table.add_row(row![r->i,
                                   rFb->format!("{:.8}", (*entry as f64 / hist.sum as f64))]);
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
        // pHYs
        //
        if let Some(phys) = &self.phys {
            fmt_png_header("pHYs", &phys.prefix, &phys.postfix);
            let unit = {
                if phys.unit == 0 {
                    "unknown unit"
                }
                else if phys.unit == 1{
                    "meter"
                }
                else {
                    panic!("pHYs: Invalid unit specifier");
                }
            };
            fmt_indentln(format!("Pixels per {}, X axis: {}", unit, phys.ppu_x));
            fmt_indentln(format!("Pixels per {}, Y axis: {}", unit, phys.ppu_y));
            println!();
        }

        //
        // sBIT
        //
        if let Some(sbit) = &self.sbit {
            fmt_png_header("sBIT", &sbit.prefix, &sbit.postfix);
            match &sbit.sbit {
                sBIT::sBIT0(inner_sbit) => {
                    fmt_indentln(format!("Significant bits: {}", inner_sbit.sig));
                },
                sBIT::sBIT2(inner_sbit) => {
                    fmt_indentln(format!("{}",
                        Color::Red.paint(
                            format!("Significant bits red:   {}",inner_sbit.rgb.red))));
                    fmt_indentln(format!("{}",
                        Color::Green.paint(
                            format!("Significant bits green: {}",inner_sbit.rgb.green))));
                    fmt_indentln(format!("{}",
                        Color::Blue.paint(
                            format!("Significant bits blue:  {}",inner_sbit.rgb.blue))));
                },
                sBIT::sBIT3(inner_sbit) => {
                    fmt_indentln(format!("{}",
                        Color::Red.paint(
                            format!("Significant bits red:   {}",inner_sbit.rgb.red))));
                    fmt_indentln(format!("{}",
                        Color::Green.paint(
                            format!("Significant bits green: {}",inner_sbit.rgb.green))));
                    fmt_indentln(format!("{}",
                        Color::Blue.paint(
                            format!("Significant bits blue:  {}",inner_sbit.rgb.blue))));
                },
                sBIT::sBIT4(inner_sbit) => {
                    fmt_indentln(format!("Significant bits: {}", inner_sbit.sig));
                    fmt_indentln(format!("{}",
                        Color::White.paint(
                            format!("Significant bits alpha  {}",inner_sbit.alpha))));
                },
                sBIT::sBIT6(inner_sbit) => {
                    fmt_indentln(format!("{}",
                        Color::Red.paint(
                            format!("Significant bits red:   {}",inner_sbit.rgb.red))));
                    fmt_indentln(format!("{}",
                        Color::Green.paint(
                            format!("Significant bits green: {}",inner_sbit.rgb.green))));
                    fmt_indentln(format!("{}",
                        Color::Blue.paint(
                            format!("Significant bits blue:  {}",inner_sbit.rgb.blue))));
                    fmt_indentln(format!("{}",
                        Color::White.paint(
                            format!("Significant bits alpha  {}",inner_sbit.alpha))));
                },
            }
            println!();
        }

        //
        // tEXt
        //
        if self.text.len() > 0 {
            for (_i, chunk) in self.text.iter().enumerate() {
                fmt_png_header("tEXt", &chunk.prefix, &chunk.postfix);
                fmt_indentln(format!("{}: {}",
                                     Color::Blue.paint(&chunk.keyword), chunk.text));
            }
            println!();
        }

        //
        // tIME
        //
        if let Some(time) = &self.time {
            fmt_png_header("tIME", &time.prefix, &time.postfix);
            fmt_indentln(format!("Date: {}.{}.{} {}:{}:{}",
                                 time.day,
                                 time.month,
                                 time.year,
                                 time.hour,
                                 time.minute,
                                 time.second));
            println!();
        }
        //
        // tRNS
        //
        if let Some(trns) = &self.trns {
            fmt_png_header("tRNS", &trns.prefix, &trns.postfix);
            match &trns.data {
                tRNS::tRNS0(data) => {
                    fmt_indentln(format!("Gray: {:#06X}", data.gray));
                },
                tRNS::tRNS2(data) => {
                    fmt_indentln(format!("Red:   {}",
                        Color::Red.paint(format!("{:#06X}", data.rgb.red))));
                    fmt_indentln(format!("Green: {}",
                        Color::Green.paint(format!("{:#06X}", data.rgb.green))));
                    fmt_indentln(format!("Blue:  {}",
                        Color::Blue.paint(format!("{:#06X}", data.rgb.blue))));
                },
                tRNS::tRNS3(data) => {
                    let mut trimmed = false;
                    let mut table = Table::new();
                    let format = prettytable::format::FormatBuilder::new()
                        .column_separator(' ')
                        .borders(' ')
                        .padding(1, 1)
                        .build();
                    table.set_format(format);
                    table.add_row(row![r->"Idx", rFw->"Transparency"]);
                    for (i, entry) in data.alpha.iter().enumerate() {
                        table.add_row(row![r->i,
                                           rFw->format!("{:#04X}", entry)]);
                        if i == TRIM_INDEX {
                            trimmed = true;
                            break;
                        }
                    }
                    table.printstd();
                    if trimmed {
                        fmt_indentln(format!("Output trimmed..."));
                    }
                },
            }
            println!();
        }

        //
        // zTXt
        //
        if self.ztxt.len() > 0 {
            for (_i, chunk) in self.ztxt.iter().enumerate() {
                fmt_png_header("zTXt", &chunk.prefix, &chunk.postfix);
                fmt_indentln(format!("Compression method: {}", chunk.comp_method));
                fmt_indent(format!("{}: ", chunk.keyword));
                for (i, b) in chunk.comp_text.iter().enumerate() {
                    if i % 16 == 0 {
                        println!("");
                        fmt_indent(format!("{:02X} ", b));
                    }
                    else {
                        print!("{:02X} ", b);
                    }
                }
                println!();
            }
            println!();
        }

        //
        // sRGB
        //
        if let Some(srgb) = &self.srgb {
            fmt_png_header("sRGB", &srgb.prefix, &srgb.postfix);
            fmt_indent(format!("Rendering intent: "));
            match srgb.rendering {
                0 => {
                    println!("{} Perceptual", srgb.rendering);
                }
                1 => {
                    println!("{} Relative colorimetric", srgb.rendering);
                }
                2 => {
                    println!("{} Saturation", srgb.rendering);
                }
                3 => {
                    println!("{} Absolute colorimetric", srgb.rendering);
                }
                _ => {
                    panic!("Invalid rendering intent");
                }

            }
            println!();
        }
        //
        // iCCP
        //
        if let Some(iccp) = &self.iccp {
            fmt_png_header("iCCP", &iccp.prefix, &iccp.postfix);
            fmt_indentln(format!("Compression method: {}", iccp.comp_method));
            fmt_indent(format!("{}: ", iccp.profile));
            for (i, b) in iccp.comp_profile.iter().enumerate() {
                if i % 16 == 0 {
                    println!("");
                    fmt_indent(format!("{:02X} ", b));
                }
                else {
                    print!("{:02X} ", b);
                }
            }
            println!();
            println!();
        }
        //
        // iTXt
        //
        if self.itxt.len() > 0 {
            for (_i, chunk) in self.itxt.iter().enumerate() {
                fmt_png_header("iTXt", &chunk.prefix, &chunk.postfix);
                fmt_indentln(format!("Compression flag: {}", chunk.comp_flag));
                fmt_indentln(format!("Lang: {}",
                                     Color::Green.paint(&chunk.lang_tag)));
                fmt_indentln(format!("Keyword: {}",
                                     Color::Blue.paint(&chunk.keyword)));
                fmt_indentln(format!("Translated keyword: {}",
                                     Color::Purple.paint(&chunk.trans_keyword)));
                if chunk.comp_flag == 1 {
                    fmt_indentln(format!("Compression method: {}", chunk.comp_method));

                    fmt_indent(format!("{}: ", chunk.keyword));
                    for (i, b) in chunk.comp_text.iter().enumerate() {
                        if i % 16 == 0 {
                            println!("");
                            fmt_indent(format!("{:02X} ", b));
                        }
                        else {
                            print!("{:02X} ", b);
                        }
                    }
                    println!();
                }
                else {
                    fmt_indentln(format!("Text: {}",
                                         std::str::from_utf8(&chunk.comp_text)?));
                }
            }
            println!();
        }
        //
        // sPLT
        //
        if self.splt.len() > 0 {
            println!("{}({}) ",Color::White.underline().paint("sPLT"),
                     self.splt.len(),);

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Idx", "Depth",
                               Fr->"Red", Fg->"Green", Fb->"Blue",
                               Fw->"Alpha", "Freq"]);
            for (i, chunk) in self.splt.iter().enumerate() {
                fmt_png_header("sPLT", &chunk.prefix, &chunk.postfix);
                fmt_indentln(format!("Name: {}",
                                     Color::Blue.paint(&chunk.name)));
                for (j, palette) in chunk.plt.iter().enumerate() {
                    match palette {
                        sPLT::sPLT8(plt) => {
                            table.add_row(row![r->j,
                                               r->chunk.depth,
                                               rFr->plt.rgb.red,
                                               rFg->plt.rgb.green,
                                               rFb->plt.rgb.blue,
                                               rFw->plt.alpha,
                                               r->plt.freq]);
                            if j == TRIM_INDEX {
                                trimmed = true;
                                break;
                            }
                        },
                        sPLT::sPLT16(plt) => {
                            table.add_row(row![r->j,
                                               r->chunk.depth,
                                               rFr->plt.rgb.red,
                                               rFg->plt.rgb.green,
                                               rFb->plt.rgb.blue,
                                               rFw->plt.alpha,
                                               r->plt.freq]);
                            if j == TRIM_INDEX {
                                trimmed = true;
                                break;
                            }
                        }
                    }
                }
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
        // oFFs
        //
        if let Some(offs) = &self.offs {
            fmt_png_header("oFFs", &offs.prefix, &offs.postfix);
            fmt_indentln(format!("X: {}", offs.x));
            fmt_indentln(format!("Y: {}", offs.y));
            fmt_indentln(format!("Unit specifier: {}", offs.unit));
            println!();
        }

        //
        // pCAL
        //
        if let Some(pcal) = &self.pcal {
            fmt_png_header("pCAL", &pcal.prefix, &pcal.postfix);
            fmt_indentln(format!("Name: {}", pcal.name));
            fmt_indentln(format!("Original zero: {}", pcal.org_zero));
            fmt_indentln(format!("Original max: {}", pcal.org_max));
            fmt_indentln(format!("Equation type: {}", pcal.equation));
            fmt_indentln(format!("Unit name: {}", pcal.unit_name));
            fmt_indentln(format!("Number of parameters: {}", pcal.parameters_count));
            for param in &pcal.parameters {
                fmt_indentln(format!("{}", param));
            }
            println!();
        }

        //
        // sCAL
        //
        if let Some(scal) = &self.scal {
            fmt_png_header("sCAl", &scal.prefix, &scal.postfix);
            fmt_indentln(format!("Unit specifier: {}", scal.unit));
            fmt_indentln(format!("Pixel width: {}", scal.pixel_width));
            fmt_indentln(format!("Pixel height: {}", scal.pixel_height));
            println!();
        }

        //
        // gIFg
        //
        if self.gifg.len() > 0 {
            println!("{}({})",Color::White.underline().paint("gIFg"), self.gifg.len());
            for chunk in &self.gifg {
                fmt_png_header("gIFg", &chunk.prefix, &chunk.postfix);
                fmt_indentln(format!("Disposal method: {}", chunk.disposal));
                fmt_indentln(format!("User input flag: {}", chunk.user_input));
                fmt_indentln(format!("Delay time: {}", chunk.delay));
                println!();
            }
        }

        //
        // gIFx
        //
        if self.gifx.len() > 0 {
            println!("{}({})",Color::White.underline().paint("gIFx"), self.gifg.len());
            for chunk in &self.gifg {
                fmt_png_header("gIFx", &chunk.prefix, &chunk.postfix);
                fmt_indentln(format!("Application identifier: {}", chunk.disposal));
                fmt_indentln(format!("Authentication code: {}", chunk.user_input));
                fmt_indentln(format!("Application data: {}", chunk.delay));
                println!();
            }
        }

        //
        // sTER
        //
        if let Some(ster) = &self.ster {
            fmt_png_header("sTER", &ster.prefix, &ster.postfix);
            match ster.mode {
                0 => {
                    fmt_indentln(format!("Mode: {}: {}", ster.mode, "cross-fuse layout"));
                },
                1 => {
                    fmt_indentln(format!("Mode: {}: {}", ster.mode, "diverging-fuse layout"));
                },
                _ => {
                    panic!("Invalid mode {} for sTER chunk", ster.mode);
                },
            }
        }

        //
        // IDAT
        //
        {
            println!("{}({}) {} {} {} {} {} {}",
                Color::White.underline().paint("IDAT"),
                self.idat.len(),
                Color::Purple.paint(format!("Compression method: {}",self.zlib.cmf << 4 >> 4)),
                Color::Cyan.paint(format!("Compression info: {}",self.zlib.cmf >> 4)),
                Color::Green.paint(format!("Checksum: {:#07b}", self.zlib.flg << 3 >> 3)),
                Color::Red.paint(format!("Dict: {:#03b}", self.zlib.flg << 2 >> 7)),
                Color::Yellow.paint(format!("Compression level: {:#04b}", self.zlib.flg >> 6)),
                Color::Fixed(221).paint(format!("Adler: {:#010X}", self.zlib.adler)));

            let mut trimmed = false;
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(1, 1)
                .build();
            table.set_format(format);
            table.add_row(row![l->"Idx", l->"Size", l->"Checksum"]);

            for (i, idat) in self.idat.iter().enumerate() {
                table.add_row(row![r->i,
                                   rFy->idat.prefix.size,
                                   rFg->format!("{:#010X}", idat.postfix.checksum)]);
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
        // IEND
        //
        if let Some(iend) = &self.iend {
            fmt_png_header("IEND", &iend.prefix, &iend.postfix);
        }

        Ok(())

    }

}
