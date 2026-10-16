use image::Rgba32FImage;

use crate::filters::filter::Filter;

#[derive(Clone, Debug)]
pub struct GaussianFilter {
    kernel: Vec<f32>,
}

impl GaussianFilter {
    pub fn new(sigma: f32) -> Self {
        let sigma = sigma.max(0.01);
        let radius = (sigma * 3.0).ceil() as i32;
        let mut kernel: Vec<f32> = (-radius..=radius)
            .map(|i| {
                let x = i as f32;
                (-(x * x) / (2.0 * sigma * sigma)).exp()
            })
            .collect();

        let sum: f32 = kernel.iter().sum();
        kernel.iter_mut().for_each(|k| *k /= sum);

        Self { kernel }
    }

    fn radius(&self) -> i32 {
        (self.kernel.len() / 2) as i32
    }
}

fn blur_pass(
    src: &[[f32; 4]],
    w: usize,
    h: usize,
    filter: &GaussianFilter,
    horizontal: bool,
) -> Vec<[f32; 4]> {
    let r = filter.radius();
    let mut dst = vec![[0.0; 4]; w * h];

    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0_f32; 4];
            for (k, weight) in filter.kernel.iter().enumerate() {
                let offset = k as i32 - r;
                // Clamp to edge so borders don't darken
                let (sx, sy) = if horizontal {
                    ((x as i32 + offset).clamp(0, w as i32 - 1) as usize, y)
                } else {
                    (x, (y as i32 + offset).clamp(0, h as i32 - 1) as usize)
                };
                let p = src[sy * w + sx];
                for c in 0..4 {
                    acc[c] += p[c] * weight;
                }
            }
            dst[y * w + x] = acc;
        }
    }

    dst
}

impl Filter for GaussianFilter {
    fn apply(&self, img: &Rgba32FImage) -> Rgba32FImage {
        let (w, h) = (img.width() as usize, img.height() as usize);

        let buf: Vec<[f32; 4]> = img
            .pixels()
            .map(|p| {
                p.0
            })
            .collect();

        let buf = blur_pass(&buf, w, h, self, true);
        let buf = blur_pass(&buf, w, h, self, false);

        let mut dst = Rgba32FImage::new(w as u32, h as u32);
        for (pixel, p) in dst.pixels_mut().zip(buf) {
            *pixel = image::Rgba(p);
        }

        dst
    }
}
