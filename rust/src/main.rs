use rand::{Rng, seq::SliceRandom};
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::exit;
use std::env;
use image::GenericImageView;
use std::fs::File;
use gif::{Frame, Encoder, Repeat, SetParameter};

// Usage Examples
// mutant 2 250 image.jpg (use mode number 2)
// mutant 1,3,5 100 image.jpg image2.png (use modes 1, 3, and 5, a delay of 100ms, with 2 images)

fn main() {
    let num_modes = 8;
    let args: Vec<String> = env::args().collect();
    let mut modes: Vec<u32> = args[1].split(",").map(|x| x.parse::<u32>().unwrap()).collect();

    if modes.len() == 0 {
        exit(0);
    }

    for mode in modes.iter() {
        if *mode == 0 || *mode > num_modes {
            println!("Invalid mode: {}", mode);
            exit(0);
        }
    }

    let delay_ms = args[2].parse::<u16>().unwrap_or_else(|_| {
        println!("Wrong delay.");
        exit(0);
    });

    let delay = (delay_ms as f64 / 10.0) as u16;

    if delay == 0 {
        println!("Delay is too short.");
        exit(0);
    }
    else if delay > 1000 {
        println!("Delay is too long.");
        exit(0);
    }

    let paths = args
        .iter()
        .skip(3)
        .map(|s| s.clone())
        .collect::<Vec<String>>();

    if paths.len() == 0 || paths.len() > 5 {
        exit(0)
    }

    let mut img = match image::open(&paths[0]) {
        Ok(im) => im,
        Err(_) => {
            println!("Invalid file path.");
            exit(0);
        }
    };

    let mut dims = img.dimensions();
    let ratio: f64 = dims.1 as f64 / dims.0 as f64;

    let width;
    let height;

    if dims.0 >= dims.1 {
        width = if dims.0 < 800 {
            dims.0
        }

        else {800};
        height = (width as f64 * ratio) as u32;
    }
    else {
        height = if dims.0 < 800 {
            dims.0
        }

        else {800};
        width = (height as f64 / ratio) as u32;
    }

    img = img.thumbnail(width, height);
    dims = img.dimensions();

    let mut bytes: Vec<Vec<u8>> = vec![img.to_rgb().to_vec()];

    if paths.len() > 1 {
        for i in 1..paths.len()
        {
            let im = match image::open(&paths[i]) {
                Ok(im) => im,
                Err(_) => {
                    println!("Invalid file path.");
                    exit(0);
                }
            };

            let imbuff = image::imageops::resize(&im, dims.0, dims.1, image::imageops::FilterType::Nearest);
            let mut byts: Vec<u8> = vec![];

            for pixel in imbuff.pixels() {
                byts.push(pixel[0]);
                byts.push(pixel[1]);
                byts.push(pixel[2]);
            }

            bytes.push(byts);
        }
    }

    let mut res_paths: Vec<String> = vec![];

    loop {
        let mut n = 1;
        let mut frames: Vec<Frame> = vec![];
        let mode = modes[modes.len() - 1];
        let mut bn = 0;

        let double_first = if mode == 3 {
            true
        } else { false };

        loop {
            let mut byts = bytes[bn].clone();
            let mut exit_early = false;

            match mode {
                // Glitch
                1 => {
                    match n {
                        1 => {
                            remove_first_byte(&mut byts);
                        },
                        2 => {
                            for _ in 0..get_num_mutations() {
                                remove_random_byte(&mut byts);
                            }
                        },
                        3 => {
                            remove_first_byte(&mut byts);
                            remove_first_byte(&mut byts);
                        },
                        4 => {
                            for _ in 0..get_num_mutations() {
                                remove_random_byte(&mut byts);
                            }
                        },
                        _ => {}
                    }
                },
                // Wave
                2 => {
                    modify_bytes(&mut byts, dims.0, dims.1, false);
                    modify_bytes(&mut byts, dims.0, dims.1, true);
                    modify_bytes(&mut byts, dims.0, dims.1, false);
                    modify_bytes(&mut byts, dims.0, dims.1, false);
                    modify_bytes(&mut byts, dims.0, dims.1, false);
                    modify_bytes(&mut byts, dims.0, dims.1, false);
                    modify_bytes(&mut byts, dims.0, dims.1, false);
                },
                // Mirror
                3 => {
                    match n {
                        2 => {
                            byts = line_reverse(byts, dims.0, "left");
                        },
                        3 => {
                            byts = line_reverse(byts, dims.0, "full");
                        },
                        4 => {
                            byts = line_reverse(byts, dims.0, "right");
                        }
                        _ => {}
                    }
                },
                // Static
                4 => {
                    make_static(&mut byts);

                    if n >= 2 {
                        if n >= paths.len() {
                            exit_early = true;
                        }
                    }
                },
                // Glow
                5 => {
                    if n == 2 {
                        byts = reverse_group(byts, false);
                    }
                    else if n == 4 {
                        byts = reverse_group(byts, true);
                    }
                },
                // Glass
                6 => {
                    match n {
                        1 => {
                            byts = swap_group(byts, 1, dims.0);
                        },
                        2 => {
                            byts = swap_group(byts, 2, dims.0);
                        },
                        3 => {
                            byts = swap_group(byts, 3, dims.0);
                        },
                        4 => {
                            byts = swap_group(byts, 2, dims.0);
                        },
                        _ => {}
                    }
                },
                // Color
                7 => {
                    if n % 2 == 0 {
                        colorize(&mut byts);
                    }
                },
                // Chalk
                8 => {
                    if n % 2 == 0 {
                        decolorize(&mut byts);

                        if n >= paths.len() {
                            exit_early = true;
                        }
                    }
                }
                _ => {}
            }

            let d = if n == 1 && double_first {
                delay * 2
            }
            else {delay};

            let mut frame = gif::Frame::from_rgb_speed(dims.0 as u16, dims.1 as u16, &mut byts, 30);
            frame.delay = d;
            frames.push(frame);

            n += 1;
            if exit_early || n > 4 {break}
            bn += 1;

            if bn >= bytes.len() {
                bn = 0;
            }
        }

        let color_map = &[0xFF, 0xFF, 0xFF, 0, 0, 0];
        let new_path = format!("mutated/{}_{}.gif", random_word(), now());
        let mut image = File::create(&new_path).unwrap();
        let mut encoder = Encoder::new(&mut image, dims.0 as u16, dims.1 as u16, color_map).unwrap();
        encoder.set(Repeat::Infinite).unwrap();

        for frame in frames {
            encoder.write_frame(&frame).unwrap();
        }

        res_paths.push(new_path);
        modes.pop();

        if modes.len() == 0 {
            break;
        }
    }

    let mut s = "".to_string();

    // There is no join in rust yet
    // without an external crate
    for path in res_paths.iter().rev() {
        s = format!("{} {}", s, path);
    }

    println!("{}", s.trim());
}

