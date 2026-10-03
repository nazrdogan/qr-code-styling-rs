//! QR code matrix wrapper providing neighbor lookup functionality.

use crate::config::QROptions;
use crate::error::{QRError, Result};
use crate::types::Mode;
use qrcode::bits::Bits;
use qrcode::canvas::{Canvas, MaskPattern};
use qrcode::types::QrError;
use qrcode::{EcLevel, QrCode, Version};

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

        if options.type_number > 40 {
            return Err(QRError::InvalidVersion(options.type_number));
        }

        // Determine the version
        let version = if options.type_number == 0 {
            None // Auto-detect
        } else {
            Some(Version::Normal(options.type_number as i16))
        };

        // Build the QR code
        let qr = match options.mode {
            Some(mode) => Self::encode_with_mode(data.as_bytes(), mode, version, ec_level)?,
            None => match version {
                Some(v) => QrCode::with_version(data.as_bytes(), v, ec_level),
                None => QrCode::with_error_correction_level(data.as_bytes(), ec_level),
            }
            .map_err(|e| QRError::QRGenerationError(e.to_string()))?,
        };

        let size = qr.width();
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

    /// Create a matrix identical to the one the JavaScript `qr-code-styling`
    /// library (via `qrcode-generator`) produces for the same input.
    ///
    /// Differences from [`QRMatrix::new`]:
    /// - one segment in the mode JS picks (`Numeric` if all digits,
    ///   `Alphanumeric` if all uppercase-alphanumeric, else `Byte`) instead
    ///   of an optimal mix of segments;
    /// - Byte mode writes one byte per character (`charCode & 0xFF`) like JS.
    ///   Strings with characters above U+00FF, which JS would corrupt, are
    ///   encoded as UTF-8 instead;
    /// - the mask is chosen with `qrcode-generator`'s penalty function,
    ///   evaluated with format/version info left blank as JS does.
    pub fn new_js_compatible(data: &str, options: &QROptions) -> Result<Self> {
        let ec_level = options.error_correction_level.to_qrcode_level();
        if options.type_number > 40 {
            return Err(QRError::InvalidVersion(options.type_number));
        }
        let version = (options.type_number != 0).then_some(Version::Normal(options.type_number as i16));
        let mode = options.mode.unwrap_or_else(|| Self::js_mode(data));

        let bytes: Vec<u8> = if mode == Mode::Byte && data.chars().all(|c| (c as u32) <= 0xFF) {
            data.chars().map(|c| c as u8).collect()
        } else {
            data.as_bytes().to_vec()
        };

        let bits = Self::encode_bits(&bytes, mode, version, ec_level)?;
        let version = bits.version();
        let (data_codewords, ec_codewords) = qrcode::ec::construct_codewords(&bits.into_bytes(), version, ec_level)
            .map_err(|e| QRError::QRGenerationError(e.to_string()))?;

        let mut canvas = Canvas::new(version, ec_level);
        canvas.draw_all_functional_patterns();
        canvas.draw_data(&data_codewords, &ec_codewords);

        let size = version.width() as usize;
        let blank = js_test_mode_blank_modules(size, version);
        // First mask with the lowest penalty wins, as in qrcode-generator
        let mut best: Option<(f64, MaskPattern)> = None;
        for pattern in ALL_MASKS {
            let mut masked = canvas.clone();
            masked.apply_mask(pattern);
            let mut modules: Vec<bool> = masked.into_colors().into_iter().map(|c| c == qrcode::Color::Dark).collect();
            for &i in &blank {
                modules[i] = false;
            }
            let penalty = js_lost_point(&modules, size);
            if best.is_none_or(|(p, _)| penalty < p) {
                best = Some((penalty, pattern));
            }
        }

        canvas.apply_mask(best.expect("eight masks evaluated").1);
        Ok(Self {
            modules: canvas.into_colors().into_iter().map(|c| c == qrcode::Color::Dark).collect(),
            size,
        })
    }

    /// The mode JS `qr-code-styling` picks for `data`.
    fn js_mode(data: &str) -> Mode {
        if data.bytes().all(|b| b.is_ascii_digit()) {
            Mode::Numeric
        } else if data.bytes().all(|b| b.is_ascii_digit() || b.is_ascii_uppercase() || b" $%*+-./:".contains(&b)) {
            Mode::Alphanumeric
        } else {
            Mode::Byte
        }
    }

    /// Encode data using a single, explicitly chosen mode.
    ///
    /// With no version given, the smallest version that fits is used.
    fn encode_with_mode(
        data: &[u8],
        mode: Mode,
        version: Option<Version>,
        ec_level: EcLevel,
    ) -> Result<QrCode> {
        let bits = Self::encode_bits(data, mode, version, ec_level)?;
        QrCode::with_bits(bits, ec_level).map_err(|e| QRError::QRGenerationError(e.to_string()))
    }

    /// Encode `data` as one segment in `mode`, terminated and padded.
    /// With no version given, the smallest version that fits is used.
    fn encode_bits(data: &[u8], mode: Mode, version: Option<Version>, ec_level: EcLevel) -> Result<Bits> {
        if !Self::data_fits_mode(data, mode) {
            return Err(QRError::QRGenerationError(format!(
                "data is not valid for {:?} mode",
                mode
            )));
        }

        let encode = |v: Version| -> std::result::Result<Bits, QrError> {
            let mut bits = Bits::new(v);
            match mode {
                Mode::Numeric => bits.push_numeric_data(data)?,
                Mode::Alphanumeric => bits.push_alphanumeric_data(data)?,
                Mode::Byte => bits.push_byte_data(data)?,
                Mode::Kanji => bits.push_kanji_data(data)?,
            }
            bits.push_terminator(ec_level)?;
            Ok(bits)
        };

        match version {
            Some(v) => encode(v),
            None => (1..=40)
                .find_map(|n| encode(Version::Normal(n)).ok())
                .ok_or(QrError::DataTooLong),
        }
        .map_err(|e| QRError::QRGenerationError(e.to_string()))
    }

    /// Check that every byte of `data` can be represented in `mode`.
    fn data_fits_mode(data: &[u8], mode: Mode) -> bool {
        match mode {
            Mode::Numeric => data.iter().all(u8::is_ascii_digit),
            Mode::Alphanumeric => data.iter().all(|b| {
                b.is_ascii_digit()
                    || b.is_ascii_uppercase()
                    || b" $%*+-./:".contains(b)
            }),
            Mode::Byte => true,
            // Kanji mode takes Shift JIS double-byte characters
            Mode::Kanji => {
                data.len().is_multiple_of(2)
                    && data.chunks(2).all(|c| {
                        let cp = u16::from(c[0]) << 8 | u16::from(c[1]);
                        (0x8140..=0x9FFC).contains(&cp) || (0xE040..=0xEBBF).contains(&cp)
                    })
            }
        }
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
            (2..=4).contains(&local_r) && (2..=4).contains(&local_c)
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

const ALL_MASKS: [MaskPattern; 8] = [
    MaskPattern::Checkerboard,
    MaskPattern::HorizontalLines,
    MaskPattern::VerticalLines,
    MaskPattern::DiagonalLines,
    MaskPattern::LargeCheckerboard,
    MaskPattern::Fields,
    MaskPattern::Diamonds,
    MaskPattern::Meadow,
];

/// Indices (row-major) of modules that `qrcode-generator` leaves light while
/// scoring masks (`makeImpl(test = true)`): format info, version info and
/// the dark module.
fn js_test_mode_blank_modules(size: usize, version: Version) -> Vec<usize> {
    let n = size;
    let mut cells: Vec<(usize, usize)> = Vec::new(); // (row, col)
    for i in (0..=5).chain([7, 8]) {
        cells.push((i, 8));
    }
    for i in (0..=5).chain([7]) {
        cells.push((8, i));
    }
    for i in n - 8..n {
        cells.push((8, i));
    }
    for i in n - 7..n {
        cells.push((i, 8));
    }
    cells.push((n - 8, 8));
    if let Version::Normal(v) = version {
        if v >= 7 {
            for a in 0..6 {
                for b in n - 11..n - 8 {
                    cells.push((a, b));
                    cells.push((b, a));
                }
            }
        }
    }
    cells.into_iter().map(|(r, c)| r * n + c).collect()
}

/// `QRUtil.getLostPoint` from `qrcode-generator` (not the ISO penalty).
fn js_lost_point(m: &[bool], n: usize) -> f64 {
    let dark = |r: usize, c: usize| m[r * n + c];
    let mut lost = 0.0;

    // Level 1: modules whose 8 neighbors mostly match
    for row in 0..n {
        for col in 0..n {
            let d = dark(row, col);
            let mut same = 0;
            for r in row.saturating_sub(1)..=(row + 1).min(n - 1) {
                for c in col.saturating_sub(1)..=(col + 1).min(n - 1) {
                    if (r, c) != (row, col) && dark(r, c) == d {
                        same += 1;
                    }
                }
            }
            if same > 5 {
                lost += (3 + same - 5) as f64;
            }
        }
    }

    // Level 2: 2x2 blocks of one color
    for row in 0..n - 1 {
        for col in 0..n - 1 {
            let count = [dark(row, col), dark(row + 1, col), dark(row, col + 1), dark(row + 1, col + 1)]
                .iter()
                .filter(|&&d| d)
                .count();
            if count == 0 || count == 4 {
                lost += 3.0;
            }
        }
    }

    // Level 3: 1:1:3:1:1 finder-like runs
    let pattern = [true, false, true, true, true, false, true];
    for row in 0..n {
        for col in 0..n - 6 {
            if (0..7).all(|k| dark(row, col + k) == pattern[k]) {
                lost += 40.0;
            }
        }
    }
    for col in 0..n {
        for row in 0..n - 6 {
            if (0..7).all(|k| dark(row + k, col) == pattern[k]) {
                lost += 40.0;
            }
        }
    }

    // Level 4: dark ratio
    let dark_count = m.iter().filter(|&&d| d).count() as f64;
    let ratio = (100.0 * dark_count / n as f64 / n as f64 - 50.0).abs() / 5.0;
    lost + ratio * 10.0
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
    use crate::types::ErrorCorrectionLevel;

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

    #[test]
    fn test_explicit_mode() {
        let numeric = QROptions::new().with_mode(Mode::Numeric);
        assert!(QRMatrix::new("0123456789", &numeric).is_ok());
        assert!(QRMatrix::new("12ab", &numeric).is_err());

        let alnum = QROptions::new().with_mode(Mode::Alphanumeric);
        assert!(QRMatrix::new("HELLO WORLD", &alnum).is_ok());
        assert!(QRMatrix::new("hello", &alnum).is_err());

        let byte = QROptions::new().with_mode(Mode::Byte);
        assert!(QRMatrix::new("héllo", &byte).is_ok());

        // UTF-8 text is not Shift JIS
        let kanji = QROptions::new().with_mode(Mode::Kanji);
        assert!(QRMatrix::new("漢字", &kanji).is_err());
    }

    #[test]
    fn test_explicit_mode_picks_smallest_version() {
        let auto = QRMatrix::new("12345", &QROptions::new()).unwrap();
        let numeric = QRMatrix::new("12345", &QROptions::new().with_mode(Mode::Numeric)).unwrap();
        assert_eq!(numeric.size(), 21);
        assert_eq!(auto.size(), numeric.size());
    }

    #[test]
    fn test_explicit_mode_with_version() {
        let options = QROptions::new().with_mode(Mode::Byte).with_type_number(5);
        let matrix = QRMatrix::new("Test", &options).unwrap();
        assert_eq!(matrix.size(), 17 + 4 * 5);
    }

    #[test]
    fn test_invalid_version() {
        let options = QROptions { type_number: 41, ..QROptions::default() };
        assert!(QRMatrix::new("Test", &options).is_err());
    }

    fn bits(m: &QRMatrix) -> String {
        let n = m.size();
        (0..n * n).map(|i| if m.is_dark(i / n, i % n) { '1' } else { '0' }).collect()
    }

    // Golden matrices produced by qrcode-generator 2.0.4 (the encoder inside
    // JS qr-code-styling 1.9.2) with qr-code-styling's mode detection.
    #[test]
    fn test_js_compatible_matches_qrcode_generator() {
        let q = QROptions::new().with_error_correction_level(ErrorCorrectionLevel::Q);
        let m = QRMatrix::new_js_compatible("https://example.com", &q).unwrap();
        assert_eq!(m.size(), 25);
        assert_eq!(bits(&m), concat!(
            "1111111010110010001111111",
            "1000001010001110101000001",
            "1011101000100010001011101",
            "1011101001000100101011101",
            "1011101000101100101011101",
            "1000001000111010001000001",
            "1111111010101010101111111",
            "0000000000000011100000000",
            "0100001110111000010000011",
            "0011100001110111110111110",
            "1100111001100111100101011",
            "1000000111110100101101001",
            "0101101000001101101100001",
            "1111100111001101100100010",
            "1000101010111101001111011",
            "1010010110011110011101101",
            "1001011000001110111110100",
            "0000000010010000100010000",
            "1111111010111000101010001",
            "1000001001100111100010010",
            "1011101001000101111110101",
            "1011101001011100111000011",
            "1011101000110011000001101",
            "1000001010101011110110001",
            "1111111001010010101001001",
        ));

        // Latin-1 text: one byte per character, like JS
        let m_opts = QROptions::new().with_error_correction_level(ErrorCorrectionLevel::M);
        let m = QRMatrix::new_js_compatible("café crème ü ö ç é", &m_opts).unwrap();
        assert_eq!(bits(&m), concat!(
            "1111111011010011101111111",
            "1000001000001111001000001",
            "1011101001000011101011101",
            "1011101010001001101011101",
            "1011101010100011001011101",
            "1000001010101000101000001",
            "1111111010101010101111111",
            "0000000011100100000000000",
            "1000101110110100011111001",
            "1100100110010111100011000",
            "0000101010000000100101000",
            "1101010001001011100110000",
            "0100111101110110001100101",
            "1010000101101001000010100",
            "0000011010011111110000100",
            "0001000100110100010011101",
            "1110101100001100111110101",
            "0000000010100111100010000",
            "1111111011100000101011100",
            "1000001000101010100010111",
            "1011101010010111111110101",
            "1011101000001000010100011",
            "1011101001111111101110010",
            "1000001001110101001010110",
            "1111111010101100001111111",
        ));

        // Numeric mode
        let h = QROptions::new().with_error_correction_level(ErrorCorrectionLevel::H);
        let m = QRMatrix::new_js_compatible("0123456789", &h).unwrap();
        assert_eq!(bits(&m), concat!(
            "111111100100001111111",
            "100000100011101000001",
            "101110101010101011101",
            "101110101100101011101",
            "101110100111001011101",
            "100000100011001000001",
            "111111101010101111111",
            "000000000010100000000",
            "000110110100100001100",
            "001011000110010111101",
            "001010111000100100110",
            "010111010011010110100",
            "010101110011101111011",
            "000000001110111001111",
            "111111101110101000011",
            "100000100100010000000",
            "101110101001011011001",
            "101110101100000010100",
            "101110100011011111011",
            "100000100111010000001",
            "111111100010011111110",
        ));
    }

    #[test]
    fn test_js_compatible_non_latin1_uses_utf8() {
        // JS would truncate 'ş' (U+015F) to '_'; we encode UTF-8 instead, so
        // the result differs from the truncated string's matrix.
        let o = QROptions::new();
        let utf8 = QRMatrix::new_js_compatible("Kayseri şube", &o).unwrap();
        let truncated = QRMatrix::new_js_compatible("Kayseri _ube", &o).unwrap();
        assert_ne!(bits(&utf8), bits(&truncated));
    }

    #[test]
    fn test_js_mode_detection() {
        assert_eq!(QRMatrix::js_mode("0123"), Mode::Numeric);
        assert_eq!(QRMatrix::js_mode(""), Mode::Numeric);
        assert_eq!(QRMatrix::js_mode("HELLO WORLD $%*+-./:"), Mode::Alphanumeric);
        assert_eq!(QRMatrix::js_mode("Hello"), Mode::Byte);
    }
}
