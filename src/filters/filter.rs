use std::ops;

use glam::{Mat2, Vec2};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

use crate::filters::{color_filter::{ColorFilter, MultiplyComponentsFilter, MultiplyFilter, apply_color_filter}, filter::ImgOp::Remap, matrix_filter::MatrixFilter};

type Size = (u32, u32);

pub enum ImgOp {
    Remap(Box<dyn Filter>),
    Color(Box<dyn ColorFilter>),
}

macro_rules! remap_op {
   ($($t:ty),*) => {$(
       impl From<$t> for ImgOp {
           fn from(f: $t) -> Self { ImgOp::Remap(Box::new(f)) }
       }
   )*};
}

macro_rules! color_op {
   ($($t:ty),*) => {$(
       impl From<$t> for ImgOp {
           fn from(f: $t) -> Self { ImgOp::Color(Box::new(f)) }
       }
   )*};
}

remap_op!(RawResizeFilter, ResizeFilter, OffsetFilter, MatrixFilter);
color_op!(MultiplyFilter, MultiplyComponentsFilter);

fn size_to_vec(value: Size) -> Vec2 {
    Vec2::new(value.0 as f32, value.1 as f32)
}

fn vec_to_size(v: Vec2) -> Size {
    (v.x.round() as u32, v.y.round() as u32)
}

pub trait Filter {
    fn new_size(&self, old_size: Vec2) -> Vec2;
    fn dst_to_src(&self, dst: Vec2, old_size: Vec2, new_size: Vec2) -> Vec2;
}

fn get_pixel_or(src: &DynamicImage, x: u32, y: u32, or: &Rgba<u8>) -> Rgba<u8> {
    let (sx, sy) = src.dimensions();

    if x >= 0 && x < sx && y >= 0 && y < sy {
        src.get_pixel(x, y)
    } else {
        or.clone()
    }
}

pub fn apply(src: &DynamicImage, op: ImgOp) -> RgbaImage {
    match op {
        ImgOp::Remap(filter) => apply_filter(src, filter.as_ref()),
        ImgOp::Color(filter) => apply_color_filter(src, filter.as_ref()),
    }
}

fn apply_filter<F>(src: &DynamicImage, filter: &F) -> RgbaImage
where F: Filter + ?Sized {
    let default_color = Rgba::from([0, 0, 0, 0]);
    let src_dim = src.dimensions();
    let src_size = size_to_vec(src_dim);
    let dst_size = filter.new_size(src_size);
    let size = vec_to_size(dst_size);
    let mut dst: RgbaImage = ImageBuffer::new(size.0, size.1);

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        let dst_vec = size_to_vec((x, y)) + 0.5 - dst_size / 2.0;
        let src_vec = filter.dst_to_src(dst_vec, src_size, dst_size) + src_size / 2.0;

        let s_pos = (src_vec.x.floor() as u32, src_vec.y.floor() as u32);


        *pixel = if src_vec.x < 0.0 || src_vec.y < 0.0 || s_pos.0 >= src_dim.0 || s_pos.1 >= src_dim.1 {
            default_color.clone()
        } else {
            src.get_pixel(s_pos.0, s_pos.1)
        };
    }

    dst
}

pub struct OffsetFilter {
    offset: Vec2,
}

impl Filter for OffsetFilter {
    fn new_size(&self, old_size: Vec2) -> Vec2 { self.offset + old_size }
    fn dst_to_src(&self, dst: Vec2, _old_size: Vec2, _new_size: Vec2) -> Vec2 { dst - self.offset }
}


pub struct RawResizeFilter {
    dim: Vec2,
}

impl RawResizeFilter {
    pub fn new(size: Size) -> Self {
        Self { dim: size_to_vec(size) }
    }
}

impl Filter for RawResizeFilter {
    fn new_size(&self, _old_size: Vec2) -> Vec2 { self.dim }
    fn dst_to_src(&self, dst: Vec2, _old_size: Vec2, _new_size: Vec2) -> Vec2 { dst }
}


pub struct ResizeFilter {
    size: Vec2,
}

impl Filter for ResizeFilter {
    fn new_size(&self, _old_size: Vec2) -> Vec2 { self.size }
    fn dst_to_src(&self, dst: Vec2, old_size: Vec2, new_size: Vec2) -> Vec2 {
        dst * (old_size / new_size)
    }
}
