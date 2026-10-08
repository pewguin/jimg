use glam::{Mat2, Vec2};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

fn to_vec(value: (u32, u32)) -> Vec2 {
    Vec2::new(value.0 as f32, value.1 as f32)
}

fn vec_to_size(v: Vec2) -> (u32, u32) {
    (v.x.round() as u32, v.y.round() as u32)
}

fn add_tuple(lhs: (u32, u32), rhs: (u32, u32)) -> (u32, u32) {
    return (lhs.0 + rhs.0, lhs.1 + rhs.1)
}

fn get_pixel_or(src: &DynamicImage, x: u32, y: u32, or: &Rgba<u8>) -> Rgba<u8> {
    let (sx, sy) = src.dimensions();

    if x > 0 && x < sx && y > 0 && y < sy {
        src.get_pixel(x, y)
    } else {
        or.clone()
    }
}

fn apply_filter<F>(src: &DynamicImage, size: (u32, u32), get_from: F) -> RgbaImage
where F: Fn(Vec2) -> Vec2 {
    let default_color = Rgba::from([0, 0, 0, 0]);
    let mut dst: RgbaImage = ImageBuffer::new(size.0, size.1);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        let dst_vec = to_vec((x, y));
        let src_vec = get_from(dst_vec);

        let s_pos = (src_vec.x.floor() as u32, src_vec.y.floor() as u32);

        *pixel = get_pixel_or(src, s_pos.0, s_pos.1, &default_color);
    }

    dst
}

fn matrix_filter(src: &DynamicImage, mat: Mat2) -> RgbaImage {
    // need for size of transformed image
    let ssize = to_vec(src.dimensions());
    let sc = ssize * 0.5;

    let corners = [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]
        .map(|c| Vec2::new(c.x * ssize.x, c.y * ssize.y))
        .map(|c| mat * c);

    let minx = corners.iter().map(|c| c.x).reduce(|p, t| p.min(t)).unwrap();
    let maxx = corners.iter().map(|c| c.x).reduce(|p, t| p.max(t)).unwrap();
    let miny = corners.iter().map(|c| c.y).reduce(|p, t| p.min(t)).unwrap();
    let maxy = corners.iter().map(|c| c.y).reduce(|p, t| p.max(t)).unwrap();

    // Bounding corners of new image in source image space
    let tr = Vec2::new(maxx, maxy);
    let bl = Vec2::new(minx, miny);

    // Distance from bl to tr is the size
    let size = vec_to_size(tr - bl);
    // let size = (100, 100);

    let c_dist = to_vec(size) * 0.5;
    let inv = mat.inverse_or_zero();

    apply_filter(
        src, 
        size,
        |ds| inv * (ds - c_dist) + sc)
}

pub fn offset(src: &DynamicImage, offset: (u32, u32)) -> RgbaImage {
    let dims = to_vec(src.dimensions());
    let offsetv = to_vec(offset);
    let offset_amount = Vec2::new(
        offsetv.x / dims.x,
        offsetv.y / dims.y
    );
    apply_filter(
        src, 
        add_tuple(offset, src.dimensions()),
        |duv| offset_amount + duv)
}

pub fn scale(src: &DynamicImage, x_fac: f32, y_fac: f32) -> RgbaImage {
    let scale = Mat2::from_rows_slice(&[
        x_fac, 0.0,
        0.0,   y_fac]);
    matrix_filter(src, scale)
}

pub fn shear(src: &DynamicImage, horizontal: f32, vertical: f32) -> RgbaImage {
    let shear = Mat2::from_rows_slice(&[
        1.0 + horizontal * vertical, horizontal,
        vertical, 1.0]);
    matrix_filter(src, shear)
}

pub fn flip_horizontal(src: &DynamicImage) -> RgbaImage {
    let flip = Mat2::from_rows_slice(&[
        -1.0, 0.0,
        0.0, 1.0]);
    matrix_filter(src, flip)
}

pub fn flip_vertical(src: &DynamicImage) -> RgbaImage {
    let flip = Mat2::from_rows_slice(&[
        1.0, 0.0,
        0.0, -1.0]);
    matrix_filter(src, flip)
}

pub fn rotate(src: &DynamicImage, angle: f32) -> RgbaImage {
    let (sin, cos) = angle.to_radians().sin_cos();
    let rotate = Mat2::from_rows_slice(&[
        cos, -sin,
        sin, cos]);
    matrix_filter(src, rotate)
}

pub fn raw_resize(src: &DynamicImage, dim: (u32, u32)) -> RgbaImage {
    let mut dst: RgbaImage = ImageBuffer::new(dim.0, dim.1);
    let empty = Rgba::from([0, 0, 0, 0]);

    let size = src.dimensions();
    let diff = (dim.0 - size.0, dim.1 - size.1);
    let offset = (diff.0 / 2, diff.1 / 2);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = get_pixel_or(src, x - offset.0, y - offset.1, &empty);
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

fn color_filter<F>(src: &DynamicImage, new_color: F) -> RgbaImage
where F: Fn(Rgba<u8>) -> Rgba<u8> {
    let default_color = Rgba::from([0, 0, 0, 0]);
    let mut dst: RgbaImage = ImageBuffer::new(src.width(), src.height());

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = new_color(get_pixel_or(src, x, y, &default_color));
    }

    dst
}

pub fn multiply_color(src: &DynamicImage, factor: f32) -> RgbaImage {
    color_filter(src, |c| Rgba(c.0.map(|n| (n as f32 * factor).round() as u8)))
}

