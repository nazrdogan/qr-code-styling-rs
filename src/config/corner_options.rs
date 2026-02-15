//! Corner styling options.

use super::{Color, Gradient};
use crate::types::{CornerSquareType, CornerDotType};

/// Options for styling QR code corner squares (finder patterns).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CornersSquareOptions {
    /// The type/style of corner squares.
    pub square_type: CornerSquareType,
    /// Solid color for corner squares (ignored if gradient is set).
    pub color: Color,
    /// Optional gradient for corner squares.
    pub gradient: Option<Gradient>,
}

impl Default for CornersSquareOptions {
    fn default() -> Self {
        Self {
            square_type: CornerSquareType::Square,
            color: Color::BLACK,
            gradient: None,
        }
    }
}

impl CornersSquareOptions {
    /// Create new corner square options with a specific type.
    pub fn new(square_type: CornerSquareType) -> Self {
        Self {
            square_type,
            ..Default::default()
        }
    }

    /// Set the square type.
    pub fn with_type(mut self, square_type: CornerSquareType) -> Self {
        self.square_type = square_type;
        self
    }

    /// Set the color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the gradient.
    pub fn with_gradient(mut self, gradient: Gradient) -> Self {
        self.gradient = Some(gradient);
        self
    }
}

/// Options for styling QR code corner dots (center of finder patterns).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CornersDotOptions {
    /// The type/style of corner dots.
    pub dot_type: CornerDotType,
    /// Solid color for corner dots (ignored if gradient is set).
    pub color: Color,
    /// Optional gradient for corner dots.
    pub gradient: Option<Gradient>,
}

impl Default for CornersDotOptions {
    fn default() -> Self {
        Self {
            dot_type: CornerDotType::Dot,
            color: Color::BLACK,
            gradient: None,
        }
    }
}

impl CornersDotOptions {
    /// Create new corner dot options with a specific type.
    pub fn new(dot_type: CornerDotType) -> Self {
        Self {
            dot_type,
            ..Default::default()
        }
    }

    /// Set the dot type.
    pub fn with_type(mut self, dot_type: CornerDotType) -> Self {
        self.dot_type = dot_type;
        self
    }

    /// Set the color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the gradient.
    pub fn with_gradient(mut self, gradient: Gradient) -> Self {
        self.gradient = Some(gradient);
        self
    }
}
