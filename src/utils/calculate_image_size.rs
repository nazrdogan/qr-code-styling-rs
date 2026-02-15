//! Image size calculation for logo embedding.

/// Result of image size calculation.
#[derive(Debug, Clone, Copy)]
pub struct ImageSizeResult {
    /// Width in pixels.
    pub width: f64,
    /// Height in pixels.
    pub height: f64,
    /// Number of dots to hide on X axis.
    pub hide_x_dots: usize,
    /// Number of dots to hide on Y axis.
    pub hide_y_dots: usize,
}

/// Calculate the optimal image size for embedding in QR code.
///
/// This function calculates how large an image can be while maintaining
/// the QR code's readability based on error correction level.
pub fn calculate_image_size(
    original_width: u32,
    original_height: u32,
    max_hidden_dots: usize,
    max_hidden_axis_dots: usize,
    dot_size: f64,
) -> ImageSizeResult {
    let k = original_height as f64 / original_width as f64; // aspect ratio

    // Calculate max X axis hidden dots
    let mut hide_x_dots = ((max_hidden_dots as f64 / k).sqrt()).floor() as usize;
    if hide_x_dots == 0 {
        hide_x_dots = 1;
    }

    // Ensure odd number for center alignment
    if hide_x_dots % 2 == 0 {
        hide_x_dots -= 1;
    }

    // Clamp to max
    hide_x_dots = hide_x_dots.min(max_hidden_axis_dots);

    // Calculate Y dots using aspect ratio
    let mut hide_y_dots = 1 + 2 * (((hide_x_dots as f64 * k) - 1.0) / 2.0).ceil() as usize;

    // Verify doesn't exceed limits
    while hide_y_dots * hide_x_dots > max_hidden_dots && hide_x_dots > 3 {
        hide_x_dots -= 2; // Keep odd
        hide_y_dots = 1 + 2 * (((hide_x_dots as f64 * k) - 1.0) / 2.0).ceil() as usize;
    }

    // Clamp Y to max
    hide_y_dots = hide_y_dots.min(max_hidden_axis_dots);

    ImageSizeResult {
        width: hide_x_dots as f64 * dot_size,
        height: hide_y_dots as f64 * dot_size,
        hide_x_dots,
        hide_y_dots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_image_size_square() {
        let result = calculate_image_size(100, 100, 100, 15, 10.0);
        assert!(result.hide_x_dots > 0);
        assert!(result.hide_y_dots > 0);
        assert!(result.hide_x_dots * result.hide_y_dots <= 100);
    }

    #[test]
    fn test_calculate_image_size_wide() {
        let result = calculate_image_size(200, 100, 100, 15, 10.0);
        assert!(result.hide_x_dots >= result.hide_y_dots);
    }

    #[test]
    fn test_calculate_image_size_tall() {
        let result = calculate_image_size(100, 200, 100, 15, 10.0);
        assert!(result.hide_y_dots >= result.hide_x_dots);
    }

    #[test]
    fn test_odd_hide_dots() {
        let result = calculate_image_size(100, 100, 100, 15, 10.0);
        assert!(result.hide_x_dots % 2 == 1);
    }
}
