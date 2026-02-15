//! Color representation for QR code styling.

use crate::error::{QRError, Result};

/// RGBA color representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
    /// Alpha component (0-255).
    pub a: u8,
}

impl Color {
    /// Create a new color with full opacity.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color with specified alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from a hex string (e.g., "#FF0000" or "#FF0000FF").
    pub fn from_hex(hex: &str) -> Result<Self> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            3 => {
                // Short form: #RGB -> #RRGGBB
                let r = u8::from_str_radix(&hex[0..1], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                let g = u8::from_str_radix(&hex[1..2], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                let b = u8::from_str_radix(&hex[2..3], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                Ok(Self::rgb(r * 17, g * 17, b * 17))
            }
            6 => {
                // Standard form: #RRGGBB
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                Ok(Self::rgb(r, g, b))
            }
            8 => {
                // With alpha: #RRGGBBAA
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| QRError::InvalidColor(hex.to_string()))?;
                Ok(Self::rgba(r, g, b, a))
            }
            _ => Err(QRError::InvalidColor(hex.to_string())),
        }
    }

    /// Convert to hex string (e.g., "#FF0000").
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// Convert to CSS rgba string.
    pub fn to_rgba_string(&self) -> String {
        if self.a == 255 {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            format!(
                "rgba({}, {}, {}, {:.3})",
                self.r,
                self.g,
                self.b,
                self.a as f64 / 255.0
            )
        }
    }

    /// Black color.
    pub const BLACK: Color = Color::rgb(0, 0, 0);

    /// White color.
    pub const WHITE: Color = Color::rgb(255, 255, 255);

    /// Transparent color.
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        assert_eq!(Color::from_hex("#FF0000").unwrap(), Color::rgb(255, 0, 0));
        assert_eq!(Color::from_hex("#00FF00").unwrap(), Color::rgb(0, 255, 0));
        assert_eq!(Color::from_hex("#0000FF").unwrap(), Color::rgb(0, 0, 255));
        assert_eq!(Color::from_hex("000000").unwrap(), Color::rgb(0, 0, 0));
        assert_eq!(Color::from_hex("#FFF").unwrap(), Color::rgb(255, 255, 255));
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(Color::rgb(255, 0, 0).to_hex(), "#FF0000");
        assert_eq!(Color::rgba(255, 0, 0, 128).to_hex(), "#FF000080");
    }
}
