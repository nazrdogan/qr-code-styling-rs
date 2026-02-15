//! QR code shape type variants.

/// Defines the overall shape of the QR code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum ShapeType {
    /// Square QR code (default).
    #[default]
    Square,
    /// Circular QR code with fake edge dots.
    Circle,
}
