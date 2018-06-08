#![allow(dead_code)]

extern crate png;
extern crate lodepng;
extern crate rustc_serialize;

use png::HasParameters;
use std::io::Error as IOError;
use std::io::Write;
use std::fs::File;

use lodepng::Error as LodeError;

use rustc_serialize::json;
use rustc_serialize::base64::FromBase64;
//use url::Url;

use std::time::SystemTime;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Color(u8, u8, u8);

/*
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum BPVal {
    Nothing = 0,
    Solid,
    ThinSolid,
    Bridge,
    Positive,
    Negative,
    Shuttle,
    ThinShuttle,
}*/

// fn num_to_bpval(v: u8) -> BPVal {
//     unsafe {
//         std::mem::transmute::<u8, BPVal>(v)
//     }
// }

const NOTHING: u8 = 0;
const SOLID: u8 = 1;
const THINSOLID: u8 = 2;
const BRIDGE: u8 = 3;
const POSITIVE: u8 = 4;
const NEGATIVE: u8 = 5;
const SHUTTLE: u8 = 6;
const THINSHUTTLE: u8 = 7;

const COLOR_OF_BASE: [Color; 8] = [
    Color(9, 25, 27), // SOLID
    Color(255, 255, 255), // NOTHING
    Color(181, 181, 181), // thinsolid

    Color(92, 204, 92), // positive
    Color(214, 87, 41), // negative

    Color(26, 126, 213), // bridge

    Color(185, 60, 174), // ribbon
    Color(108, 30, 217), // ribbonbridge
];
    
const COLOR_OF_SHUTTLE: Color = Color(147, 40, 189);
const COLOR_OF_THINSHUTTLE: Color = Color(216, 135, 248);

fn color_of_s(sv: u8) -> Color {
    // Upper 4 bits = 64 for shuttle, 128 for thinshuttle.
    // Lower 4 bits = up right down left connectivity.
    if sv & 64 != 0 { COLOR_OF_SHUTTLE } else { COLOR_OF_THINSHUTTLE }
}

#[derive(RustcDecodable, RustcEncodable, Debug)]
struct WorldV2Data {
    v: u8, // Must be 2.
    offx: i32,
    offy: i32,
    img: String,
}

