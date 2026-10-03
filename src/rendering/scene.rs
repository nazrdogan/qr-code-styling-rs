//! Backend-independent description of a rendered QR code.
//!
//! The SVG renderer computes the layout once into a [`Scene`]; each output
//! backend (SVG, CMYK PDF) then serializes the same scene.

use std::fmt::Write;

use crate::config::{Color, ColorStop};
use crate::figures::traits::Num;

/// How a shape is filled.
#[derive(Debug, Clone)]
pub(crate) enum Paint {
    Solid(Color),
    Linear {
        id: String,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        stops: Vec<ColorStop>,
    },
    Radial {
        id: String,
        cx: f64,
        cy: f64,
        r: f64,
        stops: Vec<ColorStop>,
    },
}

/// A filled path. `d` uses the restricted SVG path grammar emitted by
/// `PathBuilder`: `M`, `m`, `l`, `a` and `z`.
#[derive(Debug, Clone)]
pub(crate) struct Shape {
    pub paint: Paint,
    pub even_odd: bool,
    pub d: String,
}

/// Background rectangle, optionally with rounded corners.
#[derive(Debug, Clone)]
pub(crate) struct Background {
    pub x: f64,
    pub y: f64,
    pub width: u32,
    pub height: u32,
    pub rx: f64,
    pub paint: Paint,
}

/// Embedded logo, fitted inside its box (`xMidYMid meet`).
#[derive(Debug, Clone)]
pub(crate) struct ImageItem<'a> {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub data: &'a [u8],
}

/// Everything needed to draw a QR code, back to front.
#[derive(Debug, Clone)]
pub(crate) struct Scene<'a> {
    pub width: u32,
    pub height: u32,
    pub crisp_edges: bool,
    pub background: Background,
    pub shapes: Vec<Shape>,
    pub image: Option<ImageItem<'a>>,
}

impl Paint {
    /// SVG `fill` value; gradient definitions are appended to `defs`.
    fn to_svg(&self, defs: &mut String) -> String {
        match self {
            Paint::Solid(color) => color.to_hex(),
            Paint::Linear { id, x1, y1, x2, y2, stops } => {
                let _ = writeln!(
                    defs,
                    r#"<linearGradient id="{}" gradientUnits="userSpaceOnUse" x1="{}" y1="{}" x2="{}" y2="{}">"#,
                    id, x1, y1, x2, y2
                );
                push_stops(defs, stops);
                defs.push_str("</linearGradient>\n");
                format!("url(#{})", id)
            }
            Paint::Radial { id, cx, cy, r, stops } => {
                let _ = writeln!(
                    defs,
                    r#"<radialGradient id="{}" gradientUnits="userSpaceOnUse" fx="{}" fy="{}" cx="{}" cy="{}" r="{}">"#,
                    id, cx, cy, cx, cy, r
                );
                push_stops(defs, stops);
                defs.push_str("</radialGradient>\n");
                format!("url(#{})", id)
            }
        }
    }
}

fn push_stops(defs: &mut String, stops: &[ColorStop]) {
    for stop in stops {
        let _ = writeln!(
            defs,
            r#"<stop offset="{}%" stop-color="{}"/>"#,
            stop.offset * 100.0,
            stop.color.to_hex()
        );
    }
}

impl Scene<'_> {
    /// Serialize the scene as an SVG document.
    pub fn to_svg(&self) -> String {
        let mut defs = String::new();
        let shapes_len: usize = self.shapes.iter().map(|s| s.d.len() + 64).sum();
        let mut elements = String::with_capacity(shapes_len + 1024);

        // Background
        let bg = &self.background;
        let fill = bg.paint.to_svg(&mut defs);
        let _ = write!(
            elements,
            r#"<rect x="{}" y="{}" width="{}" height="{}""#,
            Num(bg.x),
            Num(bg.y),
            bg.width,
            bg.height
        );
        if bg.rx > 0.0 {
            let _ = write!(elements, r#" rx="{}""#, Num(bg.rx));
        }
        let _ = writeln!(elements, r#" fill="{}"/>"#, fill);

        // Dots and corners
        for shape in &self.shapes {
            let fill = shape.paint.to_svg(&mut defs);
            let rule = if shape.even_odd { r#" fill-rule="evenodd""# } else { "" };
            let _ = writeln!(elements, r#"<path fill="{}"{} d="{}"/>"#, fill, rule, shape.d);
        }

        // Logo
        if let Some(image) = &self.image {
            let base64_data =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image.data);
            // Plain `href` (SVG 2) avoids embedding the base64 payload twice
            let _ = writeln!(
                elements,
                r#"<image href="data:{};base64,{}" x="{}" y="{}" width="{}" height="{}"/>"#,
                image_mime_type(image.data),
                base64_data,
                Num(image.x),
                Num(image.y),
                Num(image.width),
                Num(image.height)
            );
        }

        let shape_rendering = if self.crisp_edges {
            r#" shape-rendering="crispEdges""#
        } else {
            ""
        };

        let mut svg = String::with_capacity(defs.len() + elements.len() + 512);
        let _ = write!(
            svg,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}"{shape_rendering}>
<defs>
{defs}</defs>
{elements}</svg>"#,
            w = self.width,
            h = self.height,
        );
        svg
    }
}

/// Detect the mime type of embedded image data (defaults to PNG).
pub(crate) fn image_mime_type(data: &[u8]) -> &'static str {
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if data.starts_with(&[0xFF, 0xD8]) {
        "image/jpeg"
    } else if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/png"
    }
}
