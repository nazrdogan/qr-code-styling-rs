//! SVG renderer for QR codes.

use std::borrow::Cow;
use std::f64::consts::PI;
use std::fmt::Write;

use crate::config::{Color, Gradient, QRCodeStylingOptions};
use crate::core::QRMatrix;
use crate::error::Result;
use crate::figures::traits::Num;
use crate::figures::{QRCornerDot, QRCornerSquare, QRDot};
use crate::types::{GradientType, ShapeType};
use crate::utils::calculate_image_size;

/// SVG renderer for QR codes.
///
/// Each layer (background, dots, each corner square and corner dot) is a
/// single filled element, so rasterizers don't need per-layer clip masks.
pub struct SvgRenderer<'a> {
    options: Cow<'a, QRCodeStylingOptions>,
    instance_id: u64,
}

/// Square mask for corner squares (7x7 pattern).
const SQUARE_MASK: [[u8; 7]; 7] = [
    [1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1],
];

/// Dot mask for corner dots (7x7 pattern).
const DOT_MASK: [[u8; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
];

static INSTANCE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl SvgRenderer<'static> {
    /// Create a new SVG renderer that owns its options.
    pub fn new(options: QRCodeStylingOptions) -> Self {
        SvgRenderer::with_options(Cow::Owned(options))
    }
}

impl<'a> SvgRenderer<'a> {
    /// Create a new SVG renderer that borrows its options (no copy of image data).
    pub fn from_ref(options: &'a QRCodeStylingOptions) -> Self {
        Self::with_options(Cow::Borrowed(options))
    }

