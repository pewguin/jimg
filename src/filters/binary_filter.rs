use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};

pub trait BinaryFilter {
    fn combine(&self, lhs: Rgba<u8>, rhs: Rgba<u8>) -> Rgba<u8>;
}

pub fn apply_binary_filter<F>(lhs: &DynamicImage, rhs: &DynamicImage, filter: &F) -> RgbaImage
where F: BinaryFilter + ?Sized {
    let mut dst: RgbaImage = ImageBuffer::new(lhs.width().min(rhs.width()), lhs.height().min(rhs.height()));

    for (x, y, pixel) in dst.enumerate_pixels_mut() {
        *pixel = filter.combine(lhs.get_pixel(x, y), rhs.get_pixel(x, y));
    }

    dst
}

pub struct AddFilter {}

impl AddFilter {
    pub fn new() -> Self { Self {} }
}

impl BinaryFilter for AddFilter {
    fn combine(&self, lhs: Rgba<u8>, rhs: Rgba<u8>) -> Rgba<u8> {
        let l = lhs.0;
        let r = rhs.0;
        Rgba([
            l[0].saturating_add(r[0]),
            l[1].saturating_add(r[1]),
            l[2].saturating_add(r[2]),
            l[3].max(r[3]),
        ])
    }
}

pub struct SubFilter {}

impl SubFilter {
    pub fn new() -> Self { Self {} }
}

impl BinaryFilter for SubFilter {
    fn combine(&self, lhs: Rgba<u8>, rhs: Rgba<u8>) -> Rgba<u8> {
        let l = lhs.0;
        let r = rhs.0;
        Rgba([
            l[0].saturating_sub(r[0]),
            l[1].saturating_sub(r[1]),
            l[2].saturating_sub(r[2]),
            l[3].max(r[3]),
        ])
    }
}
