//! Raster (PNG/JPEG/WebP) renderer for QR codes using resvg.

use crate::error::{QRError, Result};
use crate::types::OutputFormat;
use image::{DynamicImage, ImageFormat, RgbaImage};
use resvg::tiny_skia::Pixmap;
use usvg::{Transform, Tree};
use std::io::Cursor;

/// Raster renderer for converting SVG to raster formats.
pub struct RasterRenderer;

impl RasterRenderer {
    /// Convert SVG string to raster image bytes.
    pub fn render(svg: &str, width: u32, height: u32, format: OutputFormat) -> Result<Vec<u8>> {
        // Parse the SVG with resvg/usvg
        // JPEG has no alpha channel, so composite onto white; other formats keep transparency.
        let background = match format {
            OutputFormat::Jpeg => Some(resvg::tiny_skia::Color::WHITE),
            _ => None,
        };
        let image = Self::svg_to_image(svg, width, height, background)?;

        // Encode to target format
        Self::encode_image(&image, format)
    }

    /// Parse SVG and render to image buffer using resvg.
    fn svg_to_image(
        svg: &str,
        width: u32,
        height: u32,
        background: Option<resvg::tiny_skia::Color>,
    ) -> Result<DynamicImage> {
        // Parse SVG using usvg
        // Shared font database so text (e.g. border decorations) renders
        let tree = Tree::from_str(svg, &super::usvg_options(svg))
            .map_err(|e| QRError::SvgError(e.to_string()))?;

        // Get the SVG's original size
        let svg_size = tree.size();

        // Create a pixmap with the target dimensions
        let mut pixmap = Pixmap::new(width, height)
            .ok_or_else(|| QRError::SvgError("Failed to create pixmap".to_string()))?;

        if let Some(color) = background {
            pixmap.fill(color);
        }

        // Calculate scale to fit the SVG into the target dimensions
        let scale_x = width as f32 / svg_size.width();
        let scale_y = height as f32 / svg_size.height();
        let scale = scale_x.min(scale_y);

        // Calculate offset to center the SVG
        let offset_x = (width as f32 - svg_size.width() * scale) / 2.0;
        let offset_y = (height as f32 - svg_size.height() * scale) / 2.0;

        // Create transform
        let transform = Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);

        // Render the SVG
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        // Convert pixmap (premultiplied alpha) to image::RgbaImage (straight alpha)
        let data: Vec<u8> = pixmap
            .pixels()
            .iter()
            .flat_map(|p| {
                let c = p.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect();
        let img = RgbaImage::from_raw(width, height, data)
            .ok_or_else(|| QRError::SvgError("Failed to create image from pixmap".to_string()))?;

        Ok(DynamicImage::ImageRgba8(img))
    }

    /// Encode image to the specified format.
    fn encode_image(image: &DynamicImage, format: OutputFormat) -> Result<Vec<u8>> {
        let mut buffer = Cursor::new(Vec::new());

        let image_format = match format {
            OutputFormat::Png => ImageFormat::Png,
            OutputFormat::Jpeg => ImageFormat::Jpeg,
            OutputFormat::WebP => ImageFormat::WebP,
            OutputFormat::Svg => {
                return Err(QRError::ImageEncodeError(
                    "SVG format should use SVG renderer".to_string(),
                ));
            }
            OutputFormat::Pdf => {
                return Err(QRError::ImageEncodeError(
                    "PDF format should use PDF renderer".to_string(),
                ));
            }
        };

        let result = if image_format == ImageFormat::Jpeg {
            // JPEG encoder doesn't support alpha
            DynamicImage::ImageRgb8(image.to_rgb8()).write_to(&mut buffer, image_format)
        } else {
            image.write_to(&mut buffer, image_format)
        };

        result
            .map_err(|e| QRError::ImageEncodeError(e.to_string()))?;

        Ok(buffer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "png")]
    fn test_encode_png() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let result = RasterRenderer::encode_image(&img, OutputFormat::Png);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        // PNG magic bytes
        assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    #[cfg(feature = "jpeg")]
    fn test_encode_jpeg() {
        let img = DynamicImage::ImageRgba8(RgbaImage::new(100, 100));
        let result = RasterRenderer::encode_image(&img, OutputFormat::Jpeg);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
        // JPEG magic bytes
        assert_eq!(&bytes[0..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn test_svg_to_image() {
        // Simple SVG with a black square
        let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">
                <rect x="0" y="0" width="100" height="100" fill="white"/>
                <rect x="25" y="25" width="50" height="50" fill="black"/>
            </svg>"#;

        let result = RasterRenderer::svg_to_image(svg, 100, 100, None);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "png")]
    fn test_full_render() {
        let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">
                <rect x="0" y="0" width="100" height="100" fill="white"/>
                <rect x="25" y="25" width="50" height="50" fill="black"/>
            </svg>"#;

        let result = RasterRenderer::render(svg, 100, 100, OutputFormat::Png);
        assert!(result.is_ok());
        let bytes = result.unwrap();
        // PNG magic bytes
        assert_eq!(&bytes[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }
}
