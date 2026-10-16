use std::{collections::HashSet, error::Error, path::PathBuf};

use ::image::Rgba32FImage;
use eframe::egui::text::LayoutJob;
use eframe::egui::{
    self, Color32, FontId, Image, TextEdit, TextFormat, TextureHandle, TextureOptions,
};

use crate::{image, script::Program};

const DEFAULT_SCRIPT: &str = "source.rotate(360.0 * t)";

pub fn run() -> Result<(), eframe::Error> {
    eframe::run_native(
        "jimg",
        eframe::NativeOptions::default(),
        Box::new(|_creation_context| Ok(Box::<JimgApp>::default())),
    )
}

struct JimgApp {
    source: Option<Rgba32FImage>,
    preview: Option<TextureHandle>,
    script: String,
    t: f32,
    duration_seconds: f32,
    fps: u32,
    compiled_script: Option<String>,
    program: Option<Program>,
    preview_dirty: bool,
    message: Option<String>,
}

impl Default for JimgApp {
    fn default() -> Self {
        Self {
            source: None,
            preview: None,
            script: DEFAULT_SCRIPT.to_owned(),
            t: 0.0,
            duration_seconds: 1.0,
            fps: 15,
            compiled_script: None,
            program: None,
            preview_dirty: true,
            message: None,
        }
    }
}

impl JimgApp {
    fn load_source(&mut self, path: PathBuf) {
        match image::load_image(&path.to_string_lossy()) {
            Ok(source) => {
                self.source = Some(source);
                self.preview = None;
                self.preview_dirty = true;
                self.message = None;
            }
            Err(error) => self.message = Some(format!("Could not open image: {error}")),
        }
    }

    fn update_preview(&mut self, ctx: &egui::Context) {
        if !self.preview_dirty {
            return;
        }
        self.preview_dirty = false;
        let Some(source) = self.source.as_ref() else {
            self.preview = None;
            return;
        };

        if self.compiled_script.as_deref() != Some(self.script.as_str()) {
            self.compiled_script = Some(self.script.clone());
            match Program::from_script(&self.script) {
                Ok(program) => self.program = Some(program),
                Err(error) => {
                    self.program = None;
                    self.preview = None;
                    self.message = Some(format!("Script error: {error}"));
                }
            }
        }

        let Some(program) = self.program.as_ref() else {
            return;
        };
        match program.evaluate(source, self.t) {
            Ok(output) => {
                let rgba = ::image::DynamicImage::ImageRgba32F(output).to_rgba8();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba.width() as usize, rgba.height() as usize],
                    rgba.as_raw(),
                );
                if let Some(preview) = self.preview.as_mut() {
                    preview.set(color_image, TextureOptions::LINEAR);
                } else {
                    self.preview = Some(ctx.load_texture(
                        "script-preview",
                        color_image,
                        TextureOptions::LINEAR,
                    ));
                }
                self.message = None;
            }
            Err(error) => {
                self.preview = None;
                self.message = Some(format!("Script error: {error}"));
            }
        }
    }

    fn save_animation(&mut self) {
        let Some(source) = self.source.as_ref() else {
            self.message = Some("Open an image before exporting.".to_owned());
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("GIF image", &["gif"])
            .set_file_name("output.gif")
            .save_file()
        else {
            return;
        };

        let mut path = path;
        if path.extension().is_none() {
            path.set_extension("gif");
        }
        let result = (|| -> Result<(), Box<dyn Error>> {
            let program = Program::from_script(&self.script)?;
            let frame_count = (self.duration_seconds * self.fps as f32).round().max(1.0) as u32;
            let frames = (0..frame_count)
                .map(|frame| program.evaluate(source, frame as f32 / frame_count as f32))
                .collect::<Result<Vec<_>, _>>()?;
            image::save_gif(frames, &path, self.fps)?;
            Ok(())
        })();
        self.message = Some(match result {
            Ok(()) => format!("Saved {}", path.display()),
            Err(error) => format!("Could not save GIF: {error}"),
        });
    }
}

