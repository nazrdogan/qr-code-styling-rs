//! QR corner dot drawer implementation.

use crate::figures::traits::{push_circle, push_square, svg_path};
use crate::types::CornerDotType;

/// QR code corner dot drawer (center of finder patterns).
pub struct QRCornerDot {
    dot_type: CornerDotType,
}

impl QRCornerDot {
    /// Create a new corner dot drawer with the specified type.
    pub fn new(dot_type: CornerDotType) -> Self {
        Self { dot_type }
    }

    /// Draw the corner dot and return an SVG `<path>` element.
    pub fn draw(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let mut d = String::new();
        self.push_path(&mut d, x, y, size, rotation);
        svg_path(&d, None, None)
    }

    /// Append the path data to `out`. Both shapes are symmetric, so rotation
    /// doesn't change them.
    pub fn push_path(&self, out: &mut String, x: f64, y: f64, size: f64, _rotation: f64) {
        match self.dot_type {
            CornerDotType::Dot => push_circle(out, x, y, size),
            CornerDotType::Square => push_square(out, x, y, size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_dot() {
        let drawer = QRCornerDot::new(CornerDotType::Dot);
        let svg = drawer.draw(0.0, 0.0, 30.0, 0.0);
        assert!(svg.contains("a15 15 0 1 0 30 0"));
    }

    #[test]
    fn test_draw_square() {
        let drawer = QRCornerDot::new(CornerDotType::Square);
        let svg = drawer.draw(0.0, 0.0, 30.0, 0.0);
        assert_eq!(svg, r#"<path d="M0 0l30 0l0 30l-30 0z"/>"#);
    }
}