    fn with_options(options: Cow<'a, QRCodeStylingOptions>) -> Self {
        let instance_id = INSTANCE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            options,
            instance_id,
        }
    }

    /// Render the QR code as SVG string.
    pub fn render(&self, matrix: &QRMatrix) -> Result<String> {
        let count = matrix.module_count();
        let min_size = self.min_size();
        let real_qr_size = if self.options.shape == ShapeType::Circle {
            min_size as f64 / 2.0_f64.sqrt()
        } else {
            min_size as f64
        };
        let dot_size = self.round_size(real_qr_size / count as f64);

        // Calculate image hiding area if there's an image
        let (hide_x_dots, hide_y_dots) = if self.options.image.is_some() {
            self.calculate_image_hide_area(count, dot_size)
        } else {
            (0, 0)
        };

        let mut defs = String::new();
        // Rough upper bound: ~40 bytes of path data per dark module
        let mut elements = String::with_capacity(count * count * 20 + 2048);

        self.render_background(&mut defs, &mut elements);
        self.render_dots(&mut defs, &mut elements, matrix, count, dot_size, hide_x_dots, hide_y_dots);
        self.render_corners(&mut defs, &mut elements, count, dot_size);

        if let Some(ref image_data) = self.options.image {
            self.render_image(&mut elements, count, dot_size, hide_x_dots, hide_y_dots, image_data);
        }

        let shape_rendering = if self.options.dots_options.round_size {
            ""
        } else {
            r#" shape-rendering="crispEdges""#
        };

        let mut svg = String::with_capacity(defs.len() + elements.len() + 512);
        let _ = write!(
            svg,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{w}" height="{h}" viewBox="0 0 {w} {h}"{shape_rendering}>
<defs>
{defs}</defs>
{elements}</svg>"#,
            w = self.options.width,
            h = self.options.height,
        );

        Ok(svg)
    }

    fn render_background(&self, defs: &mut String, elements: &mut String) {
        let bg = &self.options.background_options;
        let name = format!("background-color-{}", self.instance_id);

        let (width, height) = if bg.round > 0.0 {
            let size = self.options.width.min(self.options.height);
            (size, size)
        } else {
            (self.options.width, self.options.height)
        };

        let x = self.round_size((self.options.width - width) as f64 / 2.0);
        let y = self.round_size((self.options.height - height) as f64 / 2.0);

        let rx = if bg.round > 0.0 {
            (height as f64 / 2.0) * bg.round
        } else {
            0.0
        };

        let fill = self.create_color(
            defs,
            bg.gradient.as_ref(),
            &bg.color,
            0.0,
            0.0,
            0.0,
            self.options.height as f64,
            self.options.width as f64,
            &name,
        );

        let _ = write!(
            elements,
            r#"<rect x="{}" y="{}" width="{}" height="{}""#,
            Num(x),
            Num(y),
            width,
            height
        );
        if rx > 0.0 {
            let _ = write!(elements, r#" rx="{}""#, Num(rx));
        }
        let _ = writeln!(elements, r#" fill="{}"/>"#, fill);
    }

    #[allow(clippy::too_many_arguments)]
    fn render_dots(
        &self,
        defs: &mut String,
        elements: &mut String,
        matrix: &QRMatrix,
        count: usize,
        dot_size: f64,
        hide_x_dots: usize,
        hide_y_dots: usize,
    ) {
        let x_beginning = self.round_size((self.options.width as f64 - count as f64 * dot_size) / 2.0);
        let y_beginning = self.round_size((self.options.height as f64 - count as f64 * dot_size) / 2.0);

        let dot_drawer = QRDot::new(self.options.dots_options.dot_type);
        let name = format!("dot-color-{}", self.instance_id);

        let fill = self.create_color(
            defs,
            self.options.dots_options.gradient.as_ref(),
            &self.options.dots_options.color,
            0.0,
            0.0,
            0.0,
            self.options.height as f64,
            self.options.width as f64,
            &name,
        );

        // All dots go into one path. Dots never overlap, so the union is
        // filled exactly once: no seams and uniform alpha.
        let _ = write!(elements, r#"<path fill="{}" d=""#, fill);

        for row in 0..count {
            for col in 0..count {
                if !self.should_draw_dot(row, col, count, hide_x_dots, hide_y_dots) {
                    continue;
                }

                if !matrix.is_dark(row, col) {
                    continue;
                }

                let x = x_beginning + col as f64 * dot_size;
                let y = y_beginning + row as f64 * dot_size;

                let neighbor_fn = |x_offset: i32, y_offset: i32| -> bool {
                    let new_col = col as i32 + x_offset;
                    let new_row = row as i32 + y_offset;
                    if new_col < 0 || new_row < 0 || new_col >= count as i32 || new_row >= count as i32
                    {
                        return false;
                    }
                    if !self.should_draw_dot(
                        new_row as usize,
                        new_col as usize,
                        count,
                        hide_x_dots,
                        hide_y_dots,
                    ) {
                        return false;
                    }
                    matrix.is_dark(new_row as usize, new_col as usize)
                };

                dot_drawer.push_path(elements, x, y, dot_size, Some(&neighbor_fn));
            }
        }

        // Handle circle shape with fake edge dots
        if self.options.shape == ShapeType::Circle {
            self.render_circle_edge_dots(elements, matrix, count, dot_size, x_beginning, y_beginning, &dot_drawer);
        }

        elements.push_str("\"/>\n");
    }

    #[allow(clippy::too_many_arguments)]
    fn render_circle_edge_dots(
        &self,
        out: &mut String,
        matrix: &QRMatrix,
        count: usize,
        dot_size: f64,
        x_beginning: f64,
        y_beginning: f64,
        dot_drawer: &QRDot,
    ) {
        let min_size = self.min_size() as f64;
        let additional_dots = self.round_size((min_size / dot_size - count as f64) / 2.0) as usize;
        let fake_count = count + additional_dots * 2;
        let x_fake_beginning = x_beginning - additional_dots as f64 * dot_size;
        let y_fake_beginning = y_beginning - additional_dots as f64 * dot_size;
        let center = fake_count as f64 / 2.0;

        let mut fake_matrix = vec![vec![0u8; fake_count]; fake_count];

        for (row, fake_row) in fake_matrix.iter_mut().enumerate() {
            for (col, cell) in fake_row.iter_mut().enumerate() {
                // Skip inner area
                if row >= additional_dots.saturating_sub(1)
                    && row <= fake_count - additional_dots
                    && col >= additional_dots.saturating_sub(1)
                    && col <= fake_count - additional_dots
                {
                    continue;
                }

                // Skip outside circle
                let dist = ((row as f64 - center).powi(2) + (col as f64 - center).powi(2)).sqrt();
                if dist > center {
                    continue;
                }

                // Get random dots from QR code
                let source_col = if col < 2 * additional_dots {
                    col
                } else if col >= count {
                    col.wrapping_sub(2 * additional_dots)
                } else {
                    col.wrapping_sub(additional_dots)
                };
                let source_row = if row < 2 * additional_dots {
                    row
                } else if row >= count {
                    row.wrapping_sub(2 * additional_dots)
                } else {
                    row.wrapping_sub(additional_dots)
                };

                if source_row < count && source_col < count && matrix.is_dark(source_row, source_col) {
                    *cell = 1;
                }
            }
        }

        for row in 0..fake_count {
            for col in 0..fake_count {
                if fake_matrix[row][col] == 0 {
                    continue;
                }

                let x = x_fake_beginning + col as f64 * dot_size;
                let y = y_fake_beginning + row as f64 * dot_size;

                let neighbor_fn = |x_offset: i32, y_offset: i32| -> bool {
                    let new_col = col as i32 + x_offset;
                    let new_row = row as i32 + y_offset;
                    if new_col < 0 || new_row < 0 || new_col >= fake_count as i32 || new_row >= fake_count as i32 {
                        return false;
                    }
                    fake_matrix[new_row as usize][new_col as usize] == 1
                };

                dot_drawer.push_path(out, x, y, dot_size, Some(&neighbor_fn));
            }
        }

    }


    fn render_corners(&self, defs: &mut String, elements: &mut String, count: usize, dot_size: f64) {
        let x_beginning = self.round_size((self.options.width as f64 - count as f64 * dot_size) / 2.0);
        let y_beginning = self.round_size((self.options.height as f64 - count as f64 * dot_size) / 2.0);

        let corners_square_size = dot_size * 7.0;
        let corners_dot_size = dot_size * 3.0;

        let square_drawer = QRCornerSquare::new(self.options.corners_square_options.square_type);
        let dot_drawer = QRCornerDot::new(self.options.corners_dot_options.dot_type);

        // Three corners: top-left, top-right, bottom-left
        let corner_positions = [(0, 0, 0.0), (1, 0, PI / 2.0), (0, 1, -PI / 2.0)];

        for (column, row, rotation) in corner_positions {
            let x = x_beginning + column as f64 * dot_size * (count - 7) as f64;
            let y = y_beginning + row as f64 * dot_size * (count - 7) as f64;

            // Corner square (a ring, so even-odd fill)
            let sq = &self.options.corners_square_options;
            let name = format!("corners-square-color-{}-{}-{}", column, row, self.instance_id);
            let fill = self.create_color(
                defs,
                sq.gradient.as_ref(),
                &sq.color,
                rotation,
                x,
                y,
                corners_square_size,
                corners_square_size,
                &name,
            );
            let _ = write!(elements, r#"<path fill="{}" fill-rule="evenodd" d=""#, fill);
            square_drawer.push_path(elements, x, y, corners_square_size, rotation);
            elements.push_str("\"/>\n");

            // Corner dot
            let (dx, dy) = (x + dot_size * 2.0, y + dot_size * 2.0);
            let dot = &self.options.corners_dot_options;
            let name = format!("corners-dot-color-{}-{}-{}", column, row, self.instance_id);
            let fill = self.create_color(
                defs,
                dot.gradient.as_ref(),
                &dot.color,
                rotation,
                dx,
                dy,
                corners_dot_size,
                corners_dot_size,
                &name,
            );
            let _ = write!(elements, r#"<path fill="{}" d=""#, fill);
            dot_drawer.push_path(elements, dx, dy, corners_dot_size, rotation);
            elements.push_str("\"/>\n");
        }
    }

    fn render_image(
        &self,
        out: &mut String,
        count: usize,
        dot_size: f64,
        hide_x_dots: usize,
        hide_y_dots: usize,
        image_data: &[u8],
    ) {
        let x_beginning = self.round_size((self.options.width as f64 - count as f64 * dot_size) / 2.0);
        let y_beginning = self.round_size((self.options.height as f64 - count as f64 * dot_size) / 2.0);

        let width = hide_x_dots as f64 * dot_size;
        let height = hide_y_dots as f64 * dot_size;

        let margin = self.options.image_options.margin as f64;
        let dx = x_beginning + self.round_size(margin + (count as f64 * dot_size - width) / 2.0);
        let dy = y_beginning + self.round_size(margin + (count as f64 * dot_size - height) / 2.0);
        let dw = width - margin * 2.0;
        let dh = height - margin * 2.0;

        // Encode image as base64 data URL
        let base64_data = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, image_data);

        // Detect mime type from image data
        let mime_type = if image_data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            "image/png"
        } else if image_data.starts_with(&[0xFF, 0xD8]) {
            "image/jpeg"
        } else if image_data.starts_with(b"RIFF") && image_data.len() > 12 && &image_data[8..12] == b"WEBP" {
            "image/webp"
        } else {
            "image/png" // Default
        };

        // Plain `href` (SVG 2) avoids embedding the base64 payload twice
        let _ = writeln!(
            out,
            r#"<image href="data:{};base64,{}" x="{}" y="{}" width="{}" height="{}"/>"#,
            mime_type,
            base64_data,
            Num(dx),
            Num(dy),
            Num(dw),
            Num(dh)
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn create_color(
        &self,
        defs: &mut String,
        gradient: Option<&Gradient>,
        color: &Color,
        additional_rotation: f64,
        x: f64,
        y: f64,
        height: f64,
        width: f64,
        name: &str,
    ) -> String {

        if let Some(grad) = gradient {
            let size = width.max(height);

            match grad.gradient_type {
                GradientType::Radial => {
                    let cx = x + width / 2.0;
                    let cy = y + height / 2.0;
                    let r = size / 2.0;

                    defs.push_str(&format!(
                        r#"<radialGradient id="{}" gradientUnits="userSpaceOnUse" fx="{}" fy="{}" cx="{}" cy="{}" r="{}">
"#,
                        name, cx, cy, cx, cy, r
                    ));

                    for stop in &grad.color_stops {
                        defs.push_str(&format!(
                            r#"<stop offset="{}%" stop-color="{}"/>
"#,
                            stop.offset * 100.0,
                            stop.color.to_hex()
                        ));
                    }

                    defs.push_str("</radialGradient>\n");
                }
                GradientType::Linear => {
                    let rotation = (grad.rotation + additional_rotation) % (2.0 * PI);
                    let positive_rotation = (rotation + 2.0 * PI) % (2.0 * PI);

                    let (mut x0, mut y0, mut x1, mut y1) = (
                        x + width / 2.0,
                        y + height / 2.0,
                        x + width / 2.0,
                        y + height / 2.0,
                    );

                    if (0.0..=0.25 * PI).contains(&positive_rotation)
                        || (positive_rotation > 1.75 * PI && positive_rotation <= 2.0 * PI)
                    {
                        x0 -= width / 2.0;
                        y0 -= (height / 2.0) * rotation.tan();
                        x1 += width / 2.0;
                        y1 += (height / 2.0) * rotation.tan();
                    } else if positive_rotation > 0.25 * PI && positive_rotation <= 0.75 * PI {
                        y0 -= height / 2.0;
                        x0 -= (width / 2.0) / rotation.tan();
                        y1 += height / 2.0;
                        x1 += (width / 2.0) / rotation.tan();
                    } else if positive_rotation > 0.75 * PI && positive_rotation <= 1.25 * PI {
                        x0 += width / 2.0;
                        y0 += (height / 2.0) * rotation.tan();
                        x1 -= width / 2.0;
                        y1 -= (height / 2.0) * rotation.tan();
                    } else if positive_rotation > 1.25 * PI && positive_rotation <= 1.75 * PI {
                        y0 += height / 2.0;
                        x0 += (width / 2.0) / rotation.tan();
                        y1 -= height / 2.0;
                        x1 -= (width / 2.0) / rotation.tan();
                    }

                    defs.push_str(&format!(
                        r#"<linearGradient id="{}" gradientUnits="userSpaceOnUse" x1="{}" y1="{}" x2="{}" y2="{}">
"#,
                        name,
                        x0.round(),
                        y0.round(),
                        x1.round(),
                        y1.round()
                    ));

                    for stop in &grad.color_stops {
                        defs.push_str(&format!(
                            r#"<stop offset="{}%" stop-color="{}"/>
"#,
                            stop.offset * 100.0,
                            stop.color.to_hex()
                        ));
                    }

                    defs.push_str("</linearGradient>\n");
                }
            }

            format!("url(#{})", name)
        } else {
            color.to_hex()
        }
    }

    fn should_draw_dot(
        &self,
        row: usize,
        col: usize,
        count: usize,
        hide_x_dots: usize,
        hide_y_dots: usize,
    ) -> bool {
        // Hide dots behind image
        if self.options.image_options.hide_background_dots && self.options.image.is_some() {
            let x_start = (count - hide_x_dots) / 2;
            let x_end = (count + hide_x_dots) / 2;
            let y_start = (count - hide_y_dots) / 2;
            let y_end = (count + hide_y_dots) / 2;

            if row >= y_start && row < y_end && col >= x_start && col < x_end {
                return false;
            }
        }

        // Skip corner squares (finder patterns)
        // Top-left
        if row < 7 && col < 7
            && (SQUARE_MASK[row][col] == 1 || DOT_MASK[row][col] == 1) {
                return false;
            }

        // Top-right
        if row < 7 && col >= count - 7 {
            let local_col = col - (count - 7);
            if SQUARE_MASK[row][local_col] == 1 || DOT_MASK[row][local_col] == 1 {
                return false;
            }
        }

        // Bottom-left
        if row >= count - 7 && col < 7 {
            let local_row = row - (count - 7);
            if SQUARE_MASK[local_row][col] == 1 || DOT_MASK[local_row][col] == 1 {
                return false;
            }
        }

        true
    }

    fn calculate_image_hide_area(&self, count: usize, dot_size: f64) -> (usize, usize) {
        // Calculate based on error correction level and image size
        let error_correction_percent = self.options.qr_options.error_correction_level.percentage();
        let cover_level = self.options.image_options.image_size * error_correction_percent;
        let max_hidden_dots = (cover_level * (count * count) as f64).floor() as usize;
        let max_hidden_axis_dots = count.saturating_sub(14);

        // Use the image's real aspect ratio; fall back to 1:1 if it can't be decoded
        let (img_width, img_height) = self
            .options
            .image
            .as_deref()
            .and_then(|data| {
                image::ImageReader::new(std::io::Cursor::new(data))
                    .with_guessed_format()
                    .ok()?
                    .into_dimensions()
                    .ok()
            })
            .unwrap_or((1, 1));

        let result = calculate_image_size(
            img_width,
            img_height,
            max_hidden_dots,
            max_hidden_axis_dots,
            dot_size,
        );

        (result.hide_x_dots, result.hide_y_dots)
    }

    /// Smallest canvas side minus margins (saturating, so oversized margins can't underflow).
    fn min_size(&self) -> u32 {
        self.options
            .width
            .min(self.options.height)
            .saturating_sub(self.options.margin.saturating_mul(2))
    }

    fn round_size(&self, value: f64) -> f64 {
        if self.options.dots_options.round_size {
            value.floor()
        } else {
            value
        }
    }
}
