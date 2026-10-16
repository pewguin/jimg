use glam::{Mat2, Mat3, Vec2};

use crate::filters::remap_filter::RemapFilter;

#[derive(Clone, Debug)]
pub struct AffineFilter {
    mat: Mat2,
    inv: Mat2,
}

impl AffineFilter {
    pub fn new(mat: Mat2) -> Self {
        Self {
            mat,
            inv: mat.inverse(),
        }
    }

    pub fn scale(x_fac: f32, y_fac: f32) -> Self {
        let scale = Mat2::from_rows_slice(&[
            x_fac, 0.0,
            0.0,   y_fac]);
        Self::new(scale)
    }

    pub fn shear(horizontal: f32, vertical: f32) -> Self {
        let shear = Mat2::from_rows_slice(&[
            1.0 + horizontal * vertical, horizontal,
            vertical, 1.0]);
        Self::new(shear)
    }

    pub fn flip_horizontal() -> Self {
        let flip = Mat2::from_rows_slice(&[
            -1.0, 0.0,
            0.0, 1.0]);
        Self::new(flip)
    }

    pub fn flip_vertical() -> Self {
        let flip = Mat2::from_rows_slice(&[
            1.0, 0.0,
            0.0, -1.0]);
        Self::new(flip)
    }

    pub fn rotate(angle: f32) -> Self {
        let (sin, cos) = angle.to_radians().sin_cos();
        let rotate = Mat2::from_rows_slice(&[
            cos, -sin,
            sin, cos]);
        Self::new(rotate)
    }
}

impl RemapFilter for AffineFilter {
    fn old_to_new(&self, oldv: Vec2, _: Vec2, _: Vec2) -> Vec2 {
        self.mat * oldv
    }

    fn new_to_old(&self, newv: Vec2, _: Vec2, _: Vec2) -> Vec2 {
        self.inv * newv
    }

    fn as_mat(&self) -> Option<Mat2> {
        Some(self.mat)
    }
}
