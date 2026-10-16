use std::fmt::Debug;

use glam::{Mat2, Vec2};
use image::{Rgba, Rgba32FImage};

use crate::filters::filter::{Filter, size_to_vec, vec_to_size};

pub trait RemapFilter {
    fn old_to_new(&self, oldv: Vec2, old_size: Vec2, new_size: Vec2) -> Vec2;
    fn new_to_old(&self, newv: Vec2, old_size: Vec2, new_size: Vec2) -> Vec2;
    fn new_size(&self, old_size: Vec2) -> Vec2 {
        let corners = [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]
            .map(|c| Vec2::new(c.x * old_size.x, c.y * old_size.y))
            .map(|c| self.old_to_new(c, old_size, Vec2::ZERO));

        let bl = corners.into_iter().reduce(Vec2::min).unwrap();
        let tr = corners.into_iter().reduce(Vec2::max).unwrap();
        
        tr - bl
    }

    fn as_mat(&self) -> Option<Mat2> { None }
}

#[derive(Clone, Debug)]
pub struct Remap<R: RemapFilter>(pub R);

impl<R: RemapFilter> From<R> for Remap<R> {
    fn from(value: R) -> Self {
        Remap(value)
    }
}

impl<R: RemapFilter + Clone + Debug> Filter for Remap<R> {
    fn apply(&self, img: &Rgba32FImage) -> Rgba32FImage {
        let old_dim = img.dimensions();
        let old_size = size_to_vec(old_dim);
        let sizev = self.0.new_size(old_size);
        let size = vec_to_size(sizev);
        let mut dst = Rgba32FImage::new(size.0, size.1);

        let default_color = Rgba::from([0.0, 0.0, 0.0, 0.0]);

        for (x, y, pixel) in dst.enumerate_pixels_mut() {
            let dstv = size_to_vec((x, y)) + 0.5 - sizev / 2.0;
            let srcv = self.0.new_to_old(dstv, old_size, sizev) + old_size / 2.0;

            let src = vec_to_size(srcv);

            *pixel = if srcv.x < 0.0 || srcv.y < 0.0 || src.0 >= old_dim.0 || src.1 >= old_dim.1 {
                default_color.clone()
            } else {
                *img.get_pixel(src.0, src.1)
            };
        }

        dst
    }

    fn as_mat(&self) -> Option<Mat2> { self.0.as_mat() }
}

#[derive(Clone, Debug)]
pub struct RawResizeFilter {
    dim: Vec2,
}

impl RawResizeFilter {
    pub fn new(size: (u32, u32)) -> Self {
        Self { dim: size_to_vec(size) }
    }
}

impl RemapFilter for RawResizeFilter {
    fn new_to_old(&self, newv: Vec2, _: Vec2, _: Vec2) -> Vec2 { newv }
    fn old_to_new(&self, oldv: Vec2, _: Vec2, _: Vec2) -> Vec2 { oldv }
    fn new_size(&self, _old_size: Vec2) -> Vec2 { self.dim }
}

#[derive(Clone, Debug)]
pub struct ResizeFilter {
    size: Vec2,
}

impl ResizeFilter {
    pub fn new(size: (u32, u32)) -> Self {
        Self { size: size_to_vec(size) }
    }
}

impl RemapFilter for ResizeFilter {
    fn new_to_old(&self, newv: Vec2, old_size: Vec2, new_size: Vec2) -> Vec2 {
        newv * (old_size / new_size)
    }
    fn old_to_new(&self, oldv: Vec2, old_size: Vec2, new_size: Vec2) -> Vec2 {
        oldv / (old_size / new_size)
    }
    fn new_size(&self, _old_size: Vec2) -> Vec2 { self.size }
}
