use std::fmt::Debug;

use image::{ImageBuffer, Rgba, Rgba32FImage};

use crate::filters::filter::Filter;

pub trait ColorFilter {
    fn filter(&self, c: Rgba<f32>) -> Rgba<f32>;
}

#[derive(Clone, Debug)]
pub struct Color<C: ColorFilter> (pub C);

impl<C: ColorFilter> From<C> for Color<C> {
    fn from(value: C) -> Self {
        Color(value)
    }
}

impl<C: ColorFilter + Clone + Debug> Filter for Color<C> {
    fn apply(&self, img: &Rgba32FImage) -> Rgba32FImage {
        let mut dst = img.clone();
        dst.pixels_mut().for_each(|p| *p = self.0.filter(p.clone()));
        dst
    }
}

#[derive(Clone, Debug)]
pub struct MultiplyFilter { factor: f32 }

impl MultiplyFilter {
    pub fn new(factor: f32) -> MultiplyFilter {
        MultiplyFilter { factor }
    }
}

impl ColorFilter for MultiplyFilter {
    fn filter(&self, c: Rgba<f32>) -> Rgba<f32> {
        Rgba(c.0.map(|n| n * self.factor))
    }
}

#[derive(Clone, Debug)]
pub struct MultiplyComponentsFilter {
    r: f32,
    g: f32,
    b: f32,
}

impl MultiplyComponentsFilter {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

impl ColorFilter for MultiplyComponentsFilter {
    fn filter(&self, c: Rgba<f32>) -> Rgba<f32> {
        Rgba([c[0] * self.r, c[1] * self.g, c[2] * self.b, c[3]])
    }
}

fn luminosity(color: Rgba<f32>) -> f32 {
    0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2]
}

#[derive(Clone, Debug)]
pub struct GrayscaleFilter {}

impl GrayscaleFilter {
    pub fn new() -> Self { Self {} }
}

impl ColorFilter for GrayscaleFilter {
    fn filter(&self, c: Rgba<f32>) -> Rgba<f32> {
        let v = luminosity(c);
        Rgba([v, v, v, c[3]])
    }
}

fn smoothstep(start: f32, end: f32, t: f32) -> f32 {
    let x = ((t - start) / (end - start)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[derive(Clone, Debug)]
pub struct SmoothstepFilter {
    start: f32,
    end: f32,
}

impl SmoothstepFilter {
    pub fn new(start: f32, end: f32) -> Self {
        Self {
            start,
            end,
        }
    }
}

impl ColorFilter for SmoothstepFilter {
    fn filter(&self, c: Rgba<f32>) -> Rgba<f32> {
        let l = luminosity(c);
        let v = smoothstep(self.start, self.end, l);
        Rgba([v, v, v, c[3]])
    }
}

#[derive(Clone, Debug)]
pub struct InvertFilter {}

impl InvertFilter {
    pub fn new() -> Self { Self {} }
}

impl ColorFilter for InvertFilter {
    fn filter(&self, c: Rgba<f32>) -> Rgba<f32> {
        let a = c.0;
        Rgba([
            1.0 - a[0],
            1.0 - a[1],
            1.0 - a[2],
            a[3]
        ])
    }
}
