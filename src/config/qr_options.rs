//! QR code generation options.

use crate::types::{ErrorCorrectionLevel, Mode};

/// Options for QR code generation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QROptions {
    /// QR code version/type number (0 = auto, 1-40 for specific versions).
    pub type_number: u8,
    /// Error correction level.
    pub error_correction_level: ErrorCorrectionLevel,
    /// Encoding mode (None = auto-detect).
    pub mode: Option<Mode>,
}

impl Default for QROptions {
    fn default() -> Self {
        Self {
            type_number: 0, // Auto
            error_correction_level: ErrorCorrectionLevel::Q,
            mode: None, // Auto-detect
        }
    }
}

impl QROptions {
    /// Create new QR options with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the type/version number.
    pub fn with_type_number(mut self, type_number: u8) -> Self {
        self.type_number = type_number.min(40);
        self
    }

    /// Set the error correction level.
    pub fn with_error_correction_level(mut self, level: ErrorCorrectionLevel) -> Self {
        self.error_correction_level = level;
        self
    }

    /// Set the encoding mode.
    pub fn with_mode(mut self, mode: Mode) -> Self {
        self.mode = Some(mode);
        self
    }
}
