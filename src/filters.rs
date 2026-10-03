use glam::{Mat2, Vec2};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

fn get_pixel_or(src: &DynamicImage, x: u32, y: u32, or: &Rgba<u8>) -> Rgba<u8> {
    let (sx, sy) = src.dimensions();

    if x < sx && y < sy {
        src.get_pixel(x, y)
    } else {
        or.clone()
    }
}

pub fn squish(src: &DynamicImage, amount: f32) -> RgbaImage {
    let mut dst: RgbaImage = ImageBuffer::new((src.width() as f32 / amount).round() as u32, src.height());

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        let px = src.get_pixel((x as f32 * amount).round() as u32, y);
        *pixel = px;
    }
    dst
}

pub fn slant(src: &DynamicImage, angle: f32) -> RgbaImage {
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

pub fn flip_horizontal(src: &DynamicImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut dst: RgbaImage = ImageBuffer::new(w, h);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = src.get_pixel(w - x - 1, y);
    }

    dst
}

pub fn flip_vertical(src: &DynamicImage) -> RgbaImage {
    let (w, h) = src.dimensions();
    let mut dst: RgbaImage = ImageBuffer::new(w, h);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = src.get_pixel(x, h - y - 1);
    }

    dst
}

pub fn rotate(src: &DynamicImage, angle: f32) -> RgbaImage {
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

pub fn raw_resize(src: &DynamicImage, dim: (u32, u32)) -> RgbaImage {
    let mut dst: RgbaImage = ImageBuffer::new(dim.0, dim.1);
    let empty = Rgba::from([0, 0, 0, 0]);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = get_pixel_or(src, x, y, &empty);
    }

    dst
}

pub fn resize(src: &DynamicImage, dim: (u32, u32)) -> RgbaImage {
    let (dw, dh) = dim;
    let mut dst: RgbaImage = ImageBuffer::new(dw, dh);
    let (sw, sh) = src.dimensions();
    let xf = sw as f32 / dw as f32;
    let yf = sh as f32 / dh as f32;

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        let sx = (x as f32 * xf).round() as u32;
        let sy = (y as f32 * yf).round() as u32;
        *pixel = src.get_pixel(sx, sy);
    }

    dst
}
