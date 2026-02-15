//! Error types for QR code styling library.

use thiserror::Error;

/// Result type alias using QRError.
pub type Result<T> = std::result::Result<T, QRError>;

/// Errors that can occur during QR code generation and styling.
#[derive(Error, Debug)]
pub enum QRError {
    /// No data provided for QR code generation.
    #[error("No data provided for QR code")]
    MissingData,

    /// Data is too large to fit in the specified QR code version.
    #[error("Data too large for QR code: data requires more capacity than available")]
    DataTooLarge,

    /// Invalid QR code version specified.
    #[error("Invalid QR code version: {0}")]
    InvalidVersion(u8),

    /// Canvas dimensions are too small for the QR code.
    #[error("Canvas dimensions too small: {width}x{height}")]
    CanvasTooSmall { width: u32, height: u32 },

    /// Invalid color format provided.
    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    /// Gradient must have at least one color stop.
    #[error("Gradient must have at least one color stop")]
    EmptyGradient,

    /// Failed to load an image.
    #[error("Failed to load image: {0}")]
    ImageLoadError(String),

    /// Failed to encode an image.
    #[error("Failed to encode image: {0}")]
    ImageEncodeError(String),

    /// IO error occurred.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Image processing error.
    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    /// QR code generation error.
    #[error("QR code generation failed: {0}")]
    QRGenerationError(String),

    /// SVG rendering error.
    #[error("SVG rendering error: {0}")]
    SvgError(String),
}
