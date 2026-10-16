use std::fmt::Debug;

use dyn_clone::DynClone;
use glam::{Mat2, Vec2};
use image::Rgba32FImage;

pub fn vec_to_size(vec: Vec2) -> (u32, u32) {
    (vec.x.round() as u32, vec.y.round() as u32)
}

pub fn size_to_vec(size: (u32, u32)) -> Vec2 {
    Vec2::new(size.0 as f32, size.1 as f32)
}

pub trait Filter: DynClone + Debug {
    fn apply(&self, img: &Rgba32FImage) -> Rgba32FImage;
    fn as_mat(&self) -> Option<Mat2> {
        None
    }
}

dyn_clone::clone_trait_object!(Filter);
