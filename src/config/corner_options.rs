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
    /// Ignore `color` and `gradient` and paint with the dots' color, like
    /// JS qr-code-styling does when neither is set.
    #[cfg_attr(feature = "serde", serde(default))]
    pub inherit_color: bool,
}

impl Default for CornersSquareOptions {
    fn default() -> Self {
        Self {
            square_type: CornerSquareType::Square,
            color: Color::BLACK,
            gradient: None,
            inherit_color: false,
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

    /// Paint like JS qr-code-styling when no color or gradient is given
    /// (see [`inherit_color`](Self::inherit_color)).
    pub fn with_inherited_color(mut self) -> Self {
        self.inherit_color = true;
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
    /// Ignore `color` and `gradient` and paint with the corner squares'
    /// color (or the dots' color, if the squares inherit too), like JS
    /// qr-code-styling does when neither is set.
    #[cfg_attr(feature = "serde", serde(default))]
    pub inherit_color: bool,
}

impl Default for CornersDotOptions {
    fn default() -> Self {
        Self {
            dot_type: CornerDotType::Dot,
            color: Color::BLACK,
            gradient: None,
            inherit_color: false,
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

    /// Paint like JS qr-code-styling when no color or gradient is given
    /// (see [`inherit_color`](Self::inherit_color)).
    pub fn with_inherited_color(mut self) -> Self {
        self.inherit_color = true;
        self
    }
}
