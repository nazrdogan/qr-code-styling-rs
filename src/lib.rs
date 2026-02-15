//! # QR Code Styling
//!
//! A Rust library for generating styled QR codes with customizable dots,
//! corners, gradients, and logo embedding.
//!
//! ## Features
//!
//! - 6 dot styles (square, dots, rounded, classy, classy-rounded, extra-rounded)
//! - 3 corner square styles (square, dot, extra-rounded)
//! - 2 corner dot styles (dot, square)
//! - Linear and radial gradient support
//! - Logo/image embedding with automatic sizing
//! - Circle shape support
//! - Multiple output formats (SVG, PNG, JPEG, WebP)
//!
//! ## Example
//!
//! ```rust
//! use qr_code_styling::{QRCodeStyling, OutputFormat};
//! use qr_code_styling::config::{DotsOptions, Color};
//! use qr_code_styling::types::DotType;
//!
//! let qr = QRCodeStyling::builder()
//!     .data("https://example.com")
//!     .width(300)
//!     .height(300)
//!     .dots_options(DotsOptions::new(DotType::Rounded).with_color(Color::rgb(0, 0, 128)))
//!     .build()
//!     .unwrap();
//!
//! // Render as SVG
//! let svg = qr.render_svg().unwrap();
//!
//! // Or save to file
//! // qr.save("qr.png", OutputFormat::Png).unwrap();
//! ```

pub mod config;
pub mod core;
pub mod error;
pub mod figures;
pub mod plugins;
pub mod rendering;
pub mod types;
pub mod utils;

// Re-export main types at crate root for convenience
pub use config::{
    BackgroundOptions, Color, ColorStop, CornersDotOptions, CornersSquareOptions, DotsOptions,
    Gradient, ImageOptions, QRCodeStylingBuilder, QRCodeStylingOptions, QROptions,
};
pub use core::QRCodeStyling;
pub use error::{QRError, Result};
pub use plugins::{BorderDecoration, BorderOptions, BorderPlugin, Position, QRBorderOptions};
pub use types::{
    CornerDotType, CornerSquareType, DotType, ErrorCorrectionLevel, GradientType, Mode,
    OutputFormat, ShapeType,
};
