pub mod filters;
pub mod renderer;
pub mod timeline;

use std::{error::Error, fs::File};

use image::{Delay, DynamicImage, Frame, ImageReader, RgbaImage, codecs::gif::{GifEncoder, Repeat}};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

use crate::{filters::filter::{ImgOp, RawResizeFilter, apply}, timeline::timeline};

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


fn main() {
    let timeline = match timeline() {
        Ok(tl) => tl,
        Err(err) => {
            panic!("{:#?}", err);
        },
    };

    println!("tl: {:?}", timeline);
    let img = load_image(&timeline.input.unwrap_or("img.png".to_string())).unwrap();

    let frames = (timeline.length as f32 / GIF_FPS as f32).round() as u32;
    let frames: Vec<RgbaImage> = (0..frames)
        .progress_with(bar(frames as u64, "rendering"))
        .map(|i| renderer::apply_all(&img, &timeline.effects, i as f32 / frames as f32))
        .collect();

    save_gif(frames, &timeline.output.unwrap_or("output.gif".to_owned())).unwrap();
}