fn now() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_millis()
}

fn get_num_mutations() -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(1, 20)
}

fn remove_random_byte(bytes: &mut Vec<u8>) {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0, bytes.len() - 1);
    let og = bytes[index];
    bytes.remove(index);
    bytes.push(og)
}

fn remove_first_byte(bytes: &mut Vec<u8>) {
    let og = bytes[0];
    bytes.remove(0);
    bytes.push(og)
}

fn modify_bytes(bytes: &mut Vec<u8>, width: u32, height: u32, start: bool) {
    let mut rng = rand::thread_rng();
    let row = if start {0} else {rng.gen_range(0, height)};

    let range = rng.gen_range((height as f64 * 0.1).round() as u32,
        (height as f64 * 0.6).round() as u32);

    let mut i = if row == 0 {0} else {
        ((row * width * 3) - 1) as usize
    };

    let mode = rng.gen_range(1, 4);

    for _ in 0..range {
        for _ in 0..width {
            let r = bytes[i];
            let g = bytes[i + 1];
            let b = bytes[i + 2];

            match mode {
                1 => {
                    bytes[i + 1] = 255 - r;
                    bytes[i + 2] = 255 - g;
                    bytes[i + 3] = 255 - b;
                },
                2 => {
                    bytes[i + 1] = 255 - b;
                    bytes[i + 2] = 255 - r;
                    bytes[i + 3] = 255 - g;
                },
                3 => {
                    bytes[i + 1] = 255 - g;
                    bytes[i + 2] = 255 - b;
                    bytes[i + 3] = 255 - r;
                },
                _ => {}
            }

            i += 3;
        }

        if i >= bytes.len() - 1 {
            break;
        }
    }
}

fn line_reverse(bytes: Vec<u8>, width: u32, mode: &str) -> Vec<u8> {
    if mode == "full" {
        return bytes.into_iter().rev().collect();
    }
    else {
        let mut n = 0;
        let mut nbytes: Vec<u8> =vec![];
        let mut line: Vec<u8> = vec![];

        for byte in bytes.iter() {
            line.push(*byte);
            n += 1;

            if n >= width {
                for b in line.into_iter().rev() {
                    nbytes.push(b)
                }

                n = 0;
                line = vec![];
            }
        }

        if mode == "right" {
            return nbytes.into_iter().rev().collect();
        }
        else {
            return nbytes;
        }
    }
}

fn make_static(bytes: &mut Vec<u8>) {
    let amount = (bytes.len() as f64 * 0.25).round() as u32;
    let mut rng = rand::thread_rng();

    for _ in 0..amount {
        let index = rng.gen_range(0, bytes.len() - 1);
        bytes[index] = rng.gen_range(0, 255);
    }
}

