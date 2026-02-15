//! Background styling options.

use super::{Color, Gradient};

/// Options for styling QR code background.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BackgroundOptions {
    /// Solid color for background (ignored if gradient is set).
    pub color: Color,
    /// Optional gradient for background.
    pub gradient: Option<Gradient>,
    /// Corner radius ratio (0.0 to 1.0, where 0.5 = fully rounded).
    pub round: f64,
}

impl Default for BackgroundOptions {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            gradient: None,
            round: 0.0,
        }
    }
}

impl BackgroundOptions {
    /// Create new background options with a specific color.
    pub fn new(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// Create transparent background.
    pub fn transparent() -> Self {
        Self {
            color: Color::TRANSPARENT,
            gradient: None,
            round: 0.0,
        }
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

    /// Set the corner radius ratio.
    pub fn with_round(mut self, round: f64) -> Self {
        self.round = round.clamp(0.0, 0.5);
        self
    }
}
