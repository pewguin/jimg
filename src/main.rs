pub mod filters;
pub mod renderer;
pub mod script;
pub mod timeline;

use std::{error::Error, fs::File, io};

use image::{Delay, DynamicImage, Frame, ImageReader, RgbaImage, codecs::gif::{GifEncoder, Repeat}};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

use crate::{filters::filter::{ImgOp, RawResizeFilter, apply}, timeline::{Source, timeline}};

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

fn save_gif(frames: Vec<RgbaImage>, path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut encoder = GifEncoder::new_with_speed(file, 15);
    encoder.set_repeat(Repeat::Infinite)?;

    let delay = Delay::from_numer_denom_ms(1000, GIF_FPS);

    let largest = frames.iter().fold((0_u32, 0_u32), |l, f| {
        (l.0.max(f.width()), l.1.max(f.height()))
    });

    let frames = frames.into_iter().map(|f| {
        apply(&DynamicImage::ImageRgba8(f), ImgOp::from(RawResizeFilter::new(largest)))
    });

    let n = frames.len() as u64;
    encoder.encode_frames(frames.into_iter().progress_with(bar(n, "encoding")).map(|f| {
        Frame::from_parts(f, 0, 0, delay)
    }))?;

    Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = timeline()?;
    let img = load_image(args.input.as_deref().unwrap_or("img.png"))?;
    let length = args.time.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "specify gif length with -t")
    })?;

    let frame_count = (length as f32 / GIF_FPS as f32).round().max(1.0) as u32;
    let program = match &args.source {
        Source::Script(path) => Some(script::Program::from_file(path)?),
        Source::Inline(_) => None,
    };
    let frames: Vec<RgbaImage> = (0..frame_count)
        .progress_with(bar(frame_count as u64, "rendering"))
        .map(|i| {
            let t = i as f32 / frame_count as f32;
            match (&args.source, &program) {
                (Source::Script(_), Some(program)) => program.evaluate(&img, t),
                (Source::Inline(effects), None) => {
                    Ok(renderer::apply_all(&img, effects, t))
                }
                _ => unreachable!("script program must match the selected source"),
            }
        })
        .collect::<Result<_, _>>()?;

    save_gif(frames, args.output.as_deref().unwrap_or("output.gif"))?;
    Ok(())
}