fn reverse_group(bytes: Vec<u8>, alt: bool) -> Vec<u8> {
    let mut nbytes: Vec<u8> = vec![];
    let mut temp: Vec<u8> = vec![];
    let mut n = 0;

    for byte in bytes.iter() {
        temp.push(*byte);
        n += 1;

        if n == 3 {
            if alt {
                nbytes.push(temp[1]);
                nbytes.push(temp[2]);
                nbytes.push(temp[0]);
            }
            else {
                nbytes.push(temp[2]);
                nbytes.push(temp[1]);
                nbytes.push(temp[0]);
            }

            temp = vec![];
            n = 0
        }
    }

    for b in temp.iter().rev() {
        nbytes.push(*b);
    }

    return nbytes;
}

fn swap_group(bytes: Vec<u8>, level: u32, width: u32) -> Vec<u8> {
    let mut nbytes: Vec<u8> = vec![];
    let mut temp: Vec<u8> = vec![];
    let mut temp2: Vec<u8> = vec![];
    let mut n = 0;

    let mut limit = ((width as f64) * (level as f64 * 0.008)) as u32;

    while limit > 0 && limit % 3 != 0 {
        limit -= 1;
    }

    if limit < 3 * level {
        limit = 3 * level;
    }

    for byte in bytes.iter() {
        n += 1;

        if n <= limit {
            temp.push(*byte);
        }
        else if n <= limit * 2 {
            temp2.push(*byte);
        }

        if n == limit * 2 {
            for b in temp2.iter() {
                nbytes.push(*b);
            }

            for b in temp.iter() {
                nbytes.push(*b);
            }

            temp = vec![];
            temp2 = vec![];
            n = 0;
        }
    }

    if temp2.len() > 0 {
        for b in temp2.iter() {
            nbytes.push(*b)
        }
    }

    if temp.len() > 0 {
        for b in temp.iter() {
            nbytes.push(*b)
        }
    }

    return nbytes;
}

fn colorize(bytes: &mut Vec<u8>) {
    let mut n = 0;

    let mut rng = rand::thread_rng();

    let w1 = rng.gen_range(128, 255);
    let w2 = rng.gen_range(128, 255);
    let w3 = rng.gen_range(128, 255);
    let b1 = rng.gen_range(0, 128);
    let b2 = rng.gen_range(0, 128);
    let b3 = rng.gen_range(0, 128);

    let mut line: Vec<u8> = vec![];

    for i in 0..bytes.len() {
        n += 1;

        line.push(bytes[i]);

        if n == 3 {
            let d1 = (line[0] as i32 - line[1] as i32).abs();
            let d2 = (line[0] as i32 - line[2] as i32).abs();
            let d3 = (line[1] as i32 - line[2] as i32).abs();

            if d1 < 20 && d2 < 20 && d3 < 20 {
                if line[0] >= 128 {
                    bytes[i - 2] = w1;
                    bytes[i - 1] = w2;
                    bytes[i - 0] = w3;
                } else {
                    bytes[i - 2] = b1;
                    bytes[i - 1] = b2;
                    bytes[i - 0] = b3;
                }
            }

            line = vec![];
            n = 0;
        }
    }
}

fn decolorize(bytes: &mut Vec<u8>) {
    let mut n = 0;
    let mut line: Vec<u8> = vec![];

    for i in 0..bytes.len() {
        n += 1;

        line.push(bytes[i]);

        if n == 3 {
            let d1 = (line[0] as i32 - line[1] as i32).abs();
            let d2 = (line[0] as i32 - line[2] as i32).abs();
            let d3 = (line[1] as i32 - line[2] as i32).abs();

            if d1 > 20 || d2 > 20 || d3 > 20 {
                if line[0] >= 128 {
                    bytes[i - 2] = 255;
                    bytes[i - 1] = 255;
                    bytes[i - 0] = 255;
                }
                else {
                    bytes[i - 2] = 0;
                    bytes[i - 1] = 0;
                    bytes[i - 0] = 0;
                }
            }

            line = vec![];
            n = 0;
        }
    }
}

fn random_word() -> String {
    let a = vec!["a","e","i","o","u"];
    let b = vec!["b", "c", "d", "f", "g", "h", "j", "k", "l", "m",
    "n", "p", "r", "s", "t", "v", "w", "x", "y", "z"];
    let mut word = "".to_string();

    for i in 1..=6 {
        let letter = if i % 2 == 0 {
            b.choose(&mut rand::thread_rng())
        }
        else {
            a.choose(&mut rand::thread_rng())
        };

        word.push_str(letter.unwrap());
    }

    return word;
}