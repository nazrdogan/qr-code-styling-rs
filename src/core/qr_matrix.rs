//! QR code matrix wrapper providing neighbor lookup functionality.

use crate::config::QROptions;
use crate::error::{QRError, Result};
use qrcode::{QrCode, Version};

/// Wrapper around the QR code matrix providing efficient module access.
#[derive(Debug, Clone)]
pub struct QRMatrix {
    /// Flat array of module values (true = dark, false = light).
    modules: Vec<bool>,
    /// Size of the QR code (number of modules per side).
    size: usize,
}

impl QRMatrix {
    /// Create a new QR matrix from data with the specified options.
    pub fn new(data: &str, options: &QROptions) -> Result<Self> {
        let ec_level = options.error_correction_level.to_qrcode_level();

        // Determine the version
        let version = if options.type_number == 0 {
            None // Auto-detect
        } else {
            Some(Version::Normal(options.type_number as i16))
        };

        // Build the QR code
        let qr = if let Some(v) = version {
            QrCode::with_version(data.as_bytes(), v, ec_level)
                .map_err(|e| QRError::QRGenerationError(e.to_string()))?
        } else {
            QrCode::with_error_correction_level(data.as_bytes(), ec_level)
                .map_err(|e| QRError::QRGenerationError(e.to_string()))?
        };

        let size = qr.width() as usize;
        let mut modules = Vec::with_capacity(size * size);

        // Convert to flat array for O(1) access
        for y in 0..size {
            for x in 0..size {
                let color = qr[(x, y)];
                modules.push(color == qrcode::Color::Dark);
            }
        }

        Ok(Self { modules, size })
    }

    /// Get the size (width/height) of the QR code in modules.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get the module count (same as size for compatibility).
    #[inline]
    pub fn module_count(&self) -> usize {
        self.size
    }

    /// Check if a module at (row, col) is dark.
    #[inline]
    pub fn is_dark(&self, row: usize, col: usize) -> bool {
        if row >= self.size || col >= self.size {
            return false;
        }
        self.modules[row * self.size + col]
    }

    /// Check if a module at (row, col) is dark, with signed coordinates.
    /// Returns false for out-of-bounds coordinates.
    #[inline]
    pub fn is_dark_signed(&self, row: i32, col: i32) -> bool {
        if row < 0 || col < 0 {
            return false;
        }
        self.is_dark(row as usize, col as usize)
    }

    /// Get neighbor state relative to a position.
    #[inline]
    pub fn get_neighbor(&self, row: i32, col: i32, offset_x: i32, offset_y: i32) -> bool {
        self.is_dark_signed(row + offset_y, col + offset_x)
    }

    /// Check if a position is part of a finder pattern (corner square).
    /// Finder patterns are 7x7 and located at:
    /// - Top-left: (0, 0)
    /// - Top-right: (0, size-7)
    /// - Bottom-left: (size-7, 0)
    pub fn is_finder_pattern(&self, row: usize, col: usize) -> bool {
        let size = self.size;

        // Top-left finder pattern
        if row < 7 && col < 7 {
            return true;
        }

        // Top-right finder pattern
        if row < 7 && col >= size - 7 {
            return true;
        }

        // Bottom-left finder pattern
        if row >= size - 7 && col < 7 {
            return true;
        }

        false
    }

    /// Check if a position is part of a finder pattern's outer square (7x7 border).
    pub fn is_finder_pattern_outer(&self, row: usize, col: usize) -> bool {
        if !self.is_finder_pattern(row, col) {
            return false;
        }

        let size = self.size;

        // Check if on the border of any finder pattern
        let check_border = |r: usize, c: usize, start_r: usize, start_c: usize| -> bool {
            let local_r = r - start_r;
            let local_c = c - start_c;
            local_r == 0 || local_r == 6 || local_c == 0 || local_c == 6
        };

        // Top-left
        if row < 7 && col < 7 {
            return check_border(row, col, 0, 0);
        }

        // Top-right
        if row < 7 && col >= size - 7 {
            return check_border(row, col, 0, size - 7);
        }

        // Bottom-left
        if row >= size - 7 && col < 7 {
            return check_border(row, col, size - 7, 0);
        }

        false
    }

    /// Check if a position is part of a finder pattern's inner dot (3x3 center).
    pub fn is_finder_pattern_inner(&self, row: usize, col: usize) -> bool {
        if !self.is_finder_pattern(row, col) {
            return false;
        }

        let size = self.size;

        let check_inner = |r: usize, c: usize, start_r: usize, start_c: usize| -> bool {
            let local_r = r - start_r;
            let local_c = c - start_c;
            local_r >= 2 && local_r <= 4 && local_c >= 2 && local_c <= 4
        };

        // Top-left
        if row < 7 && col < 7 {
            return check_inner(row, col, 0, 0);
        }

        // Top-right
        if row < 7 && col >= size - 7 {
            return check_inner(row, col, 0, size - 7);
        }

        // Bottom-left
        if row >= size - 7 && col < 7 {
            return check_inner(row, col, size - 7, 0);
        }

        false
    }
}

/// Square mask for corner squares (7x7 pattern).
/// 1 = part of outer square border, 0 = not part of border
#[allow(dead_code)]
pub const SQUARE_MASK: [[u8; 7]; 7] = [
    [1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1],
];

/// Dot mask for corner dots (7x7 pattern).
/// 1 = part of inner 3x3 dot, 0 = not part of dot
#[allow(dead_code)]
pub const DOT_MASK: [[u8; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_matrix_creation() {
        let options = QROptions::default();
        let matrix = QRMatrix::new("Hello", &options).unwrap();
        assert!(matrix.size() >= 21); // Minimum QR code size
    }

    #[test]
    fn test_is_dark() {
        let options = QROptions::default();
        let matrix = QRMatrix::new("Test", &options).unwrap();

        // Finder pattern top-left corner should be dark
        assert!(matrix.is_dark(0, 0));
    }

    #[test]
    fn test_neighbor_lookup() {
        let options = QROptions::default();
        let matrix = QRMatrix::new("Test", &options).unwrap();

        // Test that neighbor lookup works
        let dark = matrix.is_dark(0, 0);
        let neighbor = matrix.get_neighbor(0, 1, -1, 0);
        assert_eq!(dark, neighbor);
    }

    #[test]
    fn test_finder_pattern_detection() {
        let options = QROptions::default();
        let matrix = QRMatrix::new("Test", &options).unwrap();

        // Top-left corner should be finder pattern
        assert!(matrix.is_finder_pattern(0, 0));
        assert!(matrix.is_finder_pattern(3, 3));
        assert!(matrix.is_finder_pattern(6, 6));

        // Middle of QR code should not be finder pattern
        let mid = matrix.size() / 2;
        assert!(!matrix.is_finder_pattern(mid, mid));
    }
}
