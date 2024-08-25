use std::{
    env,
    fs::File,
    path::{Path, PathBuf},
};

use ::image::{codecs::gif::GifEncoder, Frame};
use image::ImageData;
use indicatif::ProgressBar;
use tree::Tree;

mod image;
mod psa;
mod tree;

fn print_usage(program: &String) {
    println!(
        "usage: {} <input-file> [-o output-file] -iter <iterations> [-gif save-delta]",
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

fn real_main() -> i32 {
    let mut input_file = None;
    let mut output_file = None;
    let mut iterations: u32 = 0;
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
            Ok((stem, extension)) => format!("{stem}-comprs.{extension}"),
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
    let progress_bar = ProgressBar::new(iterations.into());
    match gif_delta {
        Some(delta) => {
            let Ok(writer) = File::create(output_file) else {
                println!("unable to create new file");
                return 1;
            };
            let mut encoder = GifEncoder::new(writer);

            let Ok(()) = encoder.encode_frame(Frame::new(tree.render_rgba())) else {
                println!("error in encoding gif");
                return 1;
            };
            dbg!(delta);
            for i in 1..=iterations {
                if let Err(err) = tree.refine() {
                    println!("{err}");
                    return 1;
                }
                if i % delta == 0 {
                    let buf = tree.render_rgba();
                    let Ok(()) = encoder.encode_frame(Frame::new(buf)) else {
                        println!("error in encoding gif");
                        return 1;
                    };
                }
                progress_bar.tick();
            }
            progress_bar.finish();
        }
        None => {
            for _ in 1..=iterations {
                if let Err(err) = tree.refine() {
                    println!("{err}");
                    return 1;
                }
                progress_bar.tick();
            }
            progress_bar.finish();
            if let Err(err) = tree.render_rgb().save(output_file) {
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
