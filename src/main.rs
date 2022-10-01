use core::panic;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use math::ComplexPoint;
use std::env;
use std::fs;
use std::str::FromStr;
use thread_pool::ThreadPool;
mod math;
#[cfg(test)]
mod test;
mod thread_pool;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = parse_args(&args);
    let dims = config.dimensions;

    let slices = get_slices(&config);
    let mut pool = ThreadPool::new(config.num_threads);

    start_render_jobs(&mut pool, &slices, &config, dims);

    let buffer = match pool.join_all(dims) {
        Ok(b) => b,
        Err(e) => {
            panic!("Error: {}", e)
        }
    };
    println!(
        "Finished rendering across {} threads, writing to file...",
        config.num_threads
    );

    write_to_img(config.file_name.as_str(), &buffer[..], dims);
}

fn get_slices(config: &Config) -> Vec<Slice> {
    let mut slices = Vec::new();
    let rows_per_slice = config.dimensions.1 / config.num_threads;
    for y in 0..config.dimensions.1 {
        if y % rows_per_slice == 0 || y == config.dimensions.1 {
            let dim_y = if y + rows_per_slice <= config.dimensions.1 {
                rows_per_slice
            } else {
                config.dimensions.1 - y
            };
            
            slices.push(
                Slice {
                    dims: (config.dimensions.0, dim_y),
                    pos: (0, y)
                }
            );
        }
    }

    slices
}

fn start_render_jobs(
    pool: &mut ThreadPool,
    slices: &[Slice],
    config: &Config,
    dims: (u32, u32)
) {
    for i in 0..config.num_threads.try_into().unwrap() {
        let local_dims = slices[i].dims;
        let pos = slices[i].pos;
        let lr = config.lower_left.clone();
        let ur = config.upper_right.clone();
        let j = config.julia.clone();

        match pool.run_job(Box::new(move || {
            let julia = j.as_ref();
            render(local_dims, dims, pos, julia, &lr, &ur)
        })) {
            Ok(_) => {}
            Err(e) => {
                panic!("Error: {}", e)
            }
        };
    }
}

struct Slice {
    dims: (u32, u32),
    pos: (u32, u32),
}

fn write_to_img(file_name: &str, buf: &[u8], dim: (u32, u32)) {
    let file = fs::File::create(file_name).expect("failed to create file");
    let encoder = PngEncoder::new(file);
    encoder
        .write_image(buf, dim.0, dim.1, ColorType::Rgb8)
        .expect("couldn't write image");
}

fn render(
    local_dims: (u32, u32), 
    global_dims: (u32, u32), 
    pos: (u32, u32), 
    julia: Option<&ComplexPoint<f64>>, 
    lower_left: &ComplexPoint<f64>, 
    upper_right: &ComplexPoint<f64>
) -> Vec<u8> {
    let mut buf = vec![0; local_dims.0 as usize * local_dims.1 as usize * 3];

    let (w, h) = local_dims;
    for y in 0..h {
        for x in 0..w {
            let pix = (x + pos.0, y + pos.1);
            let c = pixel_to_complex(pix, global_dims, lower_left, upper_right);
            let col = match mandel_iter(c, julia, 255) {
                Some(val) => val,
                None => 0,
            };

            let (r, g, b) = gradient(col, 255);
            let y_i = y as usize;
            let w_i = w as usize;
            let x_i = x as usize;
            buf[(y_i * w_i * 3) + (x_i * 3)] = r; // red
            buf[(y_i * w_i * 3) + (x_i * 3) + 1] = g; // green
            buf[(y_i * w_i * 3) + (x_i * 3) + 2] = b; // blue
        }
    }

    buf
}

