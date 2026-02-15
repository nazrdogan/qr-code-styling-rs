//! QR code dot type variants.

/// Defines the visual style for QR code dots (modules).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum DotType {
    /// Simple square dots (default).
    #[default]
    Square,
    /// Circular dots.
    Dots,
    /// Rounded squares that adapt based on neighbors.
    Rounded,
    /// Classy style with one rounded corner.
    Classy,
    /// Classy style with more rounded edges.
    ClassyRounded,
    /// Extra rounded corner style.
    ExtraRounded,
}

impl DotType {
    /// Returns all available dot types.
    pub fn all() -> &'static [DotType] {
        &[
            DotType::Square,
            DotType::Dots,
            DotType::Rounded,
            DotType::Classy,
            DotType::ClassyRounded,
            DotType::ExtraRounded,
        ]
    }
}
