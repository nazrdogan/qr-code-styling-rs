//! QR corner square drawer implementation.

use crate::figures::traits::{rotate_transform, svg_path};
use crate::types::CornerSquareType;

/// QR code corner square drawer.
pub struct QRCornerSquare {
    square_type: CornerSquareType,
}

impl QRCornerSquare {
    /// Create a new corner square drawer with the specified type.
    pub fn new(square_type: CornerSquareType) -> Self {
        Self { square_type }
    }

    /// Draw the corner square.
    pub fn draw(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        match self.square_type {
            CornerSquareType::Square => self.draw_square(x, y, size, rotation),
            CornerSquareType::Dot => self.draw_dot(x, y, size, rotation),
            CornerSquareType::ExtraRounded => self.draw_extra_rounded(x, y, size, rotation),
        }
    }

    /// Draw basic dot (ring) shape.
    fn basic_dot(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        let dot_size = size / 7.0;
        let half_size = size / 2.0;
        let inner_radius = half_size - dot_size;

        // Ring shape using clip-rule evenodd
        let d = format!(
            "M {} {} a {} {} 0 1 0 0.1 0 z m 0 {} a {} {} 0 1 1 -0.1 0 Z",
            x + half_size,
            y,
            half_size,
            half_size,
            dot_size,
            inner_radius,
            inner_radius
        );

        svg_path(&d, Some("evenodd"), transform.as_deref())
    }

    /// Draw basic square shape with hollow center.
    fn basic_square(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        let dot_size = size / 7.0;

        // Outer square + inner square (hollow)
        let d = format!(
            "M {} {} v {} h {} v {} z M {} {} h {} v {} h {} z",
            x, y,
            size, size, -size,
            x + dot_size, y + dot_size,
            size - 2.0 * dot_size,
            size - 2.0 * dot_size,
            -(size - 2.0 * dot_size)
        );

        svg_path(&d, Some("evenodd"), transform.as_deref())
    }

    /// Draw extra-rounded shape.
    fn basic_extra_rounded(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        let dot_size = size / 7.0;

        // Outer rounded path
        let outer = format!(
            "M {} {} v {} a {} {} 0 0 0 {} {} h {} a {} {} 0 0 0 {} {} v {} a {} {} 0 0 0 {} {} h {} a {} {} 0 0 0 {} {}",
            x, y + 2.5 * dot_size,
            2.0 * dot_size,
            2.5 * dot_size, 2.5 * dot_size, 2.5 * dot_size, 2.5 * dot_size,
            2.0 * dot_size,
            2.5 * dot_size, 2.5 * dot_size, 2.5 * dot_size, -2.5 * dot_size,
            -2.0 * dot_size,
            2.5 * dot_size, 2.5 * dot_size, -2.5 * dot_size, -2.5 * dot_size,
            -2.0 * dot_size,
            2.5 * dot_size, 2.5 * dot_size, -2.5 * dot_size, 2.5 * dot_size
        );

        // Inner rounded path
        let inner = format!(
            "M {} {} h {} a {} {} 0 0 1 {} {} v {} a {} {} 0 0 1 {} {} h {} a {} {} 0 0 1 {} {} v {} a {} {} 0 0 1 {} {}",
            x + 2.5 * dot_size, y + dot_size,
            2.0 * dot_size,
            1.5 * dot_size, 1.5 * dot_size, 1.5 * dot_size, 1.5 * dot_size,
            2.0 * dot_size,
            1.5 * dot_size, 1.5 * dot_size, -1.5 * dot_size, 1.5 * dot_size,
            -2.0 * dot_size,
            1.5 * dot_size, 1.5 * dot_size, -1.5 * dot_size, -1.5 * dot_size,
            -2.0 * dot_size,
            1.5 * dot_size, 1.5 * dot_size, 1.5 * dot_size, -1.5 * dot_size
        );

        let d = format!("{} {}", outer, inner);
        svg_path(&d, Some("evenodd"), transform.as_deref())
    }

    fn draw_dot(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        self.basic_dot(x, y, size, rotation)
    }

    fn draw_square(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        self.basic_square(x, y, size, rotation)
    }

    fn draw_extra_rounded(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        self.basic_extra_rounded(x, y, size, rotation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_square() {
        let drawer = QRCornerSquare::new(CornerSquareType::Square);
        let svg = drawer.draw(0.0, 0.0, 70.0, 0.0);
        assert!(svg.contains("path"));
        assert!(svg.contains("evenodd"));
    }

    #[test]
    fn test_draw_dot() {
        let drawer = QRCornerSquare::new(CornerSquareType::Dot);
        let svg = drawer.draw(0.0, 0.0, 70.0, 0.0);
        assert!(svg.contains("path"));
        assert!(svg.contains("evenodd"));
    }

    #[test]
    fn test_draw_extra_rounded() {
        let drawer = QRCornerSquare::new(CornerSquareType::ExtraRounded);
        let svg = drawer.draw(0.0, 0.0, 70.0, 0.0);
        assert!(svg.contains("path"));
    }
}
