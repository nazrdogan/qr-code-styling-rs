//! QR corner square drawer implementation.

use crate::figures::traits::PathBuilder;
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

    /// Draw the corner square and return an SVG `<path>` element (even-odd filled).
    pub fn draw(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let mut d = String::new();
        self.push_path(&mut d, x, y, size, rotation);
        format!(r#"<path d="{}" fill-rule="evenodd" clip-rule="evenodd"/>"#, d)
    }

    /// Append the path data to `out`. The shape is a ring, so it must be
    /// filled with `fill-rule="evenodd"`.
    pub fn push_path(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        match self.square_type {
            CornerSquareType::Square => self.basic_square(out, x, y, size, rotation),
            CornerSquareType::Dot => self.basic_dot(out, x, y, size, rotation),
            CornerSquareType::ExtraRounded => self.basic_extra_rounded(out, x, y, size, rotation),
        }
    }

    /// Draw basic dot (ring) shape.
    fn basic_dot(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        let dot_size = size / 7.0;
        let half_size = size / 2.0;
        let inner_radius = half_size - dot_size;

        PathBuilder::new(out, x, y, size, rotation)
            .move_to(x + half_size, y)
            .arc_by(half_size, true, false, 0.1, 0.0)
            .close()
            .move_by(0.0, dot_size)
            .arc_by(inner_radius, true, true, -0.1, 0.0)
            .close();
    }

    /// Draw basic square shape with hollow center.
    fn basic_square(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        let dot_size = size / 7.0;
        let inner = size - 2.0 * dot_size;

        let mut b = PathBuilder::new(out, x, y, size, rotation);
        b.move_to(x, y).v(size).h(size).v(-size).close();
        b.move_to(x + dot_size, y + dot_size).h(inner).v(inner).h(-inner).close();
    }

    /// Draw extra-rounded shape.
    fn basic_extra_rounded(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        let d = size / 7.0;
        let (outer_r, inner_r) = (2.5 * d, 1.5 * d);

        let mut b = PathBuilder::new(out, x, y, size, rotation);
        // Outer rounded path
        b.move_to(x, y + outer_r)
            .v(2.0 * d)
            .arc_by(outer_r, false, false, outer_r, outer_r)
            .h(2.0 * d)
            .arc_by(outer_r, false, false, outer_r, -outer_r)
            .v(-2.0 * d)
            .arc_by(outer_r, false, false, -outer_r, -outer_r)
            .h(-2.0 * d)
            .arc_by(outer_r, false, false, -outer_r, outer_r)
            .close();
        // Inner rounded path
        b.move_to(x + outer_r, y + d)
            .h(2.0 * d)
            .arc_by(inner_r, false, true, inner_r, inner_r)
            .v(2.0 * d)
            .arc_by(inner_r, false, true, -inner_r, inner_r)
            .h(-2.0 * d)
            .arc_by(inner_r, false, true, -inner_r, -inner_r)
            .v(-2.0 * d)
            .arc_by(inner_r, false, true, inner_r, -inner_r)
            .close();
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
