use std::ops;

use glam::{Mat2, Vec2};

use crate::filters::filter::Filter;

pub struct MatrixFilter {
    mat: Mat2,
    inv: Mat2
}

impl MatrixFilter {
    pub fn new(mat: Mat2) -> Self {
        Self {
            mat,
            inv: mat.inverse(),
        }
    }

    pub fn then(self, next: MatrixFilter) -> MatrixFilter {
        next * self
    }
}

impl ops::Mul<MatrixFilter> for MatrixFilter {
    type Output = MatrixFilter;

    fn mul(self, rhs: MatrixFilter) -> Self::Output {
        MatrixFilter::new(self.mat * rhs.mat)
    }
}

impl Filter for MatrixFilter {
    fn new_size(&self, old_size: Vec2) -> Vec2 {
        let corners = [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y]
            .map(|c| Vec2::new(c.x * old_size.x, c.y * old_size.y))
            .map(|c| self.mat * c);

        let bl = corners.into_iter().reduce(Vec2::min).unwrap();
        let tr = corners.into_iter().reduce(Vec2::max).unwrap();

        tr - bl
    }

    fn dst_to_src(&self, dst: Vec2, _old_size: Vec2, _new_size: Vec2) -> Vec2 {
        self.inv * dst
    }
}

pub fn scale_filter(x_fac: f32, y_fac: f32) -> MatrixFilter {
    let scale = Mat2::from_rows_slice(&[
        x_fac, 0.0,
        0.0,   y_fac]);
    MatrixFilter::new(scale)
}

pub fn shear_filter(horizontal: f32, vertical: f32) -> MatrixFilter {
    let shear = Mat2::from_rows_slice(&[
        1.0 + horizontal * vertical, horizontal,
        vertical, 1.0]);
    MatrixFilter::new(shear)
}

pub fn flip_horizontal() -> MatrixFilter {
    let flip = Mat2::from_rows_slice(&[
        -1.0, 0.0,
        0.0, 1.0]);
    MatrixFilter::new(flip)
}

pub fn flip_vertical() -> MatrixFilter {
    let flip = Mat2::from_rows_slice(&[
        1.0, 0.0,
        0.0, -1.0]);
    MatrixFilter::new(flip)
}

pub fn rotate_filter(angle: f32) -> MatrixFilter {
    let (sin, cos) = angle.to_radians().sin_cos();
    let rotate = Mat2::from_rows_slice(&[
        cos, -sin,
        sin, cos]);
    MatrixFilter::new(rotate)
}

