use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

pub trait ColorFilter {
    fn filter(&self, c: Rgba<u8>) -> Rgba<u8>;
}

pub fn apply_color_filter<F>(src: &DynamicImage, filter: &F) -> RgbaImage
where F: ColorFilter + ?Sized {
    let mut dst: RgbaImage = ImageBuffer::new(src.width(), src.height());

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = filter.filter(src.get_pixel(x, y));
    }

    dst
}

pub struct MultiplyFilter { factor: f32 }

impl MultiplyFilter {
    pub fn new(factor: f32) -> MultiplyFilter {
        MultiplyFilter { factor }
    }
}

impl ColorFilter for MultiplyFilter {
    fn filter(&self, c: Rgba<u8>) -> Rgba<u8> {
        Rgba(c.0.map(|n| (n as f32 * self.factor).round() as u8))
    }
}

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
    fn filter(&self, c: Rgba<u8>) -> Rgba<u8> {
        let f = c.0.map(|n| n as f32);
        let float_scaled = [f[0] * self.r, f[1] * self.g, f[2] * self.b, f[3]];
        Rgba(float_scaled.map(|n| n as u8))
    }
}

