use std::{borrow::Cow, error::Error, fs::File};

use glam::{Mat2, Vec2};
use image::{Delay, DynamicImage, Frame, GenericImageView, ImageBuffer, ImageReader, RgbaImage, codecs::gif::{GifEncoder, Repeat}};
use clap::{Parser, Subcommand};

const GIF_FPS: u32 = 15;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: String,

    #[arg(short, long, global = true)]
    output: Option<String>,

    #[arg(short, long)]
    duration: Option<f32>,

    #[arg(required = true, num_args = 1.., allow_hyphen_values = true)]
    ops: Vec<String>,
}

#[derive(Parser)]
#[command(no_binary_name = true)]
struct Step {
    #[command(subcommand)]
    op: Op,
}

#[derive(Subcommand, Clone)]
enum Op {
    Squish { amount: f32 },
    Slant {
        #[arg(value_parser = validate_angle)]
        angle: f32
    },
    FlipHorizontal,
    FlipVertical,
    Rotate { angle: f32 },
}

fn lerp (a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t 
}

impl Op {
    /// Op at t=0 should be no effect, t=1 full effect
    pub fn at(&self, t: f32) -> Op {
        match self {
            Op::Squish { amount } => Op::Squish { amount: lerp(1.0, *amount, t) },
            Op::Slant { angle } => Op::Slant { angle: angle * t },
            Op::FlipHorizontal => Op::FlipHorizontal,
            Op::FlipVertical => Op::FlipVertical,
            Op::Rotate { angle } => Op::Rotate { angle: angle * t }
        }
    }
}

fn validate_angle(s: &str) -> Result<f32, String> {
    let val: f32 = s
        .parse()
        .map_err(|_| format!("{} isn't a valid floating-point number", s))?;

    if (0.0..=70.0).contains(&val) {
        Ok(val)
    } else {
        Err(String::from("value must be between 0.0 and 70.0"))
    }
}

fn squish(src: &DynamicImage, amount: f32) -> RgbaImage {
    let mut dst: RgbaImage = ImageBuffer::new((src.width() as f32 / amount).round() as u32, src.height());

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        let px = src.get_pixel((x as f32 * amount).round() as u32, y);
        *pixel = px;
    }
    dst
}

fn slant(src: &DynamicImage, angle: f32) -> RgbaImage {
    let alpha = angle.to_radians();
    let (w, h) = src.dimensions();
    let m = (-alpha + std::f32::consts::PI / 2.0).tan();
 
    let slant = |x: u32, y: u32| {
        (y as f32 / m + x as f32).round() as u32
    };

    let mut dst: RgbaImage = ImageBuffer::new(slant(w, h), h);

    for (x, y, pixel) in src.pixels() {
        dst.put_pixel(slant(x, y), y, pixel);
    }

    dst
}

fn flip_horizontal(src: &DynamicImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut dst: RgbaImage = ImageBuffer::new(w, h);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = src.get_pixel(w - x - 1, y);
    }

    dst
}

fn flip_vertical(src: &DynamicImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut dst: RgbaImage = ImageBuffer::new(w, h);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = src.get_pixel(x, h - y - 1);
    }

    dst
}

fn rotate(src: &DynamicImage, angle: f32) -> RgbaImage {
    let (w, h) = src.dimensions();

    let diag = ((w * w + h * h) as f32).sqrt().ceil() as u32;
    let src_center = Vec2::new(w as f32, h as f32) / 2.0;
    let dst_center = Vec2::splat(diag as f32 / 2.0);

    let mut dst: RgbaImage = ImageBuffer::new(diag, diag);

    let inv_rot = Mat2::from_angle(angle.to_radians()).inverse();

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        let dst_offset = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - dst_center;
        let src_offset = inv_rot * dst_offset + src_center;

        let (s_x, s_y) = src_offset.floor().into();

        if s_x >= 0.0 && s_y >= 0.0 && s_x < w as f32 && s_y < h as f32 {
            *pixel = src.get_pixel(s_x as u32, s_y as u32);
        }
    }

    dst
}

fn load_image(path: &str) -> Result<DynamicImage, Box<dyn Error + 'static>> {
    Ok(ImageReader::open(path)?.decode()?)
}

fn save_gif(frames: Vec<RgbaImage>, path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut encoder = GifEncoder::new(file);
    encoder.set_repeat(Repeat::Infinite)?;

    let delay = Delay::from_numer_denom_ms((1.0 / GIF_FPS as f32).round() as u32, 1);

    encoder.encode_frames(frames.into_iter().map(|f| {
        Frame::from_parts(f, 0, 0, delay)
    }))?;

    Ok(())
}

fn apply(img: &DynamicImage, op: Op) -> RgbaImage {
    match op {
        Op::Squish { amount } => squish(&img, amount),
        Op::Slant { angle } => slant(&img, angle),
        Op::FlipHorizontal => flip_horizontal(&img),
        Op::FlipVertical => flip_vertical(&img),
        Op::Rotate { angle } => rotate(&img, angle),
    }
}

fn apply_all(img: &DynamicImage, ops: &[Op]) -> RgbaImage {
    ops.into_iter().fold(Cow::Borrowed(img), |acc, op| {
        Cow::Owned(DynamicImage::ImageRgba8(apply(&acc, op.clone())))
    }).to_rgba8()
}

fn apply_all_at(img: &DynamicImage, ops: &[Op], t: f32) -> RgbaImage {
    ops.into_iter().fold(Cow::Borrowed(img), |acc, op| {
        Cow::Owned(DynamicImage::ImageRgba8(apply(&acc, op.at(t))))
    }).to_rgba8()
}

fn main() {
    let args = Args::parse();

    let steps: Vec<Step> = args.ops
        .split(|a| a == "+")
        .map(|chunk| Step::try_parse_from(chunk).unwrap_or_else(|e| e.exit()))
        .collect();

    let ops: Vec<Op> = steps.into_iter().map(|s| s.op).collect();

    let img = load_image(&args.input).unwrap();

    if let Some(d) = args.duration {
        let frames = (d / (GIF_FPS as f32)).round() as u32;
        let frames: Vec<RgbaImage> = (0..frames)
            .map(|i| apply_all_at(&img, &ops, i as f32 / frames as f32))
            .collect();

        save_gif(frames, &args.output.unwrap_or("output.gif".to_owned())).unwrap();
    } else {
        apply_all(&img, &ops).save(args.output.unwrap_or("output.png".to_owned())).unwrap();
    }

}
