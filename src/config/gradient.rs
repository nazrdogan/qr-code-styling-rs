//! Gradient configuration for QR code styling.

use super::Color;
use crate::types::GradientType;

/// A color stop in a gradient.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorStop {
    /// Position of the stop (0.0 to 1.0).
    pub offset: f64,
    /// Color at this stop.
    pub color: Color,
}

impl ColorStop {
    /// Create a new color stop.
    pub fn new(offset: f64, color: Color) -> Self {
        Self {
            offset: offset.clamp(0.0, 1.0),
            color,
        }
    }
}

/// Gradient definition for coloring QR code elements.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gradient {
    /// Type of gradient (linear or radial).
    pub gradient_type: GradientType,
    /// Rotation angle in radians (for linear gradients).
    pub rotation: f64,
    /// Color stops defining the gradient.
    pub color_stops: Vec<ColorStop>,
}

impl Gradient {
    /// Create a new linear gradient.
    pub fn linear(color_stops: Vec<ColorStop>) -> Self {
        Self {
            gradient_type: GradientType::Linear,
            rotation: 0.0,
            color_stops,
        }
    }

    /// Create a new linear gradient with rotation.
    pub fn linear_rotated(rotation: f64, color_stops: Vec<ColorStop>) -> Self {
        Self {
            gradient_type: GradientType::Linear,
            rotation,
            color_stops,
        }
    }

    /// Create a new radial gradient.
    pub fn radial(color_stops: Vec<ColorStop>) -> Self {
        Self {
            gradient_type: GradientType::Radial,
            rotation: 0.0,
            color_stops,
        }
    }

    /// Create a simple two-color linear gradient.
    pub fn simple_linear(start: Color, end: Color) -> Self {
        Self::linear(vec![
            ColorStop::new(0.0, start),
            ColorStop::new(1.0, end),
        ])
    }

    /// Create a simple two-color radial gradient.
    pub fn simple_radial(center: Color, edge: Color) -> Self {
        Self::radial(vec![
            ColorStop::new(0.0, center),
            ColorStop::new(1.0, edge),
        ])
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self::simple_linear(Color::BLACK, Color::BLACK)
    }
}
