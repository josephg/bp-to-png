extern crate lodepng;
extern crate rustc_serialize;
//extern crate url;

use lodepng::Bitmap;
use rustc_serialize::json;
use std::path::Path;
use rustc_serialize::base64::FromBase64;
//use url::Url;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Color(u8, u8, u8);

/*
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
}
*/

const COLOR_OF_BASE: [Color; 8] = [
    Color(9, 25, 27), // solid
    Color(255, 255, 255), // nothing
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

fn render_png() {
    let path = Path::new("./foo.png");
    let mut data: [Color; 512*512] = [Color(0, 0, 0); 512*512];

    let mut i = 0 as u8;
    for p in data.iter_mut() {
        p.0 = i;
        p.1 = 255 - i;
        i = i.wrapping_add(1);
    }

    match lodepng::encode24_file(path, &data, 512, 512) {
        Ok(()) => println!("Encoded to {}", path.display()),
        Err(err) => println!("Encoding error: {}", err),
    }

}

fn parse_json() {
//    let data = "{\"v\":2,\"offx\":17,\"offy\":6,\"img\":\"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAMAAAABCAYAAAAb4BS0AAAAFUlEQVQIW2NkbGL4z3CCgYHBjoEBABTlAopSl9HTAAAAAElFTkSuQmCC\"}";
    let data = "{\"v\":2,\"offx\":41,\"offy\":10,\"img\":\"data:image/png;base64,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\"}";
    let d: WorldV2Data = json::decode(&data).unwrap();
    //println!("Data {:?}", d);

    let base64_data = &d.img["data:image/png;base64,".len()..];
    //println!("{:?}", base64_data);
    let png_data = base64_data.from_base64().unwrap();

    //println!("{:?}", png_data);

    let bitmap = lodepng::decode24(png_data).unwrap();
    //println!("{:?}", bitmap);



    let mut result_pixels = vec![Color(0,0,0); bitmap.width * bitmap.height];

    for y in 0..bitmap.height {
        for x in 0..bitmap.width {
            let idx = x+y*bitmap.width;
            let bv = bitmap.buffer.get(idx).unwrap().r;
            let sv = bitmap.buffer.get(idx).unwrap().g;
            result_pixels[idx] = if sv != 0 {
                color_of_s(sv)
            } else {
                COLOR_OF_BASE[bv as usize]
            };
        }
    }

    let path = Path::new("./foo.png");
    lodepng::encode24_file(path, &result_pixels, bitmap.width, bitmap.height).unwrap()
}

fn main() {
    println!("Hello, world!");

    parse_json();
}
