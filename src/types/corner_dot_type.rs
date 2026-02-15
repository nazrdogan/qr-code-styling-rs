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
}

impl CornerDotType {
    /// Returns all available corner dot types.
    pub fn all() -> &'static [CornerDotType] {
        &[CornerDotType::Dot, CornerDotType::Square]
    }
}
