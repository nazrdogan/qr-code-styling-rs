//! QR dot drawer implementation.

use std::f64::consts::PI;

use crate::figures::traits::{push_circle, push_square, svg_path, PathBuilder};
use crate::types::DotType;

/// QR code dot drawer.
pub struct QRDot {
    dot_type: DotType,
}

impl QRDot {
    /// Create a new dot drawer with the specified type.
    pub fn new(dot_type: DotType) -> Self {
        Self { dot_type }
    }

    /// Draw a basic dot (circle).
    fn basic_dot(&self, out: &mut String, x: f64, y: f64, size: f64) {
        push_circle(out, x, y, size);
    }

    /// Draw a basic square.
    fn basic_square(&self, out: &mut String, x: f64, y: f64, size: f64) {
        push_square(out, x, y, size);
    }

    /// Draw a side-rounded shape (if rotation === 0, right side is rounded).
    fn basic_side_rounded(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        let half = size / 2.0;
        PathBuilder::new(out, x, y, size, rotation)
            .move_to(x, y)
            .v(size)
            .h(half)
            .arc_by(half, false, false, 0.0, -size)
            .close();
    }

    /// Draw a corner-rounded shape (if rotation === 0, top right corner is rounded).
    fn basic_corner_rounded(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        let half = size / 2.0;
        PathBuilder::new(out, x, y, size, rotation)
            .move_to(x, y)
            .v(size)
            .h(size)
            .v(-half)
            .arc_by(half, false, false, -half, -half)
            .close();
    }

    /// Draw an extra-rounded corner shape.
    fn basic_corner_extra_rounded(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        PathBuilder::new(out, x, y, size, rotation)
            .move_to(x, y)
            .v(size)
            .h(size)
            .arc_by(size, false, false, -size, -size)
            .close();
    }

    /// Draw corners-rounded shape (left bottom and right top corners are rounded).
    fn basic_corners_rounded(&self, out: &mut String, x: f64, y: f64, size: f64, rotation: f64) {
        let half = size / 2.0;
        PathBuilder::new(out, x, y, size, rotation)
            .move_to(x, y)
            .v(half)
            .arc_by(half, false, false, half, half)
            .h(half)
            .v(-half)
            .arc_by(half, false, false, -half, -half)
            .close();
    }

    /// Draw the dot and return an SVG `<path>` element.
    pub fn draw<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool,
    {
        let mut d = String::new();
        self.push_path(&mut d, x, y, size, get_neighbor);
        svg_path(&d, None, None)
    }

    /// Append this dot's path data to `out`, so many dots can share one `<path>`.
    pub fn push_path<F>(&self, out: &mut String, x: f64, y: f64, size: f64, get_neighbor: Option<F>)
    where
        F: Fn(i32, i32) -> bool,
    {
        match self.dot_type {
            DotType::Dots => self.basic_dot(out, x, y, size),
            DotType::Square => self.basic_square(out, x, y, size),
            DotType::Rounded => self.draw_rounded(out, x, y, size, get_neighbor),
            DotType::ExtraRounded => self.draw_extra_rounded(out, x, y, size, get_neighbor),
            DotType::Classy => self.draw_classy(out, x, y, size, get_neighbor),
            DotType::ClassyRounded => self.draw_classy_rounded(out, x, y, size, get_neighbor),
        }
    }

