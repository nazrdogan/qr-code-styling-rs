//! QR corner dot drawer implementation.

use crate::figures::traits::{rotate_transform, svg_circle, svg_rect};
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

    /// Draw the corner dot.
    pub fn draw(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        match self.dot_type {
            CornerDotType::Dot => self.draw_dot(x, y, size, rotation),
            CornerDotType::Square => self.draw_square(x, y, size, rotation),
        }
    }

    /// Draw basic dot (circle).
    fn basic_dot(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        svg_circle(
            x + size / 2.0,
            y + size / 2.0,
            size / 2.0,
            transform.as_deref(),
        )
    }

    /// Draw basic square.
    fn basic_square(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        svg_rect(x, y, size, size, transform.as_deref())
    }

    fn draw_dot(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        self.basic_dot(x, y, size, rotation)
    }

    fn draw_square(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        self.basic_square(x, y, size, rotation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_dot() {
        let drawer = QRCornerDot::new(CornerDotType::Dot);
        let svg = drawer.draw(0.0, 0.0, 30.0, 0.0);
        assert!(svg.contains("circle"));
        assert!(svg.contains("r=\"15\""));
    }

    #[test]
    fn test_draw_square() {
        let drawer = QRCornerDot::new(CornerDotType::Square);
        let svg = drawer.draw(0.0, 0.0, 30.0, 0.0);
        assert!(svg.contains("rect"));
        assert!(svg.contains("width=\"30\""));
    }
}
