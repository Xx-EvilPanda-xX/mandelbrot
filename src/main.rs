use math::ComplexPoint;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use std::fs;
use std::env;
use std::str::FromStr;

mod test;
mod math;
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args);
    let dims = config.dimensions;
    let mut buffer = vec![0; dims.0 as usize * dims.1 as usize];
    render(&mut buffer, dims, &config.lower_left, &config.upper_right);
    write_to_img(config.file_name.as_str(), &buffer, dims)
}

fn write_to_img(file_name: &str, buf: &[u8], dim: (u32, u32)) {
    let file = fs::File::create(file_name).expect("failed to create file");
    let encoder = PngEncoder::new(file);
    encoder.write_image(buf, dim.0, dim.1, ColorType::L8).expect("couldn't write image");
}

fn render(buf: &mut [u8], dims: (u32, u32), lower_left: &ComplexPoint<f64>, upper_right: &ComplexPoint<f64>) {
    if buf.len() as u32 != dims.0 * dims.1 {
        panic!("Incorrect buffer length");
    }

    let (w, h) = (dims.0 as usize, dims.1 as usize);
    for y in 0..h {
        for x in 0..w {
            let c = pixel_to_complex((x as u32, y as u32), dims, lower_left, upper_right);
            buf[(y * w) + x] = match mandel_iter(&c, 255) {
                Some(val) => val as u8,
                None => 0
            }
        }
    }
}

fn mandel_iter(c: &ComplexPoint<f64>, iters: u32) -> Option<u32> {
    let mut z = ComplexPoint::new(0.0, 0.0);
    for i in 0..iters {
        if !is_in_circle((z.re, z.im), (0.0, 0.0), 2.0) {
            return Some(i);
        }
        z = z.mul(&z).add(c);
    }

    None
}

fn pixel_to_complex(pixel: (u32, u32), dims: (u32, u32), lower_left: &ComplexPoint<f64>, upper_right: &ComplexPoint<f64>) -> ComplexPoint<f64> {
    assert!(pixel < dims);
    assert!(lower_left.re < upper_right.re);
    assert!(lower_left.im < upper_right.im);

    let complex_width = upper_right.re - lower_left.re;
    let complex_height = upper_right.im - lower_left.im;

    let x = (complex_width * pixel.0 as f64) / dims.0 as f64;
    let y = (complex_height * pixel.1 as f64) / dims.1 as f64;
    
    ComplexPoint::new(lower_left.re + x, lower_left.im + y)
}

fn is_in_circle(point: (f64, f64), circle_pos: (f64, f64), rad: f64) -> bool {
    (point.0 - circle_pos.0) * (point.0 - circle_pos.0) +
    (point.1 - circle_pos.1) * (point.1 - circle_pos.1) <= rad * rad
}

struct Config {
    dimensions: (u32, u32),
    lower_left: ComplexPoint<f64>,
    upper_right: ComplexPoint<f64>,
    file_name: String
}

fn parse_args(args: &Vec<String>) -> Config {
    if args.len() != 6 {
        println!("USAGE: {} img_width img_height l_l.x,l_l.y u_r.x,u_r.y file_name", args[0]);
        std::process::exit(-1);
    }

    let buffer_width = u32::from_str(args[1].as_str()).expect("Error: failed to parse buffer width");
    let buffer_height = u32::from_str(args[2].as_str()).expect("Error: failed to parse buffer height");
    
    let lower_left = match parse_complex(&args[3]) {
        Ok(val) => val,
        Err(e) => panic!("Error: failed to parse complex number because: {}", e)
    };

    let upper_right = match parse_complex(&args[4]) {
        Ok(val) => val,
        Err(e) => panic!("Error: failed to parse complex number because: {}", e)
    };

    Config { dimensions: (buffer_width, buffer_height), lower_left, upper_right, file_name: args[5].clone() }
}

fn parse_complex(string: &String) -> Result<ComplexPoint<f64>, String> {
    let index = match string.find(',') {
        Some(i) => i,
        None => return Err(String::from("must include comma between values"))
    };

    let first = &string[..index];
    let second = &string[index + 1..];

    Ok(ComplexPoint::new(f64::from_str(first).expect("Error: failed to parse real"),
     f64::from_str(second).expect("Error: failed to parse imaginary")))
}

