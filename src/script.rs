use std::{error::Error, fs, path::Path};

use image::{DynamicImage, RgbaImage};
use rhai::{AST, Engine, EvalAltResult, Scope};

use crate::filters::{
    binary_filter::{AddFilter, BinaryFilter, SubFilter, apply_binary_filter}, blur_filter::{GaussianFilter, apply_gaussian}, color_filter::{GrayscaleFilter, InvertFilter, MultiplyComponentsFilter, MultiplyFilter, SmoothstepFilter}, filter::{ImgOp, RawResizeFilter, ResizeFilter, apply}, matrix_filter::{
        flip_horizontal, flip_vertical, rotate_filter, scale_filter, shear_filter,
    },
};

#[derive(Clone)]
pub struct ScriptImage(DynamicImage);

impl ScriptImage {
    fn apply(self, op: ImgOp) -> Self {
        Self(DynamicImage::ImageRgba8(apply(&self.0, op)))
    }
}

fn apply_binary<F>(img_a: ScriptImage, img_b: ScriptImage, filter: &F) -> ScriptImage
where F: BinaryFilter {
    ScriptImage(DynamicImage::ImageRgba8(apply_binary_filter(&img_a.0, &img_b.0, filter)))
}

pub struct Program {
    engine: Engine,
    ast: AST,
}

impl Program {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let script = fs::read_to_string(path)?;
        let engine = image_engine();
        let ast = engine.compile(&script)?;
        Ok(Self { engine, ast })
    }

    pub fn evaluate(&self, source: &DynamicImage, t: f32) -> Result<RgbaImage, Box<dyn Error>> {
        let mut scope = Scope::new();
        scope.push("source", ScriptImage(source.clone()));
        scope.push("t", t as f64);
        let image: ScriptImage = self.engine.eval_ast_with_scope(&mut scope, &self.ast)?;
        Ok(image.0.to_rgba8())
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
        apply_binary(i1, i2, &AddFilter::new())
    });
    engine.register_fn("-", |i1: ScriptImage, i2: ScriptImage| {
        apply_binary(i1, i2, &SubFilter::new())
    });
    // Filters
    engine.register_fn("scale", |image: ScriptImage, x: f64, y: f64| {
        image.apply(scale_filter(x as f32, y as f32).into())
    });
    engine.register_fn("shear", |image: ScriptImage, x: f64, y: f64| {
        image.apply(shear_filter(x as f32, y as f32).into())
    });
    engine.register_fn("flip_horizontal", |image: ScriptImage| {
        image.apply(flip_horizontal().into())
    });
    engine.register_fn("flip_vertical", |image: ScriptImage| {
        image.apply(flip_vertical().into())
    });
    engine.register_fn("rotate", |image: ScriptImage, degrees: f64| {
        image.apply(rotate_filter(degrees as f32).into())
    });
    engine.register_fn(
        "raw_resize",
        |image: ScriptImage, x: i64, y: i64| -> Result<ScriptImage, Box<EvalAltResult>> {
            let size = checked_size(x, y)?;
            Ok(image.apply(RawResizeFilter::new(size).into()))
        },
    );
    engine.register_fn(
        "resize",
        |image: ScriptImage, x: i64, y: i64| -> Result<ScriptImage, Box<EvalAltResult>> {
            let size = checked_size(x, y)?;
            Ok(image.apply(ResizeFilter::new(size).into()))
        },
    );
    engine.register_fn("multiply", |image: ScriptImage, factor: f64| {
        image.apply(MultiplyFilter::new(factor as f32).into())
    });
    engine.register_fn(
        "multiply_all",
        |image: ScriptImage, r: f64, g: f64, b: f64| {
            image.apply(MultiplyComponentsFilter::new(r as f32, g as f32, b as f32).into())
        },
    );
    engine.register_fn("grayscale", |image: ScriptImage| {
        image.apply(GrayscaleFilter::new().into())
    });
    engine.register_fn("gaussian", |image: ScriptImage, sigma: f64| {
        let filter = GaussianFilter::new(sigma as f32);
        ScriptImage(DynamicImage::ImageRgba8(apply_gaussian(&image.0, &filter)))
    });
    engine.register_fn("smoothstep", |image: ScriptImage, start: f64, end: f64| {
        image.apply(SmoothstepFilter::new(start as f32, end as f32).into())
    });
    engine.register_fn("invert", |image: ScriptImage| {
        image.apply(InvertFilter::new().into())
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
