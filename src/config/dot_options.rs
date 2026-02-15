//! Dot styling options.

use super::{Color, Gradient};
use crate::types::DotType;

/// Options for styling QR code dots.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotsOptions {
    /// The type/style of dots.
    pub dot_type: DotType,
    /// Solid color for dots (ignored if gradient is set).
    pub color: Color,
    /// Optional gradient for dots.
    pub gradient: Option<Gradient>,
    /// Whether to round dot sizes to whole pixels.
    pub round_size: bool,
}

impl Default for DotsOptions {
    fn default() -> Self {
        Self {
            dot_type: DotType::Square,
            color: Color::BLACK,
            gradient: None,
            round_size: true,
        }
    }
}

impl DotsOptions {
    /// Create new dots options with a specific type.
    pub fn new(dot_type: DotType) -> Self {
        Self {
            dot_type,
            ..Default::default()
        }
    }

    /// Set the dot type.
    pub fn with_type(mut self, dot_type: DotType) -> Self {
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

    /// Set round_size option.
    pub fn with_round_size(mut self, round_size: bool) -> Self {
        self.round_size = round_size;
        self
    }
}
