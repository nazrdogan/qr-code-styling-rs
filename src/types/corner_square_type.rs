//! QR code corner square type variants.

/// Defines the visual style for QR code corner squares (finder patterns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum CornerSquareType {
    /// Square corner pattern (default).
    #[default]
    Square,
    /// Dot/ring corner pattern.
    Dot,
    /// Extra rounded corner pattern.
    ExtraRounded,
    /// Draw the pattern module by module with the dots' type, like JS
    /// qr-code-styling does when the type is not set.
    #[cfg_attr(feature = "serde", serde(rename = "from-dots"))]
    FromDots,
}

impl CornerSquareType {
    /// Returns all available corner square types.
    pub fn all() -> &'static [CornerSquareType] {
        &[
            CornerSquareType::Square,
            CornerSquareType::Dot,
            CornerSquareType::ExtraRounded,
            CornerSquareType::FromDots,
        ]
    }
}
