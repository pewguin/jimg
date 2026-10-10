use std::borrow::Cow;

use image::{DynamicImage, RgbaImage};

use crate::{filters::{color_filter::{MultiplyComponentsFilter, MultiplyFilter}, filter::{ImgOp, RawResizeFilter, apply}, matrix_filter::{flip_horizontal, flip_vertical, rotate_filter, scale_filter}}, timeline::{Ease, Effect, Loop, Op, Param, Timing}};

impl Ease {
    fn apply(self, t: f32) -> f32 {
        match self {
            Ease::Linear => t,
            Ease::Quad => t * t,
            Ease::Cubic => t * t * t,
            Ease::EaseInOut => t * t * (3.0 - 2.0 * t),
        }
    }
}

impl Loop {
    fn apply(self, local: f32) -> f32 {
        match self {
            Loop::Once => local,
            Loop::Reset => {
                if local >= 1.0 { 1.0 } else { local.fract() }
            },
            Loop::Pingpong => {
                let c = (local * 2.0) % 2.0;
                if c <= 1.0 { c } else { 2.0 - c }
            },
        }
    }
}

impl Timing {
    fn progress(&self, t: f32) -> f32 {
        let len = (self.end - self.start).max(1e-6);
        let local = ((t - self.start) / len).clamp(0.0, 1.0);
        self.ease.apply(self.looping.apply(local))
    }
}

impl Param {
    fn at(&self, t: f32) -> f32 {
        self.from + (self.to - self.from) * t
    }
}

impl Effect {
    /// Op at t=0 should be no effect, t=1 full effect
    pub fn apply(&self, img: &DynamicImage, t: f32) -> RgbaImage {
        let t = self.timing.progress(t);
        let op = ImgOp::from(match &self.op {
            Op::Scale { x_factor, y_factor } => ImgOp::from(scale_filter(x_factor.at(t), y_factor.at(t))),
            Op::Shear { horizontal, vertical } => ImgOp::from(scale_filter(horizontal.at(t), vertical.at(t))),
            Op::FlipHorizontal => ImgOp::from(flip_horizontal()),
            Op::FlipVertical => ImgOp::from(flip_vertical()),
            Op::Rotate { angle } => ImgOp::from(rotate_filter(angle.at(t))),
            Op::RawResize { x, y } => ImgOp::from(RawResizeFilter::new((*x, *y))),
            Op::Resize { x, y } => ImgOp::from(RawResizeFilter::new((*x, *y))),
            Op::Multiply { factor } => ImgOp::from(MultiplyFilter::new(factor.at(t))),
            Op::MultiplyAll { r, g, b } => ImgOp::from(MultiplyComponentsFilter::new(r.at(t), g.at(t), b.at(t)))
        });
        apply(&img, op)
    }
}

pub fn apply_all(img: &DynamicImage, effects: &[Effect], t: f32) -> RgbaImage {
    effects.into_iter().fold(Cow::Borrowed(img), |acc, eff| {
        Cow::Owned(DynamicImage::ImageRgba8(eff.apply(&acc.into_owned(), t)))
    }).to_rgba8()
}