fn gradient(iters: u32, max_iters: u32) -> (u8, u8, u8) {
    assert!(iters <= max_iters);
    const CHANNELS: usize = 3;
    let mut color: [u8; 3] = [0; CHANNELS];
    let color_range = ((255 * CHANNELS) / max_iters as usize) as u8;
    let mut iters = iters;

    for i in 0..3 {
        for _ in 0..255 {
            if iters == 0 || color[i] == 255 {
                break;
            }
    
            color[i] += color_range;

            let lhs = if i < 1 { CHANNELS - 1 } else { i - 1 };
            let rhs = if i > CHANNELS - 2 { 0 } else { i + 1 };

            let lhs_distance = (lhs as i32 - i as i32).abs();
            let rhs_distance = (rhs as i32 - i as i32).abs();
            if  lhs_distance == rhs_distance {
                // color[lhs] += color_range - (color_range / 3) * 2;
                // color[rhs] += color_range - (color_range / 3) * 2;
            }
            else if lhs_distance < rhs_distance {
                // color[lhs] += color_range - (color_range / 3) * 1;
                // color[rhs] += color_range - (color_range / 3) * 1;
            }
            else if lhs_distance > rhs_distance {
                // color[lhs] += color_range - (color_range / 3) * 1;
                // color[rhs] += color_range - (color_range / 3) * 1;
            }

            iters -= 1;
        }
    }

    (color[0], color[1], color[2])
}

fn mandel_iter(c: ComplexPoint<f64>, julia: Option<&ComplexPoint<f64>>, iters: u32) -> Option<u32> {
    let mut z = c.clone();
    for i in 0..iters {
        if !is_in_circle((z.re, z.im), (0.0, 0.0), 2.0) {
            return Some(i);
        }
        z = z.mul(&z).add(match julia {
            Some(j) => j,
            None => &c,
        });
    }

    None
}

fn pixel_to_complex(
    pixel: (u32, u32),
    dims: (u32, u32),
    lower_left: &ComplexPoint<f64>,
    upper_right: &ComplexPoint<f64>
) -> ComplexPoint<f64> {
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
    (point.0 - circle_pos.0) * (point.0 - circle_pos.0)
        + (point.1 - circle_pos.1) * (point.1 - circle_pos.1)
        <= rad * rad
}

struct Config {
    dimensions: (u32, u32),
    lower_left: ComplexPoint<f64>,
    upper_right: ComplexPoint<f64>,
    julia: Option<ComplexPoint<f64>>,
    file_name: String,
    num_threads: u32,
}

fn parse_args(args: &Vec<String>) -> Config {
    if args.len() != 8 {
        println!("USAGE: {} [img_width] [img_height] [ll.x,ll.y] [ur.x,ur.y] [julia] [file_name] [num_threads]", args[0]);
        std::process::exit(-1);
    }

    let buffer_width =
        u32::from_str(args[1].as_str()).expect("Error: failed to parse buffer width");
    let buffer_height =
        u32::from_str(args[2].as_str()).expect("Error: failed to parse buffer height");

    let lower_left = match parse_complex(&args[3]) {
        Ok(val) => val,
        Err(e) => panic!(
            "Error: failed to parse complex number because: {} (Expected complex point)",
            e
        ),
    };

    let upper_right = match parse_complex(&args[4]) {
        Ok(val) => val,
        Err(e) => panic!(
            "Error: failed to parse complex number because: {} (Expected complex point)",
            e
        ),
    };

    let julia = match args[5].as_str() {
        "none" => None,
        _ => {
            let j = match parse_complex(&args[5]) {
                Ok(val) => val,
                Err(e) => panic!("Error: failed to parse complex number because: {} (Expected complex point or `none`)", e)
            };
            Some(j)
        }
    };

    let num_threads = u32::from_str(args[7].as_str()).expect("Failed to parse number of threads");

    Config { 
        dimensions: (buffer_width, buffer_height),
        lower_left,
        upper_right,
        julia,
        file_name: args[6].clone(),
        num_threads
    }
}

fn parse_complex(string: &String) -> Result<ComplexPoint<f64>, String> {
    let index = match string.find(',') {
        Some(i) => i,
        None => {
            return Err(String::from(
                "structure must be two floating point values separated by a comma",
            ))
        }
    };

    let first = &string[..index];
    let second = &string[index + 1..];

    Ok(ComplexPoint::new(
        f64::from_str(first).expect("Error: failed to parse real"),
        f64::from_str(second).expect("Error: failed to parse imaginary"),
    ))
}
