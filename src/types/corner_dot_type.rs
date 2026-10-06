//! QR code corner dot type variants.

/// Defines the visual style for QR code corner dots (center of finder patterns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum CornerDotType {
    /// Circular dot (default).
    #[default]
    Dot,
    /// Square dot.
    Square,
    /// Draw the pattern module by module with the dots' type, like JS
    /// qr-code-styling does when the type is not set.
    #[cfg_attr(feature = "serde", serde(rename = "from-dots"))]
    FromDots,
}

impl CornerDotType {
    /// Returns all available corner dot types.
    pub fn all() -> &'static [CornerDotType] {
        &[CornerDotType::Dot, CornerDotType::Square, CornerDotType::FromDots]
    }
}
