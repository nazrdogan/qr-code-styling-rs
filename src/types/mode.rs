//! QR code encoding mode variants.

/// Encoding mode for QR code data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mode {
    /// Numeric mode (0-9).
    Numeric,
    /// Alphanumeric mode (0-9, A-Z, space, $%*+-./: ).
    Alphanumeric,
    /// Byte mode (any 8-bit data).
    Byte,
    /// Kanji mode.
    Kanji,
}

impl Mode {
    /// Auto-detect the best encoding mode for the given data.
    pub fn detect(data: &str) -> Mode {
        if data.chars().all(|c| c.is_ascii_digit()) {
            Mode::Numeric
        } else if data.chars().all(|c| {
            c.is_ascii_digit()
                || c.is_ascii_uppercase()
                || matches!(c, ' ' | '$' | '%' | '*' | '+' | '-' | '.' | '/' | ':')
        }) {
            Mode::Alphanumeric
        } else {
            Mode::Byte
        }
    }
}
