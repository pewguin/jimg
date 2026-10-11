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

fn luminosity(color: Rgba<u8>) -> f32 {
    (0.2126 * color[0] as f32 + 0.7152 * color[1] as f32 + 0.0722 * color[2] as f32) / 255.0
}

pub struct GrayscaleFilter {}

impl GrayscaleFilter {
    pub fn new() -> Self { Self {} }
}

impl ColorFilter for GrayscaleFilter {
    fn filter(&self, c: Rgba<u8>) -> Rgba<u8> {
        let v = (luminosity(c) * 255.0).round().clamp(0.0, 255.0) as u8;
        Rgba([v, v, v, c[3]])
    }
}

fn smoothstep(start: f32, end: f32, t: f32) -> f32 {
    let x = ((t - start) / (end - start)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

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
    fn filter(&self, c: Rgba<u8>) -> Rgba<u8> {
        let l = luminosity(c);
        let v = (smoothstep(self.start, self.end, l) * 255.0).round().clamp(0.0, 255.0) as u8;
        Rgba([v, v, v, c[3]])
    }
}