    /// Draw rounded type based on neighbors.
    fn draw_rounded<F>(&self, out: &mut String, x: f64, y: f64, size: f64, get_neighbor: Option<F>)
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_dot(out, x, y, size);
        }

        if count > 2 || (left == 1 && right == 1) || (top == 1 && bottom == 1) {
            return self.basic_square(out, x, y, size);
        }

        if count == 2 {
            let rotation = if left == 1 && top == 1 {
                PI / 2.0
            } else if top == 1 && right == 1 {
                PI
            } else if right == 1 && bottom == 1 {
                -PI / 2.0
            } else {
                0.0
            };
            return self.basic_corner_rounded(out, x, y, size, rotation);
        }

        // count == 1
        let rotation = if top == 1 {
            PI / 2.0
        } else if right == 1 {
            PI
        } else if bottom == 1 {
            -PI / 2.0
        } else {
            0.0
        };
        self.basic_side_rounded(out, x, y, size, rotation)
    }

    /// Draw extra-rounded type based on neighbors.
    fn draw_extra_rounded<F>(&self, out: &mut String, x: f64, y: f64, size: f64, get_neighbor: Option<F>)
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_dot(out, x, y, size);
        }

        if count > 2 || (left == 1 && right == 1) || (top == 1 && bottom == 1) {
            return self.basic_square(out, x, y, size);
        }

        if count == 2 {
            let rotation = if left == 1 && top == 1 {
                PI / 2.0
            } else if top == 1 && right == 1 {
                PI
            } else if right == 1 && bottom == 1 {
                -PI / 2.0
            } else {
                0.0
            };
            return self.basic_corner_extra_rounded(out, x, y, size, rotation);
        }

        // count == 1
        let rotation = if top == 1 {
            PI / 2.0
        } else if right == 1 {
            PI
        } else if bottom == 1 {
            -PI / 2.0
        } else {
            0.0
        };
        self.basic_side_rounded(out, x, y, size, rotation)
    }

    /// Draw classy type based on neighbors.
    fn draw_classy<F>(&self, out: &mut String, x: f64, y: f64, size: f64, get_neighbor: Option<F>)
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_corners_rounded(out, x, y, size, PI / 2.0);
        }

        if left == 0 && top == 0 {
            return self.basic_corner_rounded(out, x, y, size, -PI / 2.0);
        }

        if right == 0 && bottom == 0 {
            return self.basic_corner_rounded(out, x, y, size, PI / 2.0);
        }

        self.basic_square(out, x, y, size)
    }

    /// Draw classy-rounded type based on neighbors.
    fn draw_classy_rounded<F>(&self, out: &mut String, x: f64, y: f64, size: f64, get_neighbor: Option<F>)
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_corners_rounded(out, x, y, size, PI / 2.0);
        }

        if left == 0 && top == 0 {
            return self.basic_corner_extra_rounded(out, x, y, size, -PI / 2.0);
        }

        if right == 0 && bottom == 0 {
            return self.basic_corner_extra_rounded(out, x, y, size, PI / 2.0);
        }

        self.basic_square(out, x, y, size)
    }

    /// Get neighbor states.
    fn get_neighbors<F>(&self, get_neighbor: &Option<F>) -> (u8, u8, u8, u8)
    where
        F: Fn(i32, i32) -> bool,
    {
        match get_neighbor {
            Some(f) => (
                if f(-1, 0) { 1 } else { 0 },
                if f(1, 0) { 1 } else { 0 },
                if f(0, -1) { 1 } else { 0 },
                if f(0, 1) { 1 } else { 0 },
            ),
            None => (0, 0, 0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_square() {
        let drawer = QRDot::new(DotType::Square);
        let svg = drawer.draw(0.0, 0.0, 10.0, None::<fn(i32, i32) -> bool>);
        assert_eq!(svg, r#"<path d="M0 0l10 0l0 10l-10 0z"/>"#);
    }

    #[test]
    fn test_draw_dot() {
        let drawer = QRDot::new(DotType::Dots);
        let svg = drawer.draw(0.0, 0.0, 10.0, None::<fn(i32, i32) -> bool>);
        assert!(svg.contains("a5 5 0 1 0 10 0"));
    }

    #[test]
    fn test_draw_rounded_no_neighbors() {
        let drawer = QRDot::new(DotType::Rounded);
        let svg = drawer.draw(0.0, 0.0, 10.0, None::<fn(i32, i32) -> bool>);
        // With no neighbors, should draw a circle
        assert!(svg.contains("a5 5 0 1 0 10 0"));
    }

    #[test]
    fn test_draw_rounded_with_neighbors() {
        let drawer = QRDot::new(DotType::Rounded);
        let neighbor_fn = |x: i32, _y: i32| x == 1; // right neighbor
        let svg = drawer.draw(0.0, 0.0, 10.0, Some(neighbor_fn));
        // With one neighbor, should draw a side-rounded path
        assert!(svg.contains("path"));
        assert!(!svg.contains("a5 5 0 1 0 10 0"));
    }
}
