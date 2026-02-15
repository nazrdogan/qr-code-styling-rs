//! Configuration types for QR code styling.

mod color;
mod gradient;
mod dot_options;
mod corner_options;
mod background_options;
mod image_options;
mod qr_options;
mod options;

pub use color::Color;
pub use gradient::{Gradient, ColorStop};
pub use dot_options::DotsOptions;
pub use corner_options::{CornersSquareOptions, CornersDotOptions};
pub use background_options::BackgroundOptions;
pub use image_options::ImageOptions;
pub use qr_options::QROptions;
pub use options::{QRCodeStylingOptions, QRCodeStylingBuilder};
