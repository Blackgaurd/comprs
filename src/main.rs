use std::{
    env,
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use ::image::{codecs::gif::GifEncoder, Frame};
use image::{ImageData, RGB};
use tree::Tree;

mod image;
mod psa;
mod tree;

fn print_usage(program: &String) {
    println!(
        "usage: {} <input-file> [-o output-file] -iter <iterations> [-outline hex-code] [-gif save-delta]",
        program
    );
}

fn file_without_extension(path: &String) -> Result<(String, String), String> {
    let file_path = Path::new(path);
    if let Some(stem) = file_path.file_stem() {
        let new_path: PathBuf = match file_path.parent() {
            Some(parent) => parent.join(stem),
            None => PathBuf::from(stem),
        };
        if let Some(new_path_str) = new_path.to_str() {
            if let Some(extension) = file_path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    Ok((new_path_str.into(), ext_str.into()))
                } else {
                    Err("failed to convert extension to string".into())
                }
            } else {
                Ok((new_path_str.into(), String::new()))
            }
        } else {
            Err("failed to convert file path to string".into())
        }
    } else {
        Err("failed to get file stem".into())
    }
}

fn hex_to_rgb(hex: &String) -> Result<RGB<u8>, String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Err("hex code must be 6 characters long".into());
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "invalid hex code")?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "invalid hex code")?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "invalid hex code")?;

    Ok(RGB::new(r, g, b))
}

fn real_main() -> i32 {
    let mut input_file = None;
    let mut output_file = None;
    let mut iterations: u32 = 0;
    let mut outline = None;
    let mut gif_delta: Option<u32> = None;

    let mut args = env::args();
    let Some(program_name) = args.next() else {
        return 1;
    };
    while let Some(arg) = args.next() {
        if arg == "-h" {
            print_usage(&program_name);
            return 0;
        } else if arg == "-o" {
            if let Some(of) = args.next() {
                output_file = Some(of);
            } else {
                println!("output file not specified");
                print_usage(&program_name);
                return 1;
            }
        } else if arg == "-iter" {
            if let Some(i_str) = args.next() {
                iterations = match i_str.parse() {
                    Ok(iters) => iters,
                    Err(_) => {
                        println!("invalid number of iterations");
                        print_usage(&program_name);
                        return 1;
                    }
                }
            } else {
                println!("number of iterations not specified");
                print_usage(&program_name);
                return 1;
            }
        } else if arg == "-outline" {
            if let Some(h_str) = args.next() {
                outline = match hex_to_rgb(&h_str) {
                    Ok(rgb) => Some(rgb),
                    Err(err) => {
                        println!("{err}");
                        return 1;
                    }
                }
            } else {
                println!("outline hex code not specified");
                print_usage(&program_name);
                return 1;
            }
        } else if arg == "-gif" {
            if let Some(g_str) = args.next() {
                gif_delta = match g_str.parse() {
                    Ok(delta) => Some(delta),
                    Err(_) => {
                        println!("invalid gif save delta");
                        print_usage(&program_name);
                        return 1;
                    }
                }
            } else {
                println!("gif save delta not specified");
                print_usage(&program_name);
                return 1;
            }
        } else {
            input_file = Some(arg);
        }
    }

    let input_file = match input_file {
        Some(in_s) => in_s,
        None => {
            println!("no input file given");
            print_usage(&program_name);
            return 1;
        }
    };

    let output_file = match output_file {
        Some(out_s) => out_s,
        None => match file_without_extension(&input_file) {
            Ok((stem, extension)) => {
                if let Some(_) = gif_delta {
                    format!("{stem}-comprs.gif")
                } else {
                    format!("{stem}-comprs.{extension}")
                }
            }
            Err(err) => {
                println!("{err}");
                return 1;
            }
        },
    };

    let data = match ImageData::from_path(&input_file) {
        Ok(d) => d,
        Err(err) => {
            println!("{err}");
            return 1;
        }
    };

    let mut tree = Tree::new(data);
    match gif_delta {
        Some(delta) => {
            let mut frames = Vec::new();
            let buf = tree.render_rgba(outline);
            frames.push(Frame::new(buf));
            for i in 1..=iterations {
                if let Err(err) = tree.refine() {
                    println!("{err}");
                    return 1;
                }
                if i % delta == 0 {
                    let buf = tree.render_rgba(outline);
                    frames.push(Frame::new(buf));
                }
            }

            println!("encoding gif...");
            let Ok(file) = File::create(output_file) else {
                println!("unable to create new file");
                return 1;
            };
            let writer = BufWriter::new(file);
            let mut encoder = GifEncoder::new_with_speed(writer, 30);
            if let Err(_) = encoder.encode_frames(frames) {
                println!("error in encoding gif");
                return 1;
            }
        }
        None => {
            for _ in 1..=iterations {
                if let Err(err) = tree.refine() {
                    println!("{err}");
                    return 1;
                }
            }
            if let Err(err) = tree.render_rgb(outline).save(output_file) {
                println!("{err}");
                return 1;
            }
        }
    }

    0
}

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code);
}
