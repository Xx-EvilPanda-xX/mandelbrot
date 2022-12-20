use std::env;
use image::ImageBuffer;
use math::ComplexPoint;
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

    let thread_ids = start_render_jobs(&mut pool, &slices, &config, dims);

    let buffer = match pool.join_all(dims, &thread_ids) {
        Ok(b) => b,
        Err(e) => {
            panic!("Error: {}", e)
        }
    };
    
    println!(
        "Finished rendering across {} threads, writing to file...",
        config.num_threads
    );

    let img_buf: ImageBuffer<image::Rgb<_>, _> = image::ImageBuffer::from_raw(config.dimensions.0, config.dimensions.1, &buffer[..]).unwrap();
    img_buf.save(config.file_name.as_str()).expect("Failed to save img");
}

fn get_slices(config: &Config) -> Vec<Slice> {
    let mut slices = Vec::new();
    let rows_per_slice = config.dimensions.1 / config.num_threads;
    for y in 0..config.dimensions.1 {
        if y % rows_per_slice == 0 {
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
) -> Vec<u32> {
    let mut thread_ids = Vec::new();

    for i in 0..config.num_threads.try_into().unwrap() {
        let local_dims = slices[i].dims;
        let pos = slices[i].pos;
        let lr = config.lower_left.clone();
        let ur = config.upper_right.clone();
        let j = config.julia.clone();

        match pool.run_job(move || {
            render(
                local_dims,
                dims,
                pos,
                j.as_ref(),
                &lr,
                &ur
            )
        }) {
            Ok(thread_id) => thread_ids.push(thread_id),
            Err(e) => {
                panic!("Error: {}", e)
            }
        };
    }

    thread_ids
}

struct Slice {
    dims: (u32, u32),
    pos: (u32, u32),
}

fn render(
    local_dims: (u32, u32),
    global_dims: (u32, u32),
    pos: (u32, u32),
    julia: Option<&ComplexPoint<f64>>,
    lower_left: &ComplexPoint<f64>,
    upper_right: &ComplexPoint<f64>
) -> Vec<u8> {
    const MAX_ITERS: u32 = 255;
    let mut buf = vec![0; local_dims.0 as usize * local_dims.1 as usize * 3];

    let (w, h) = local_dims;
    for y in 0..h {
        for x in 0..w {
            let pix = (x + pos.0, y + pos.1);
            let c = pixel_to_complex(pix, global_dims, lower_left, upper_right);
            let col = match mandel_iter(c, julia, MAX_ITERS) {
                Some(val) => val,
                None => 0,
            };

            let (r, g, b) = gradient(col, MAX_ITERS, pix);
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

const RED_COEF: (bool, u8) = (true, 1);
const GREEN_COEF: (bool, u8) = (false, 2);
const BLUE_COEF: (bool, u8) = (false, 1);

fn gradient(iters: u32, max_iters: u32, _pos: (u32, u32)) -> (u8, u8, u8) {
    assert!(iters <= max_iters);
    let iters = iters as u64;
    let max_iters = max_iters as u64;

    let r = if RED_COEF.0 {
        wraping_add(0, iters * iters * iters, max_iters) as u8 / RED_COEF.1
    } else {
        (iters as f32 / max_iters as f32 * 255.0) as u8 / RED_COEF.1
    };


    let g = if GREEN_COEF.0 {
        wraping_add(0, iters * iters * iters * iters, max_iters) as u8 / GREEN_COEF.1
    } else {
        (iters as f32 / max_iters as f32 * 255.0) as u8 / GREEN_COEF.1
    };

    let b = if BLUE_COEF.0 {
        wraping_add(0, iters * iters * iters * iters * iters, max_iters) as u8 / BLUE_COEF.1
    } else {
        (iters as f32 / max_iters as f32 * 255.0) as u8 / BLUE_COEF.1
    };

    (r, g, b)
}

fn wraping_add(mut addend1: u64, addend2: u64, wrap_at: u64) -> u64 {
    if addend1 >= wrap_at {
        addend1 = wraping_add(0, addend1, wrap_at);
    }

    let add = addend1 + addend2;
    if add < wrap_at {
        return add;
    }

    let diff = addend2 - (wrap_at - addend1);
    diff % wrap_at
}

fn _gradient2(iters: u32, max_iters: u32) -> (u8, u8, u8) {
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

fn parse_args(args: &[String]) -> Config {
    if args.len() != 8 {
        println!("USAGE: {} [img_width] [img_height] [ll.x,ll.y] [ur.x,ur.y] [julia] [file_name] [num_threads]", args[0]);
        std::process::exit(-1);
    }

    let buffer_width = args[1].parse().expect("Error: failed to parse buffer width");
    let buffer_height = args[2].parse().expect("Error: failed to parse buffer height");

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

    let num_threads = args[7].parse().expect("Failed to parse number of threads");

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
        first.parse().expect("Error: failed to parse real"),
        second.parse().expect("Error: failed to parse imaginary"),
    ))
}
