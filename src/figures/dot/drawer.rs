//! QR dot drawer implementation.

use std::f64::consts::PI;

use crate::figures::traits::{rotate_transform, svg_circle, svg_path, svg_rect};
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
    fn basic_dot(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        svg_circle(
            x + size / 2.0,
            y + size / 2.0,
            size / 2.0,
            transform.as_deref(),
        )
    }

    /// Draw a basic square.
    fn basic_square(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        svg_rect(x, y, size, size, transform.as_deref())
    }

    /// Draw a side-rounded shape (if rotation === 0, right side is rounded).
    fn basic_side_rounded(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        let half = size / 2.0;
        let d = format!(
            "M {} {} v {} h {} a {} {} 0 0 0 0 {}",
            x, y, size, half, half, half, -size
        );
        svg_path(&d, None, transform.as_deref())
    }

    /// Draw a corner-rounded shape (if rotation === 0, top right corner is rounded).
    fn basic_corner_rounded(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        let half = size / 2.0;
        let d = format!(
            "M {} {} v {} h {} v {} a {} {} 0 0 0 {} {}",
            x, y, size, size, -half, half, half, -half, -half
        );
        svg_path(&d, None, transform.as_deref())
    }

    /// Draw an extra-rounded corner shape.
    fn basic_corner_extra_rounded(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        let d = format!(
            "M {} {} v {} h {} a {} {} 0 0 0 {} {}",
            x, y, size, size, size, size, -size, -size
        );
        svg_path(&d, None, transform.as_deref())
    }

    /// Draw corners-rounded shape (left bottom and right top corners are rounded).
    fn basic_corners_rounded(&self, x: f64, y: f64, size: f64, rotation: f64) -> String {
        let transform = rotate_transform(x, y, size, rotation);
        let half = size / 2.0;
        let d = format!(
            "M {} {} v {} a {} {} 0 0 0 {} {} h {} v {} a {} {} 0 0 0 {} {}",
            x, y, half, half, half, half, half, half, -half, half, half, -half, -half
        );
        svg_path(&d, None, transform.as_deref())
    }

    /// Draw the dot and return SVG element string.
    pub fn draw<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool,
    {
        match self.dot_type {
            DotType::Dots => self.basic_dot(x, y, size, 0.0),
            DotType::Square => self.basic_square(x, y, size, 0.0),
            DotType::Rounded => self.draw_rounded(x, y, size, get_neighbor),
            DotType::ExtraRounded => self.draw_extra_rounded(x, y, size, get_neighbor),
            DotType::Classy => self.draw_classy(x, y, size, get_neighbor),
            DotType::ClassyRounded => self.draw_classy_rounded(x, y, size, get_neighbor),
        }
    }

    /// Draw rounded type based on neighbors.
    fn draw_rounded<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_dot(x, y, size, 0.0);
        }

        if count > 2 || (left == 1 && right == 1) || (top == 1 && bottom == 1) {
            return self.basic_square(x, y, size, 0.0);
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
            return self.basic_corner_rounded(x, y, size, rotation);
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
        self.basic_side_rounded(x, y, size, rotation)
    }

    /// Draw extra-rounded type based on neighbors.
    fn draw_extra_rounded<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_dot(x, y, size, 0.0);
        }

        if count > 2 || (left == 1 && right == 1) || (top == 1 && bottom == 1) {
            return self.basic_square(x, y, size, 0.0);
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
            return self.basic_corner_extra_rounded(x, y, size, rotation);
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
        self.basic_side_rounded(x, y, size, rotation)
    }

    /// Draw classy type based on neighbors.
    fn draw_classy<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_corners_rounded(x, y, size, PI / 2.0);
        }

        if left == 0 && top == 0 {
            return self.basic_corner_rounded(x, y, size, -PI / 2.0);
        }

        if right == 0 && bottom == 0 {
            return self.basic_corner_rounded(x, y, size, PI / 2.0);
        }

        self.basic_square(x, y, size, 0.0)
    }

    /// Draw classy-rounded type based on neighbors.
    fn draw_classy_rounded<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool,
    {
        let (left, right, top, bottom) = self.get_neighbors(&get_neighbor);
        let count = left + right + top + bottom;

        if count == 0 {
            return self.basic_corners_rounded(x, y, size, PI / 2.0);
        }

        if left == 0 && top == 0 {
            return self.basic_corner_extra_rounded(x, y, size, -PI / 2.0);
        }

        if right == 0 && bottom == 0 {
            return self.basic_corner_extra_rounded(x, y, size, PI / 2.0);
        }

        self.basic_square(x, y, size, 0.0)
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
        assert!(svg.contains("rect"));
        assert!(svg.contains("width=\"10\""));
    }

    #[test]
    fn test_draw_dot() {
        let drawer = QRDot::new(DotType::Dots);
        let svg = drawer.draw(0.0, 0.0, 10.0, None::<fn(i32, i32) -> bool>);
        assert!(svg.contains("circle"));
        assert!(svg.contains("r=\"5\""));
    }

    #[test]
    fn test_draw_rounded_no_neighbors() {
        let drawer = QRDot::new(DotType::Rounded);
        let svg = drawer.draw(0.0, 0.0, 10.0, None::<fn(i32, i32) -> bool>);
        // With no neighbors, should draw a circle
        assert!(svg.contains("circle"));
    }

    #[test]
    fn test_draw_rounded_with_neighbors() {
        let drawer = QRDot::new(DotType::Rounded);
        let neighbor_fn = |x: i32, _y: i32| x == 1; // right neighbor
        let svg = drawer.draw(0.0, 0.0, 10.0, Some(neighbor_fn));
        // With one neighbor, should draw a side-rounded path
        assert!(svg.contains("path"));
    }
}