struct EncodeResult {
    data: Vec<u8>,
    w: usize,
    h: usize,
}
fn parse_json(data: &str, cell_width: usize, border: usize) -> Result<EncodeResult, LodeError> {
    assert!(cell_width >= border);

    let d: WorldV2Data = json::decode(&data).unwrap();
    //println!("Data {:?}", d);

    let base64_data = &d.img["data:image/png;base64,".len()..];
    //println!("{:?}", base64_data);
    let png_data = base64_data.from_base64().unwrap();

    //println!("{:?}", png_data);

    let bitmap = lodepng::decode24(png_data).unwrap();
    //println!("{:?}", bitmap);

    let bwidth = bitmap.width;
    let bheight = bitmap.height;


    // const cell_width: usize = 16;
    

    let b0: usize = border; // border width (eg 1)
    let b1: usize = cell_width - b0; // Border high end (eg 7)

    let (out_w, out_h) = (cell_width * (bwidth + 2), cell_width * (bheight + 2));
    let mut result_pixels = vec![COLOR_OF_BASE[0]; (out_w * out_h) as usize];

    let get = |x: usize, y: usize| -> (u8, u8) {
        // Interestingly, because of overflow wrap-around this will still be
        // correct for negative x and y values (Although it might abort in
        // debug mode)
        if x >= bwidth || y >= bheight { return (NOTHING, NOTHING); }

        let c = bitmap.buffer.get(x+y*bitmap.width).unwrap();
        (c.r, c.g)
    };

    for y in 0..bheight {
        for x in 0..bwidth {
            let (bv, sv) = get(x, y);

            if bv == NOTHING { continue; } // We've initialized the image with the solid colour anyway.

            let col = if sv != NOTHING {
                //println!("{}", sv & 0b1111);
                color_of_s(sv)
            } else {
                COLOR_OF_BASE[bv as usize]
            };

            let base = cell_width * (1+x + (1+y)*out_w);

            // Cache these for the corners below.
            let sl = if sv != 0 && x > 0 { get(x - 1, y).1 } else { 0 };
            let sr = if sv != 0 && x < bwidth-1 { get(x+1, y).1 } else { 0 };
            let su = if sv != 0 && y > 0 { get(x, y-1).1 } else { 0 };
            let sd = if sv != 0 && y < bheight-1 { get(x, y+1).1 } else { 0 };

            for py in 0..cell_width {
                let col = if sv != 0 && (
                    // White on top and bottom
                    (py < b0 && (sv&0b0001)==0) ||
                    (py >= b1 && (sv&0b0100)==0)
                ) { COLOR_OF_BASE[SOLID as usize] } else { col };

                for px in 0..cell_width {
                    let col = if sv != 0 && (
                        // Left and right
                        (px <  b0 && (sv&0b1000)==0) ||
                        (px >= b1 && (sv&0b0010)==0) ||

                        // Corners. There's probably a nicer way to write this.
                        (px <  b0 && py <  b0 && (su & 0b1000 == 0 && sl & 0b0001 == 0)) || // Top left corner
                        (px >= b1 && py <  b0 && (su & 0b0010 == 0 && sr & 0b0001 == 0)) || // Top right corner
                        (px <  b0 && py >= b1 && (sd & 0b1000 == 0 && sl & 0b0100 == 0)) || // bot left corner
                        (px >= b1 && py >= b1 && (sd & 0b0010 == 0 && sr & 0b0100 == 0)) || // bot right corner
                        false
                    ) {
                        COLOR_OF_BASE[SOLID as usize]
                    } else { col };

                    result_pixels[base + px + py * out_w] = col;
                }
            }
        }
    }
    // println!("encoded data. Generating png {} x {}", out_w, out_h);

    // lodepng::encode24_file("foo.png", &result_pixels, out_w, out_h)?;

    println!("encoded data. Generating png");

    let mut start = SystemTime::now();
    let out_data = lodepng::encode24(&result_pixels, out_w, out_h)?;
    println!("lodepng generated a {} byte PNG in {:?}", out_data.len(), SystemTime::now().duration_since(start).unwrap());

    let mut vec = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut vec, out_w as u32, out_h as u32);
        encoder.set(png::ColorType::RGB);
        // let data_u8 = result_pixels.iter().flat_map(|&c| [c.0, c.1, c.2, 255].iter()).collect::<Vec<u8>>();

        // Flatten vector of colors into a vector of RGB bytes
        let mut data_u8 = vec![0; out_w * out_h * 3];

        for y in 0..out_h {
            for x in 0..out_w {
                let base = (x + y * out_w) * 3;
                let col = result_pixels[x + y * out_w];
                data_u8[base] = col.0;
                data_u8[base+1] = col.1;
                data_u8[base+2] = col.2;
                // data_u8[base+3] = 255;
            }
        }

        start = SystemTime::now();
        encoder.write_header().unwrap().write_image_data(data_u8.as_slice()).unwrap();
    }
    println!("png generated a {} byte PNG in {:?}", vec.len(), SystemTime::now().duration_since(start).unwrap());

    Result::Ok(EncodeResult {
        // data: out_data,
        data: vec,
        w: out_w,
        h: out_h
    })
}

fn write_file(filename: &str, data: &Vec<u8>) -> Result<(), IOError> {
    File::create(filename)?.write_all(data)
}

fn main() {
    let data = "{\"v\":2,\"offx\":12,\"offy\":5,\"img\":\"data:image/png;base64,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\"}";
    let result = parse_json(data, 16, 2).unwrap();

    let filename = "./foo.png";
    write_file(filename, &result.data).unwrap();
    println!("Wrote {:?} at size {} x {}", filename, result.w, result.h);
}
