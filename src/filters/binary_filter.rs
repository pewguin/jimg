use image::{ImageBuffer, Rgba, Rgba32FImage};

pub trait BinaryFilter {
    fn apply(&self, lhs: &Rgba32FImage, rhs: &Rgba32FImage) -> Rgba32FImage;
}

pub trait BinaryColorFilter {
    fn combine(&self, lhs: Rgba<f32>, rhs: Rgba<f32>) -> Rgba<f32>;
}

impl<F> BinaryFilter for F
where
    F: BinaryColorFilter + ?Sized {
    fn apply(&self, lhs: &Rgba32FImage, rhs: &Rgba32FImage) -> Rgba32FImage {
        let mut dst: Rgba32FImage = ImageBuffer::new(lhs.width().min(rhs.width()), lhs.height().min(rhs.height()));

        for (x, y, pixel) in dst.enumerate_pixels_mut() {
            *pixel = self.combine(*lhs.get_pixel(x, y), *rhs.get_pixel(x, y));
        }
        
        dst
    }
}

pub struct AddFilter {}

impl AddFilter {
    pub fn new() -> Self { Self {} }
}

impl BinaryColorFilter for AddFilter {
    fn combine(&self, lhs: Rgba<f32>, rhs: Rgba<f32>) -> Rgba<f32> {
        let l = lhs.0;
        let r = rhs.0;
        Rgba([
            l[0] + r[0],
            l[1] + r[1],
            l[2] + r[2],
            l[3].max(r[3]),
        ])
    }
}

pub struct SubFilter {}

impl SubFilter {
    pub fn new() -> Self { Self {} }
}

impl BinaryColorFilter for SubFilter {
    fn combine(&self, lhs: Rgba<f32>, rhs: Rgba<f32>) -> Rgba<f32> {
        let l = lhs.0;
        let r = rhs.0;
        Rgba([
            l[0] - r[0],
            l[1] - r[1],
            l[2] - r[2],
            l[3].max(r[3]),
        ])
    }
}