impl eframe::App for JimgApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_preview(ctx);

        egui::SidePanel::right("toolbar")
            .default_width(390.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                if ui
                    .add(egui::Slider::new(&mut self.t, 0.0..=1.0).text("t"))
                    .changed()
                {
                    self.preview_dirty = true;
                }
                if ui.button("Open image...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp", "tiff"])
                        .pick_file()
                    {
                        self.load_source(path);
                    }
                }
                ui.add_space(8.0);

                let editor_height = (ui.available_height() - 170.0).max(120.0);
                let mut layouter = |ui: &egui::Ui, text: &str, wrap_width| {
                    let mut job = highlight_script(text);
                    job.wrap.max_width = wrap_width;
                    ui.fonts(|fonts| fonts.layout_job(job))
                };
                let editor = ui.add_sized(
                    [ui.available_width(), editor_height],
                    TextEdit::multiline(&mut self.script)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
                if editor.changed() {
                    self.preview_dirty = true;
                }

                ui.separator();
                ui.label("export");
                ui.add(
                    egui::Slider::new(&mut self.duration_seconds, 0.1..=30.0).text("Duration (s)"),
                );
                ui.add(egui::Slider::new(&mut self.fps, 1..=60).text("FPS"));
                if ui
                    .add_enabled(self.source.is_some(), egui::Button::new("Save GIF..."))
                    .clicked()
                {
                    self.save_animation();
                }
                if let Some(message) = self.message.as_deref() {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(message).color(
                        if message.starts_with("Saved ") {
                            Color32::LIGHT_GREEN
                        } else {
                            Color32::LIGHT_RED
                        },
                    ));
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(preview) = &self.preview {
                let available = ui.available_size();
                let image_size = preview.size_vec2();
                let scale = (available.x / image_size.x)
                    .min(available.y / image_size.y)
                    .min(1.0);
                ui.centered_and_justified(|ui| {
                    ui.add(Image::new((preview.id(), image_size * scale)));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Open an image to preview the script output");
                });
            }
        });
    }
}

fn highlight_script(text: &str) -> LayoutJob {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    while cursor < text.len() {
        let current = text[cursor..]
            .chars()
            .next()
            .expect("cursor is on a char boundary");
        if current.is_whitespace() {
            cursor += current.len_utf8();
            continue;
        }
        let start = cursor;
        if is_identifier_start(current) {
            cursor += current.len_utf8();
            while cursor < text.len() {
                let character = text[cursor..].chars().next().unwrap();
                if !is_identifier_continue(character) {
                    break;
                }
                cursor += character.len_utf8();
            }
            tokens.push((start, cursor, TokenKind::Identifier));
        } else if current.is_ascii_digit() {
            cursor += current.len_utf8();
            while cursor < text.len() {
                let character = text[cursor..].chars().next().unwrap();
                if !(character.is_ascii_alphanumeric() || matches!(character, '.' | '_')) {
                    break;
                }
                cursor += character.len_utf8();
            }
            tokens.push((start, cursor, TokenKind::Number));
        } else {
            cursor += current.len_utf8();
            tokens.push((start, cursor, TokenKind::Other));
        }
    }

    let mut variables = HashSet::from(["source".to_owned(), "t".to_owned()]);
    for pair in tokens.windows(2) {
        if pair[0].2 == TokenKind::Identifier && &text[pair[0].0..pair[0].1] == "let" {
            if pair[1].2 == TokenKind::Identifier {
                variables.insert(text[pair[1].0..pair[1].1].to_owned());
            }
        }
    }

    let mut job = LayoutJob::default();
    let mut previous_end = 0;
    for (start, end, kind) in tokens.iter().copied() {
        if previous_end < start {
            job.append(
                &text[previous_end..start],
                0.0,
                TextFormat::simple(FontId::monospace(14.0), Color32::from_rgb(235, 235, 235)),
            );
        }
        let color = match kind {
            TokenKind::Number => Color32::from_rgb(255, 174, 82),
            TokenKind::Identifier => {
                let name = &text[start..end];
                let followed_by_call = text[end..].trim_start().starts_with('(');
                if followed_by_call {
                    Color32::from_rgb(105, 180, 255)
                } else if variables.contains(name) {
                    Color32::from_rgb(235, 235, 235)
                } else {
                    Color32::from_rgb(255, 105, 105)
                }
            }
            TokenKind::Other => Color32::from_rgb(255, 105, 105),
        };
        job.append(
            &text[start..end],
            0.0,
            TextFormat::simple(FontId::monospace(14.0), color),
        );
        previous_end = end;
    }
    if previous_end < text.len() {
        job.append(
            &text[previous_end..],
            0.0,
            TextFormat::simple(FontId::monospace(14.0), Color32::from_rgb(235, 235, 235)),
        );
    }
    job
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Identifier,
    Number,
    Other,
}

fn is_identifier_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

fn is_identifier_continue(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}
