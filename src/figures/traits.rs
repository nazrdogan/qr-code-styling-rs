//! Traits for figure drawing.

use std::f64::consts::PI;
use std::fmt::{self, Write};

/// Trait for drawing QR code figures.
pub trait FigureDrawer {
    /// Draw the figure and return SVG element string.
    fn draw<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool;
}

/// Legacy type alias for compatibility.
pub type NeighborFn = dyn Fn(i32, i32) -> bool;

/// Formats a coordinate for SVG output: rounded to 3 decimals, trailing zeros trimmed.
#[derive(Clone, Copy)]
pub struct Num(pub f64);

impl fmt::Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = (self.0 * 1000.0).round() / 1000.0;
        // Avoid printing "-0"
        if v == 0.0 {
            f.write_str("0")
        } else {
            write!(f, "{}", v)
        }
    }
}

/// Builds SVG path data for a figure whose local coordinates are rotated
/// around the figure's center. The rotation is baked into the emitted
/// coordinates, so many figures can share one `<path>` element.
pub struct PathBuilder<'a> {
    out: &'a mut String,
    cx: f64,
    cy: f64,
    cos: f64,
    sin: f64,
}

impl<'a> PathBuilder<'a> {
    /// Start a figure occupying the `size`×`size` box at (`x`, `y`), rotated by `rotation` radians.
    pub fn new(out: &'a mut String, x: f64, y: f64, size: f64, rotation: f64) -> Self {
        let (mut sin, mut cos) = rotation.sin_cos();
        // Quarter turns must be exact so adjacent figures line up
        if (sin.round() - sin).abs() < 1e-9 {
            sin = sin.round();
        }
        if (cos.round() - cos).abs() < 1e-9 {
            cos = cos.round();
        }
        Self {
            out,
            cx: x + size / 2.0,
            cy: y + size / 2.0,
            cos,
            sin,
        }
    }

    fn rotate(&self, dx: f64, dy: f64) -> (f64, f64) {
        (dx * self.cos - dy * self.sin, dx * self.sin + dy * self.cos)
    }

    /// Absolute move.
    pub fn move_to(&mut self, x: f64, y: f64) -> &mut Self {
        let (dx, dy) = self.rotate(x - self.cx, y - self.cy);
        let _ = write!(self.out, "M{} {}", Num(self.cx + dx), Num(self.cy + dy));
        self
    }

    /// Relative move.
    pub fn move_by(&mut self, dx: f64, dy: f64) -> &mut Self {
        let (dx, dy) = self.rotate(dx, dy);
        let _ = write!(self.out, "m{} {}", Num(dx), Num(dy));
        self
    }

    /// Relative line.
    pub fn line_by(&mut self, dx: f64, dy: f64) -> &mut Self {
        let (dx, dy) = self.rotate(dx, dy);
        let _ = write!(self.out, "l{} {}", Num(dx), Num(dy));
        self
    }

    /// Relative horizontal line (in local coordinates).
    pub fn h(&mut self, dx: f64) -> &mut Self {
        self.line_by(dx, 0.0)
    }

    /// Relative vertical line (in local coordinates).
    pub fn v(&mut self, dy: f64) -> &mut Self {
        self.line_by(0.0, dy)
    }

    /// Relative circular arc. Rotation preserves the sweep direction.
    pub fn arc_by(&mut self, r: f64, large: bool, sweep: bool, dx: f64, dy: f64) -> &mut Self {
        let (dx, dy) = self.rotate(dx, dy);
        let _ = write!(
            self.out,
            "a{} {} 0 {} {} {} {}",
            Num(r),
            Num(r),
            large as u8,
            sweep as u8,
            Num(dx),
            Num(dy)
        );
        self
    }

    /// Close the current subpath.
    pub fn close(&mut self) -> &mut Self {
        self.out.push('z');
        self
    }
}

/// Append a full circle inscribed in the `size` box at (`x`, `y`).
pub fn push_circle(out: &mut String, x: f64, y: f64, size: f64) {
    let r = size / 2.0;
    PathBuilder::new(out, x, y, size, 0.0)
        .move_to(x, y + r)
        .arc_by(r, true, false, size, 0.0)
        .arc_by(r, true, false, -size, 0.0)
        .close();
}

/// Append an axis-aligned square at (`x`, `y`).
pub fn push_square(out: &mut String, x: f64, y: f64, size: f64) {
    PathBuilder::new(out, x, y, size, 0.0)
        .move_to(x, y)
        .h(size)
        .v(size)
        .h(-size)
        .close();
}

/// Helper to apply rotation transform to SVG element.
pub fn rotate_transform(x: f64, y: f64, size: f64, rotation: f64) -> Option<String> {
    if rotation.abs() < 0.0001 {
        return None;
    }
    let cx = x + size / 2.0;
    let cy = y + size / 2.0;
    let degrees = (180.0 * rotation) / PI;
    Some(format!("rotate({},{},{})", degrees, cx, cy))
}

/// Helper to create SVG circle element.
pub fn svg_circle(cx: f64, cy: f64, r: f64, transform: Option<&str>) -> String {
    match transform {
        Some(t) => format!(
            r#"<circle cx="{}" cy="{}" r="{}" transform="{}"/>"#,
            cx, cy, r, t
        ),
        None => format!(r#"<circle cx="{}" cy="{}" r="{}"/>"#, cx, cy, r),
    }
}

/// Helper to create SVG rect element.
pub fn svg_rect(x: f64, y: f64, width: f64, height: f64, transform: Option<&str>) -> String {
    match transform {
        Some(t) => format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" transform="{}"/>"#,
            x, y, width, height, t
        ),
        None => format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}"/>"#,
            x, y, width, height
        ),
    }
}

/// Helper to create SVG path element.
pub fn svg_path(d: &str, clip_rule: Option<&str>, transform: Option<&str>) -> String {
    let mut attrs = format!(r#"d="{}""#, d);
    if let Some(rule) = clip_rule {
        attrs.push_str(&format!(r#" clip-rule="{}""#, rule));
    }
    if let Some(t) = transform {
        attrs.push_str(&format!(r#" transform="{}""#, t));
    }
    format!(r#"<path {}/>"#, attrs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_format() {
        assert_eq!(Num(3.9215686274509802).to_string(), "3.922");
        assert_eq!(Num(10.0).to_string(), "10");
        assert_eq!(Num(-0.0001).to_string(), "0");
        assert_eq!(Num(-2.5).to_string(), "-2.5");
    }

    #[test]
    fn test_rotation_is_baked_in() {
        // A unit right-pointing line rotated a quarter turn points down
        let mut d = String::new();
        PathBuilder::new(&mut d, 0.0, 0.0, 10.0, PI / 2.0)
            .move_to(0.0, 0.0)
            .h(10.0);
        assert_eq!(d, "M10 0l0 10");
    }
}
