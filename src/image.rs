use std::{error::Error, fs::File, path::Path};

use ::image::{
    Delay, DynamicImage, Frame, ImageReader, Rgba32FImage,
    codecs::gif::{GifEncoder, Repeat},
};
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

use crate::filters::{
    filter::Filter,
    remap_filter::{RawResizeFilter, Remap},
};

pub fn load_image(path: &str) -> Result<Rgba32FImage, Box<dyn Error>> {
    Ok(ImageReader::open(path)?.decode()?.to_rgba32f())
}

pub(crate) fn bar(len: u64, msg: &'static str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template("{msg:>10} [{bar:40}] {pos}/{len} eta {eta}")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.set_message(msg);
    pb
}

pub fn save_gif(frames: Vec<Rgba32FImage>, path: &Path, fps: u32) -> Result<(), Box<dyn Error>> {
    if fps == 0 {
        return Err("GIF frame rate must be greater than zero".into());
    }

    let file = File::create(path)?;
    let mut encoder = GifEncoder::new_with_speed(file, 15);
    encoder.set_repeat(Repeat::Infinite)?;

    let delay = Delay::from_numer_denom_ms(1000, fps);
    let largest = frames.iter().fold((0_u32, 0_u32), |l, f| {
        (l.0.max(f.width()), l.1.max(f.height()))
    });
    let filter = Remap(RawResizeFilter::new(largest));
    let frames = frames.into_iter().map(|frame| {
        let frame = filter.apply(&frame);
        DynamicImage::ImageRgba32F(frame).to_rgba8()
    });

    let frame_count = frames.len() as u64;
    encoder.encode_frames(
        frames
            .progress_with(bar(frame_count, "encoding"))
            .map(|frame| Frame::from_parts(frame, 0, 0, delay)),
    )?;

    Ok(())
}
