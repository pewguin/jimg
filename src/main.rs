use std::{env, error::Error};

use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, RgbImage, Rgba, RgbaImage};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    oper: String,
    modifier: f32,

    #[arg(short, long)]
    input: String,

    #[arg(short, long, default_value_t = "output.png".to_string())]
    output: String,
}

fn main() {
    let args = Args::parse();

    let img = load_image(&args.input).unwrap();

    match args.oper.as_ref() {
        "squish" => { squish_image(&img, args.modifier) }
        "slant" => { slant_image(&img, args.modifier) }
        _ => { panic!("Not an operation") }
    }.save(args.output).unwrap();
}

fn load_image(path: &str) -> Result<DynamicImage, Box<dyn Error + 'static>> {
    Ok(ImageReader::open(path)?.decode()?)
}

fn squish_image(src: &DynamicImage, amount: f32) -> RgbaImage {
    let mut new_img: RgbaImage = ImageBuffer::new((src.width() as f32 / amount).round() as u32, src.height());

    for (x, y, pixel) in new_img.enumerate_pixels_mut() {
        let px = src.get_pixel((x as f32 * amount).round() as u32, y);
        *pixel = px;
    }
    new_img
}

fn slant_image(src: &DynamicImage, angle: f32) -> RgbaImage {
    let alpha = angle.to_radians();
    let w = src.width();
    let h = src.height();
    let m = (-alpha + std::f32::consts::PI / 2.0).tan();
 
    let slant = |x: u32, y: u32| {
        (y as f32 / m + x as f32).round() as u32
    };

    let mut new_img: RgbaImage = ImageBuffer::new(slant(w, h), h);

    for (x, y, pixel) in src.pixels() {
        new_img.put_pixel(slant(x, y), y, pixel);
    }

    new_img
}
