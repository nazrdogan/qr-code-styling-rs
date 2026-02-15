//! Main QR code styling options with builder pattern.

use super::{
    BackgroundOptions, CornersDotOptions, CornersSquareOptions, DotsOptions, ImageOptions,
    QROptions,
};
use crate::error::{QRError, Result};
use crate::types::ShapeType;

/// Main configuration for QR code styling.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QRCodeStylingOptions {
    /// Data to encode in the QR code.
    pub data: String,
    /// Width of the QR code in pixels.
    pub width: u32,
    /// Height of the QR code in pixels.
    pub height: u32,
    /// Margin around the QR code in pixels.
    pub margin: u32,
    /// Overall shape of the QR code.
    pub shape: ShapeType,
    /// Optional image/logo to embed.
    pub image: Option<Vec<u8>>,
    /// QR code generation options.
    pub qr_options: QROptions,
    /// Dot styling options.
    pub dots_options: DotsOptions,
    /// Corner square styling options.
    pub corners_square_options: CornersSquareOptions,
    /// Corner dot styling options.
    pub corners_dot_options: CornersDotOptions,
    /// Background styling options.
    pub background_options: BackgroundOptions,
    /// Image embedding options.
    pub image_options: ImageOptions,
}

impl Default for QRCodeStylingOptions {
    fn default() -> Self {
        Self {
            data: String::new(),
            width: 300,
            height: 300,
            margin: 0,
            shape: ShapeType::Square,
            image: None,
            qr_options: QROptions::default(),
            dots_options: DotsOptions::default(),
            corners_square_options: CornersSquareOptions::default(),
            corners_dot_options: CornersDotOptions::default(),
            background_options: BackgroundOptions::default(),
            image_options: ImageOptions::default(),
        }
    }
}

/// Builder for constructing QRCodeStylingOptions.
#[derive(Debug, Default, Clone)]
pub struct QRCodeStylingBuilder {
    data: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    margin: Option<u32>,
    shape: Option<ShapeType>,
    image: Option<Vec<u8>>,
    qr_options: Option<QROptions>,
    dots_options: Option<DotsOptions>,
    corners_square_options: Option<CornersSquareOptions>,
    corners_dot_options: Option<CornersDotOptions>,
    background_options: Option<BackgroundOptions>,
    image_options: Option<ImageOptions>,
}

impl QRCodeStylingBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the data to encode.
    pub fn data(mut self, data: impl Into<String>) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Set the width in pixels.
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height in pixels.
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set both width and height to the same value.
    pub fn size(mut self, size: u32) -> Self {
        self.width = Some(size);
        self.height = Some(size);
        self
    }

    /// Set the margin in pixels.
    pub fn margin(mut self, margin: u32) -> Self {
        self.margin = Some(margin);
        self
    }

    /// Set the overall shape.
    pub fn shape(mut self, shape: ShapeType) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Set the image/logo data.
    pub fn image(mut self, image: Vec<u8>) -> Self {
        self.image = Some(image);
        self
    }

    /// Set QR code generation options.
    pub fn qr_options(mut self, options: QROptions) -> Self {
        self.qr_options = Some(options);
        self
    }

    /// Set dot styling options.
    pub fn dots_options(mut self, options: DotsOptions) -> Self {
        self.dots_options = Some(options);
        self
    }

    /// Set corner square styling options.
    pub fn corners_square_options(mut self, options: CornersSquareOptions) -> Self {
        self.corners_square_options = Some(options);
        self
    }

    /// Set corner dot styling options.
    pub fn corners_dot_options(mut self, options: CornersDotOptions) -> Self {
        self.corners_dot_options = Some(options);
        self
    }

    /// Set background styling options.
    pub fn background_options(mut self, options: BackgroundOptions) -> Self {
        self.background_options = Some(options);
        self
    }

    /// Set image embedding options.
    pub fn image_options(mut self, options: ImageOptions) -> Self {
        self.image_options = Some(options);
        self
    }

    /// Build the QRCodeStylingOptions (internal use).
    pub(crate) fn build_options(self) -> Result<QRCodeStylingOptions> {
        let data = self.data.ok_or(QRError::MissingData)?;

        if data.is_empty() {
            return Err(QRError::MissingData);
        }

        let width = self.width.unwrap_or(300);
        let height = self.height.unwrap_or(300);

        if width < 21 || height < 21 {
            return Err(QRError::CanvasTooSmall { width, height });
        }

        Ok(QRCodeStylingOptions {
            data,
            width,
            height,
            margin: self.margin.unwrap_or(0),
            shape: self.shape.unwrap_or(ShapeType::Square),
            image: self.image,
            qr_options: self.qr_options.unwrap_or_default(),
            dots_options: self.dots_options.unwrap_or_default(),
            corners_square_options: self.corners_square_options.unwrap_or_default(),
            corners_dot_options: self.corners_dot_options.unwrap_or_default(),
            background_options: self.background_options.unwrap_or_default(),
            image_options: self.image_options.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let options = QRCodeStylingBuilder::new()
            .data("https://example.com")
            .width(400)
            .height(400)
            .build_options()
            .unwrap();

        assert_eq!(options.data, "https://example.com");
        assert_eq!(options.width, 400);
        assert_eq!(options.height, 400);
    }

    #[test]
    fn test_builder_missing_data() {
        let result = QRCodeStylingBuilder::new().build_options();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_empty_data() {
        let result = QRCodeStylingBuilder::new().data("").build_options();
        assert!(result.is_err());
    }
}
