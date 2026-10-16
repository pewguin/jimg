use std::{error::Error, fs, mem, path::Path};

use image::Rgba32FImage;
use rhai::{AST, Engine, EvalAltResult, Scope};

use crate::filters::{affine_filter::AffineFilter, binary_filter::{AddFilter, BinaryFilter, SubFilter}, blur_filter::GaussianFilter, color_filter::{Color, GrayscaleFilter, InvertFilter, MultiplyComponentsFilter, MultiplyFilter, SmoothstepFilter}, filter::{self, Filter}, remap_filter::{RawResizeFilter, Remap, ResizeFilter}};

pub struct Program {
    engine: Engine,
    ast: AST,
}

#[derive(Clone)]
struct ScriptImage {
    img: Rgba32FImage,
    ops: Vec<Box<dyn Filter>>,
}

impl ScriptImage {
    fn new(img: Rgba32FImage) -> Self {
        Self { img, ops: Vec::new() }
    }
    fn filtered(mut self) -> Rgba32FImage {
        self.optimize_filters();
        self.ops.iter().fold(self.img, |acc, f| f.apply(&acc))
    }

    fn optimize_filters(&mut self) {
        let mut optimized: Vec<Box<dyn Filter>> = Vec::new();
        let mut iter = mem::take(&mut self.ops).into_iter();

        let mut prev = match iter.next() {
            Some(f) => f,
            None => return
        };

        while let Some(op) = iter.next() {
            if let (Some(first), Some(second)) = (prev.as_mat(), op.as_mat()) {
                prev = Box::new(Remap(AffineFilter::new(second * first)));
            } else {
                optimized.push(prev);
                prev = op;
            }
        }

        optimized.push(prev);
        self.ops = optimized;
    }

    fn add_filter(&mut self, filter: Box<dyn Filter>) {
        self.ops.push(filter);
    }
}

impl Program {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let script = fs::read_to_string(path)?;
        let engine = image_engine();
        let ast = engine.compile(&script)?;
        Ok(Self { engine, ast })
    }

    pub fn evaluate(&self, source: &Rgba32FImage, t: f32) -> Result<Rgba32FImage, Box<dyn Error>> {
        let mut scope = Scope::new();
        scope.push("source", ScriptImage::new(source.clone()));
        scope.push("t", t as f64);
        let image: ScriptImage = self.engine.eval_ast_with_scope(&mut scope, &self.ast)?;
        Ok(image.filtered())
    }
}

fn image_engine() -> Engine {
    let mut engine = Engine::new();
    // Useful
    engine.register_fn("smoothstep", |start: f64, end: f64, t: f64 | {
        let x = ((t - start) / (end - start)).clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    });
    // Image manip
    engine.register_type_with_name::<ScriptImage>("Image");

    engine.register_fn("+", |i1: ScriptImage, i2: ScriptImage| {
        ScriptImage::new(AddFilter::new().apply(&i1.filtered(), &i2.filtered()))
    });
    engine.register_fn("-", |i1: ScriptImage, i2: ScriptImage| {
        ScriptImage::new(SubFilter::new().apply(&i1.filtered(), &i2.filtered()))
    });
    // Filters
    engine.register_fn("scale", |mut image: ScriptImage, x: f64, y: f64| {
        image.add_filter(Box::new(Remap(AffineFilter::scale(x as f32, y as f32))));
        image
    });
    engine.register_fn("shear", |mut image: ScriptImage, x: f64, y: f64| {
        image.add_filter(Box::new(Remap(AffineFilter::shear(x as f32, y as f32))));
        image
    });
    engine.register_fn("flip_horizontal", |mut image: ScriptImage| {
        image.add_filter(Box::new(Remap(AffineFilter::flip_horizontal())));
        image
    });
    engine.register_fn("flip_vertical", |mut image: ScriptImage| {
        image.add_filter(Box::new(Remap(AffineFilter::flip_vertical())));
        image
    });
    engine.register_fn("rotate", |mut image: ScriptImage, degrees: f64| {
        image.add_filter(Box::new(Remap(AffineFilter::rotate(degrees as f32))));
        image
    });
    engine.register_fn(
        "raw_resize",
        |mut image: ScriptImage, x: i64, y: i64| -> Result<ScriptImage, Box<EvalAltResult>> {
            let size = checked_size(x, y)?;
            image.add_filter(Box::new(Remap(RawResizeFilter::new(size))));
            Ok(image)
        },
    );
    engine.register_fn(
        "resize",
        |mut image: ScriptImage, x: i64, y: i64| -> Result<ScriptImage, Box<EvalAltResult>> {
            let size = checked_size(x, y)?;
            image.add_filter(Box::new(Remap(ResizeFilter::new(size))));
            Ok(image)
        },
    );
    engine.register_fn("multiply", |mut image: ScriptImage, factor: f64| {
        image.add_filter(Box::new(Color(MultiplyFilter::new(factor as f32))));
        image
    });
    engine.register_fn("multiply_all", |mut image: ScriptImage, r: f64, g: f64, b: f64| {
        image.add_filter(Box::new(Color(MultiplyComponentsFilter::new(
            r as f32, g as f32, b as f32,
        ))));
        image
    });
    engine.register_fn("grayscale", |mut image: ScriptImage| {
        image.add_filter(Box::new(Color(GrayscaleFilter::new())));
        image
    });
    engine.register_fn("gaussian", |mut image: ScriptImage, sigma: f64| {
        image.add_filter(Box::new(GaussianFilter::new(sigma as f32)));
        image
    });
    engine.register_fn("smoothstep", |mut image: ScriptImage, start: f64, end: f64| {
        image.add_filter(Box::new(Color(SmoothstepFilter::new(start as f32, end as f32))));
        image
    });
    engine.register_fn("invert", |mut image: ScriptImage| {
        image.add_filter(Box::new(Color(InvertFilter::new())));
        image
    });
    engine
}

fn checked_size(x: i64, y: i64) -> Result<(u32, u32), Box<EvalAltResult>> {
    let convert = |value: i64, axis| {
        u32::try_from(value)
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| {
                Box::new(EvalAltResult::ErrorRuntime(
                    format!("image {axis} dimension must be a positive 32-bit integer").into(),
                    rhai::Position::NONE,
                ))
            })
    };
    Ok((convert(x, "width")?, convert(y, "height")?))
}
