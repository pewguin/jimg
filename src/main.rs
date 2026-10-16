pub mod filters;
pub mod script;

use std::{error::Error, fs::File, path::Path};

use image::{Delay, DynamicImage, Frame, ImageReader, Rgba32FImage, codecs::gif::{GifEncoder, Repeat}};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};
use clap::Parser;

use crate::filters::{filter::Filter, remap_filter::{RawResizeFilter, Remap}};

const GIF_FPS: u32 = 15;


fn load_image(path: &str) -> Result<DynamicImage, Box<dyn Error + 'static>> {
    Ok(ImageReader::open(path)?.decode()?)
}

fn bar(len: u64, msg: &'static str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template("{msg:>10} [{bar:40}] {pos}/{len} eta {eta}")
        .unwrap()
        .progress_chars("=> ")
    );
    pb.set_message(msg);
    pb
}

fn save_gif(frames: Vec<Rgba32FImage>, path: &Path) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut encoder = GifEncoder::new_with_speed(file, 15);
    encoder.set_repeat(Repeat::Infinite)?;

    let delay = Delay::from_numer_denom_ms(1000, GIF_FPS);

    let largest = frames.iter().fold((0_u32, 0_u32), |l, f| {
        (l.0.max(f.width()), l.1.max(f.height()))
    });

    let filter = Remap(RawResizeFilter::new(largest));
    let frames = frames.into_iter().map(|f| {
        let fimg = filter.apply(&f);
        DynamicImage::to_rgba8(&DynamicImage::ImageRgba32F(fimg))
    });

    let n = frames.len() as u64;
    encoder.encode_frames(frames.into_iter().progress_with(bar(n, "encoding")).map(|f| {
        Frame::from_parts(f, 0, 0, delay)
    }))?;

    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: String,
    #[arg(short, long, default_value_t = "output.gif".to_string())]
    output: String,
    #[arg(short, long, default_value_t = "filter.rhai".to_string())]
    file: String,
    #[arg(short, long, default_value_t = 1000)]
    time: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let img = DynamicImage::to_rgba32f(&load_image(args.input.as_ref())?);

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

    save_gif(frames, output)?;
    Ok(())
}
