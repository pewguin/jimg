pub mod filters;
pub mod gui;
pub mod image;
pub mod script;

use std::{error::Error, path::Path};

use ::image::Rgba32FImage;
use clap::Parser;
use indicatif::ProgressIterator;

use crate::image::{bar, load_image, save_gif};

const GIF_FPS: u32 = 15;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,
    #[arg(short, long, default_value_t = "output.gif".to_string())]
    output: String,
    #[arg(short, long, default_value_t = "filter.rhai".to_string())]
    file: String,
    #[arg(short, long, default_value_t = 1000)]
    time: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let Some(input) = args.input else {
        if std::env::args_os().len() == 1 {
            return gui::run().map_err(Into::into);
        }
        return Err("the --input option is required when using CLI mode".into());
    };
    let img = load_image(&input)?;

    let frame_count = (args.time as f32 / GIF_FPS as f32).round().max(1.0) as u32;

    let output = Path::new(&args.output);
    let file = Path::new(&args.file);

    let program = script::Program::from_file(file)?;

    let frames: Vec<Rgba32FImage> = (0..frame_count)
        .progress_with(bar(frame_count as u64, "rendering"))
        .map(|i| {
            let t = i as f32 / frame_count as f32;
            program.evaluate(&img, t)
        })
        .collect::<Result<_, _>>()?;

    save_gif(frames, output, GIF_FPS)?;
    Ok(())
}
