//! PDF renderer for QR codes using SVG to PDF vector conversion.

use crate::error::{QRError, Result};
use svg2pdf::usvg;

/// PDF renderer for converting SVG to PDF format (vector).
pub struct PdfRenderer;

impl PdfRenderer {
    /// Render SVG string directly to PDF format (preserves vector quality).
    pub fn render_from_svg(svg: &str, _width: u32, _height: u32) -> Result<Vec<u8>> {
        // Create font database and load system fonts
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();
        let fontdb = std::sync::Arc::new(fontdb);

        // Create options with the font database
        let mut options = usvg::Options::default();
        options.fontdb = fontdb;

        // Parse SVG using usvg with font database (text will be converted to paths)
        let tree = usvg::Tree::from_str(svg, &options)
            .map_err(|e| QRError::SvgError(format!("Failed to parse SVG: {}", e)))?;

        // Convert to PDF using svg2pdf
        let pdf = svg2pdf::to_pdf(
            &tree,
            svg2pdf::ConversionOptions::default(),
            svg2pdf::PageOptions::default(),
        )
        .map_err(|e| QRError::ImageEncodeError(format!("PDF conversion error: {:?}", e)))?;

        Ok(pdf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_render_from_svg() {
        let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100">
    <rect x="0" y="0" width="100" height="100" fill="white"/>
    <rect x="25" y="25" width="50" height="50" fill="black"/>
</svg>"#;

        let result = PdfRenderer::render_from_svg(svg, 100, 100);
        assert!(result.is_ok());

        let pdf_bytes = result.unwrap();
        // PDF files start with %PDF
        assert!(pdf_bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn test_pdf_render_circle() {
        let svg = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" width="200" height="200">
    <circle cx="100" cy="100" r="50" fill="blue"/>
</svg>"#;

        let result = PdfRenderer::render_from_svg(svg, 200, 200);
        assert!(result.is_ok());
    }
}
