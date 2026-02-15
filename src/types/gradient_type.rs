//! Gradient type variants.

/// Defines the type of gradient for coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum GradientType {
    /// Linear gradient.
    #[default]
    Linear,
    /// Radial gradient.
    Radial,
}
