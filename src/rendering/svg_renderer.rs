//! SVG renderer for QR codes.

use std::borrow::Cow;
use std::f64::consts::PI;

use crate::config::{Color, Gradient, QRCodeStylingOptions};
use crate::core::QRMatrix;
use crate::error::Result;
use crate::figures::{QRCornerDot, QRCornerSquare, QRDot};
use crate::rendering::scene::{Background, ImageItem, Paint, Scene, Shape};
use crate::types::{CornerDotType, CornerSquareType, GradientType, ShapeType};
use crate::utils::calculate_image_size;

/// SVG renderer for QR codes.
///
/// Each layer (background, dots, each corner square and corner dot) is a
/// single filled element, so rasterizers don't need per-layer clip masks.
pub struct SvgRenderer<'a> {
    options: Cow<'a, QRCodeStylingOptions>,
    instance_id: u64,
    js_compatible: bool,
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
            js_compatible: false,
        }
    }

    /// Draw like the JavaScript `qr-code-styling` library where the two
    /// differ: for [`ShapeType::Circle`], the extra ring of dots uses a
    /// rounded center and samples the QR matrix transposed, as JS does.
    pub fn js_compatible(mut self, on: bool) -> Self {
        self.js_compatible = on;
        self
    }

    /// Render the QR code as SVG string.
    pub fn render(&self, matrix: &QRMatrix) -> Result<String> {
        Ok(self.scene(matrix).to_svg())
    }

    /// Compute the backend-independent layout of the QR code.
    pub(crate) fn scene(&self, matrix: &QRMatrix) -> Scene<'_> {
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

        let mut shapes = Vec::with_capacity(7);
        shapes.push(self.render_dots(matrix, count, dot_size, hide_x_dots, hide_y_dots));
        self.render_corners(&mut shapes, count, dot_size);

        let image = self
            .options
            .image
            .as_deref()
            .map(|data| self.render_image(count, dot_size, hide_x_dots, hide_y_dots, data));

        Scene {
            width: self.options.width,
            height: self.options.height,
            crisp_edges: !self.options.dots_options.round_size,
            background: self.render_background(),
            shapes,
            image,
        }
    }

    fn render_background(&self) -> Background {
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

        let paint = self.create_paint(
            bg.gradient.as_ref(),
            &bg.color,
            0.0,
            0.0,
            0.0,
            self.options.height as f64,
            self.options.width as f64,
            &name,
        );

        Background {
            x,
            y,
            width,
            height,
            rx,
            paint,
        }
    }

    fn render_dots(
        &self,
        matrix: &QRMatrix,
        count: usize,
        dot_size: f64,
        hide_x_dots: usize,
        hide_y_dots: usize,
    ) -> Shape {
        let x_beginning = self.round_size((self.options.width as f64 - count as f64 * dot_size) / 2.0);
        let y_beginning = self.round_size((self.options.height as f64 - count as f64 * dot_size) / 2.0);

        let dot_drawer = QRDot::new(self.options.dots_options.dot_type);
        let name = format!("dot-color-{}", self.instance_id);

        let paint = self.create_paint(
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
        // Rough upper bound: ~20 bytes of path data per module
        let mut d = String::with_capacity(count * count * 20);

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

                dot_drawer.push_path(&mut d, x, y, dot_size, Some(&neighbor_fn));
            }
        }

        // Handle circle shape with fake edge dots
        if self.options.shape == ShapeType::Circle {
            self.render_circle_edge_dots(&mut d, matrix, count, dot_size, x_beginning, y_beginning, &dot_drawer);
        }

        Shape {
            paint,
            even_odd: false,
            d,
        }
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
        let center = if self.js_compatible {
            self.round_size(fake_count as f64 / 2.0)
        } else {
            fake_count as f64 / 2.0
        };

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

                // JS calls isDark(col-derived, row-derived), i.e. transposed
                let (r, c) = if self.js_compatible {
                    (source_col, source_row)
                } else {
                    (source_row, source_col)
                };
                if r < count && c < count && matrix.is_dark(r, c) {
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


    fn render_corners(&self, shapes: &mut Vec<Shape>, count: usize, dot_size: f64) {
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
            let paint = self.create_paint(
                sq.gradient.as_ref(),
                &sq.color,
                rotation,
                x,
                y,
                corners_square_size,
                corners_square_size,
                &name,
            );
            let square_paint = if sq.inherit_color { self.dots_paint(&name) } else { paint };
            let mut d = String::new();
            let even_odd = if sq.square_type == CornerSquareType::FromDots {
                self.push_mask_dots(&mut d, &SQUARE_MASK, x, y, dot_size);
                false
            } else {
                square_drawer.push_path(&mut d, x, y, corners_square_size, rotation);
                true
            };
            shapes.push(Shape {
                paint: square_paint.clone(),
                even_odd,
                d,
            });

            // Corner dot
            let (dx, dy) = (x + dot_size * 2.0, y + dot_size * 2.0);
            let dot = &self.options.corners_dot_options;
            let name = format!("corners-dot-color-{}-{}-{}", column, row, self.instance_id);
            let paint = self.create_paint(
                dot.gradient.as_ref(),
                &dot.color,
                rotation,
                dx,
                dy,
                corners_dot_size,
                corners_dot_size,
                &name,
            );
            // An inheriting dot takes the square's paint (JS draws it into
            // the square's clip path), under its own gradient id.
            let paint = if dot.inherit_color { renamed(square_paint, &name) } else { paint };
            let mut d = String::new();
            if dot.dot_type == CornerDotType::FromDots {
                // DOT_MASK already includes the 2-module inset
                self.push_mask_dots(&mut d, &DOT_MASK, x, y, dot_size);
            } else {
                dot_drawer.push_path(&mut d, dx, dy, corners_dot_size, rotation);
            }
            shapes.push(Shape {
                paint,
                even_odd: false,
                d,
            });
        }
    }

    /// The dots' paint (over the whole canvas), under gradient id `name`.
    fn dots_paint(&self, name: &str) -> Paint {
        let dots = &self.options.dots_options;
        self.create_paint(
            dots.gradient.as_ref(),
            &dots.color,
            0.0,
            0.0,
            0.0,
            self.options.height as f64,
            self.options.width as f64,
            name,
        )
    }

    /// Draw each set cell of a 7×7 finder `mask` at (`x`, `y`) as a dot of
    /// the dots' type. Neighbors outside the mask count as empty, so e.g.
    /// rounded dots join along the ring but not into the data area.
    fn push_mask_dots(&self, out: &mut String, mask: &[[u8; 7]; 7], x: f64, y: f64, dot_size: f64) {
        let drawer = QRDot::new(self.options.dots_options.dot_type);
        for (row, cells) in mask.iter().enumerate() {
            for (col, &cell) in cells.iter().enumerate() {
                if cell == 0 {
                    continue;
                }
                let neighbor_fn = |x_offset: i32, y_offset: i32| -> bool {
                    let (r, c) = (row as i32 + y_offset, col as i32 + x_offset);
                    (0..7).contains(&r) && (0..7).contains(&c) && mask[r as usize][c as usize] == 1
                };
                let (dx, dy) = (x + col as f64 * dot_size, y + row as f64 * dot_size);
                drawer.push_path(out, dx, dy, dot_size, Some(&neighbor_fn));
            }
        }
    }

    fn render_image<'d>(
        &self,
        count: usize,
        dot_size: f64,
        hide_x_dots: usize,
        hide_y_dots: usize,
        image_data: &'d [u8],
    ) -> ImageItem<'d> {
        let x_beginning = self.round_size((self.options.width as f64 - count as f64 * dot_size) / 2.0);
        let y_beginning = self.round_size((self.options.height as f64 - count as f64 * dot_size) / 2.0);

        let width = hide_x_dots as f64 * dot_size;
        let height = hide_y_dots as f64 * dot_size;

        let margin = self.options.image_options.margin as f64;
        let dx = x_beginning + self.round_size(margin + (count as f64 * dot_size - width) / 2.0);
        let dy = y_beginning + self.round_size(margin + (count as f64 * dot_size - height) / 2.0);
        let dw = width - margin * 2.0;
        let dh = height - margin * 2.0;

        ImageItem {
            x: dx,
            y: dy,
            width: dw,
            height: dh,
            data: image_data,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn create_paint(
        &self,
        gradient: Option<&Gradient>,
        color: &Color,
        additional_rotation: f64,
        x: f64,
        y: f64,
        height: f64,
        width: f64,
        name: &str,
    ) -> Paint {
        let Some(grad) = gradient else {
            return Paint::Solid(*color);
        };

        let size = width.max(height);
        match grad.gradient_type {
            GradientType::Radial => Paint::Radial {
                id: name.to_string(),
                cx: x + width / 2.0,
                cy: y + height / 2.0,
                r: size / 2.0,
                stops: grad.color_stops.clone(),
            },
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

                Paint::Linear {
                    id: name.to_string(),
                    x1: x0.round(),
                    y1: y0.round(),
                    x2: x1.round(),
                    y2: y1.round(),
                    stops: grad.color_stops.clone(),
                }
            }
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

/// `paint` with its gradient (if any) under the id `name`.
fn renamed(paint: Paint, name: &str) -> Paint {
    match paint {
        Paint::Solid(c) => Paint::Solid(c),
        Paint::Linear { x1, y1, x2, y2, stops, .. } => Paint::Linear { id: name.to_string(), x1, y1, x2, y2, stops },
        Paint::Radial { cx, cy, r, stops, .. } => Paint::Radial { id: name.to_string(), cx, cy, r, stops },
    }
}
