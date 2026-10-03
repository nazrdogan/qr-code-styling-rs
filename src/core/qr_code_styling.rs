//! Main QRCodeStyling struct.

use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::config::{QRCodeStylingBuilder, QRCodeStylingOptions};
use crate::core::QRMatrix;
use crate::error::Result;
#[cfg(feature = "pdf")]
use crate::rendering::PdfRenderer;
use crate::rendering::{RasterRenderer, SvgRenderer};
use crate::types::OutputFormat;

/// Main QR code styling struct.
///
/// This is the primary entry point for creating styled QR codes.
///
/// # Example
///
/// ```rust
/// use qr_code_styling::{QRCodeStyling, OutputFormat};
///
/// let qr = QRCodeStyling::builder()
///     .data("https://example.com")
///     .width(300)
///     .height(300)
///     .build()
///     .unwrap();
///
/// let svg = qr.render_svg().unwrap();
/// ```
pub struct QRCodeStyling {
    options: QRCodeStylingOptions,
    matrix: QRMatrix,
}

impl QRCodeStylingBuilder {
    /// Build the QRCodeStyling with the configured options.
    pub fn build(self) -> Result<QRCodeStyling> {
        let options = self.build_options()?;
        QRCodeStyling::new(options)
    }
}

impl QRCodeStyling {
    /// Create a new QRCodeStyling builder.
    pub fn builder() -> QRCodeStylingBuilder {
        QRCodeStylingBuilder::new()
    }

    /// Create a new QRCodeStyling with the given options.
    pub fn new(options: QRCodeStylingOptions) -> Result<Self> {
        let matrix = QRMatrix::new(&options.data, &options.qr_options)?;

        Ok(Self { options, matrix })
    }

    /// Update the data and regenerate the QR code.
    pub fn update(&mut self, data: &str) -> Result<&mut Self> {
        self.options.data = data.to_string();
        self.matrix = QRMatrix::new(&self.options.data, &self.options.qr_options)?;
        Ok(self)
    }

    /// Render the QR code as an SVG string.
    pub fn render_svg(&self) -> Result<String> {
        let renderer = SvgRenderer::from_ref(&self.options);
        renderer.render(&self.matrix)
    }

    /// Render the QR code in the specified format.
    pub fn render(&self, format: OutputFormat) -> Result<Vec<u8>> {
        match format {
            OutputFormat::Svg => {
                let svg = self.render_svg()?;
                Ok(svg.into_bytes())
            }
            OutputFormat::Png | OutputFormat::Jpeg | OutputFormat::WebP => {
                let svg = self.render_svg()?;
                RasterRenderer::render(&svg, self.options.width, self.options.height, format)
            }
            #[cfg(feature = "pdf")]
            OutputFormat::Pdf => {
                // Convert SVG directly to PDF (vector quality preserved)
                let svg = self.render_svg()?;
                PdfRenderer::render_from_svg(&svg, self.options.width, self.options.height)
            }
            #[cfg(not(feature = "pdf"))]
            OutputFormat::Pdf => Err(crate::error::QRError::ImageEncodeError(
                "PDF output requires the `pdf` feature".to_string(),
            )),
        }
    }

    /// Render a print-ready PDF in the DeviceCMYK color space.
    ///
    /// Shapes are written directly (no SVG/sRGB step), so every color is an
    /// exact CMYK value: map brand colors with
    /// [`CmykPdfOptions::with_color`](crate::CmykPdfOptions::with_color);
    /// others are converted with [`Cmyk::from_rgb`](crate::Cmyk::from_rgb),
    /// which prints black as 100% K. `BorderPlugin` decorations aren't
    /// included, since they are applied to the SVG string.
    #[cfg(feature = "cmyk")]
    pub fn render_pdf_cmyk(&self, options: &crate::rendering::CmykPdfOptions) -> Result<Vec<u8>> {
        let renderer = SvgRenderer::from_ref(&self.options);
        let scene = renderer.scene(&self.matrix);
        crate::rendering::cmyk_pdf_write(&scene, options)
    }

    /// Write the QR code as a CMYK Form XObject into your own PDF, e.g. to
    /// lay out many codes on one sheet.
    ///
    /// Objects are allocated from `alloc` (advanced past the last id used).
    /// The form is self-contained (it carries its own resources) and its
    /// bounding box is `[0 0 width height]` in points; place it with a
    /// `cm` transform followed by `Do`. Colors work as in
    /// [`render_pdf_cmyk`](Self::render_pdf_cmyk).
    ///
    /// ```
    /// use qr_code_styling::pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref};
    /// use qr_code_styling::{CmykPdfOptions, QRCodeStyling};
    ///
    /// let mut pdf = Pdf::new();
    /// let mut alloc = Ref::new(1);
    /// let (catalog, tree, page, contents) = (alloc.bump(), alloc.bump(), alloc.bump(), alloc.bump());
    ///
    /// let options = CmykPdfOptions::new();
    /// let qr = QRCodeStyling::builder().data("https://example.com").size(100).build().unwrap();
    /// let form = qr.write_cmyk_xobject(&mut pdf, &mut alloc, &options).unwrap();
    ///
    /// // Draw it at (50, 50), scaled to 72 pt
    /// let mut content = Content::new();
    /// content.save_state();
    /// content.transform([72.0 / form.width, 0.0, 0.0, 72.0 / form.height, 50.0, 50.0]);
    /// content.x_object(Name(b"Qr0"));
    /// content.restore_state();
    /// pdf.stream(contents, &content.finish());
    ///
    /// pdf.catalog(catalog).pages(tree);
    /// pdf.pages(tree).kids([page]).count(1);
    /// let mut p = pdf.page(page);
    /// p.media_box(Rect::new(0.0, 0.0, 595.0, 842.0)).parent(tree).contents(contents);
    /// p.resources().x_objects().pair(Name(b"Qr0"), form.id);
    /// p.finish();
    /// let bytes = pdf.finish();
    /// assert!(bytes.starts_with(b"%PDF"));
    /// ```
    #[cfg(feature = "cmyk")]
    pub fn write_cmyk_xobject(
        &self,
        chunk: &mut crate::pdf_writer::Chunk,
        alloc: &mut crate::pdf_writer::Ref,
        options: &crate::rendering::CmykPdfOptions,
    ) -> Result<crate::rendering::CmykXObject> {
        let renderer = SvgRenderer::from_ref(&self.options);
        let scene = renderer.scene(&self.matrix);
        crate::rendering::cmyk_xobject_write(&scene, options, chunk, alloc)
    }

    /// Save a CMYK PDF (see [`render_pdf_cmyk`](Self::render_pdf_cmyk)).
    #[cfg(feature = "cmyk")]
    pub fn save_pdf_cmyk<P: AsRef<Path>>(
        &self,
        path: P,
        options: &crate::rendering::CmykPdfOptions,
    ) -> Result<()> {
        let data = self.render_pdf_cmyk(options)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Save the QR code to a file.
    pub fn save<P: AsRef<Path>>(&self, path: P, format: OutputFormat) -> Result<()> {
        let data = self.render(format)?;
        let mut file = File::create(path)?;
        file.write_all(&data)?;
        Ok(())
    }

    /// Get the QR code module count.
    pub fn module_count(&self) -> usize {
        self.matrix.module_count()
    }

    /// Get the current options.
    pub fn options(&self) -> &QRCodeStylingOptions {
        &self.options
    }

    /// Get mutable reference to options (requires regeneration after).
    pub fn options_mut(&mut self) -> &mut QRCodeStylingOptions {
        &mut self.options
    }

    /// Regenerate the QR matrix (call after modifying options).
    pub fn regenerate(&mut self) -> Result<()> {
        self.matrix = QRMatrix::new(&self.options.data, &self.options.qr_options)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DotType;
    use crate::config::DotsOptions;

    #[test]
    fn test_basic_creation() {
        let qr = QRCodeStyling::builder()
            .data("https://example.com")
            .build()
            .unwrap();

        assert!(qr.module_count() >= 21);
    }

    #[test]
    fn test_render_svg() {
        let qr = QRCodeStyling::builder()
            .data("Test")
            .width(200)
            .height(200)
            .build()
            .unwrap();

        let svg = qr.render_svg().unwrap();
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_update() {
        let mut qr = QRCodeStyling::builder()
            .data("First")
            .build()
            .unwrap();

        let count1 = qr.module_count();

        qr.update("This is a much longer string that should result in a larger QR code")
            .unwrap();

        let count2 = qr.module_count();

        // Longer data should result in larger QR code
        assert!(count2 >= count1);
    }

    #[test]
    fn test_with_dot_options() {
        let qr = QRCodeStyling::builder()
            .data("Test")
            .dots_options(DotsOptions::new(DotType::Dots))
            .build()
            .unwrap();

        let svg = qr.render_svg().unwrap();
        // Lone dots are drawn as two-arc circles inside the shared dots path
        assert!(svg.contains(" 0 1 0 "));
        assert!(!svg.contains("clipPath"));
    }

    #[test]
    #[cfg(feature = "png")]
    fn test_render_png() {
        let qr = QRCodeStyling::builder()
            .data("Test")
            .width(100)
            .height(100)
            .build()
            .unwrap();

        let png = qr.render(OutputFormat::Png).unwrap();
        // PNG magic bytes
        assert_eq!(&png[0..4], &[0x89, 0x50, 0x4E, 0x47]);
    }

    #[test]
    fn test_oversized_margin_is_error_not_panic() {
        let result = QRCodeStyling::builder().data("Test").size(100).margin(50).build();
        assert!(result.is_err());

        // Bypassing the builder must not panic either
        let mut qr = QRCodeStyling::builder().data("Test").size(100).build().unwrap();
        qr.options_mut().margin = 1000;
        assert!(qr.render_svg().is_ok());
    }

    #[test]
    #[cfg(all(feature = "png", feature = "jpeg"))]
    fn test_png_keeps_transparency() {
        use crate::config::{BackgroundOptions, Color};

        let qr = QRCodeStyling::builder()
            .data("Test")
            .size(100)
            .dots_options(DotsOptions::new(DotType::Square).with_color(Color::rgba(255, 0, 0, 128)))
            .background_options(BackgroundOptions::transparent())
            .build()
            .unwrap();

        let png = qr.render(OutputFormat::Png).unwrap();
        let img = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(img.get_pixel(0, 0)[3], 0);
        // Semi-transparent pixels must not be premultiplied
        assert!(img.pixels().any(|p| p.0 == [255, 0, 0, 128]));

        // JPEG still renders (alpha dropped)
        assert!(qr.render(OutputFormat::Jpeg).is_ok());
    }

    #[test]
    #[cfg(feature = "cmyk")]
    fn test_render_pdf_cmyk() {
        use crate::config::{BackgroundOptions, Color, CornersSquareOptions, Gradient};
        use crate::rendering::{Cmyk, CmykPdfOptions};
        use crate::types::CornerSquareType;

        let qr = QRCodeStyling::builder()
            .data("Test")
            .size(200)
            .dots_options(DotsOptions::new(DotType::Rounded))
            .corners_square_options(
                CornersSquareOptions::new(CornerSquareType::ExtraRounded)
                    .with_gradient(Gradient::simple_linear(Color::BLACK, Color::rgb(0, 0, 255))),
            )
            .background_options(BackgroundOptions::new(Color::WHITE).with_round(0.2))
            .build()
            .unwrap();

        let options = CmykPdfOptions::new()
            .with_color(Color::rgb(0, 0, 255), Cmyk::new(100.0, 80.0, 0.0, 0.0))
            .with_compression(false);
        let pdf = qr.render_pdf_cmyk(&options).unwrap();
        let text = String::from_utf8_lossy(&pdf);

        assert!(pdf.starts_with(b"%PDF"));
        assert!(text.contains("/MediaBox [0 0 200 200]"));
        // Dots in 100% K, no RGB anywhere
        assert!(text.contains("0 0 0 1 k"));
        assert!(text.contains("/DeviceCMYK"));
        assert!(!text.contains(" rg\n") && !text.contains("/ICCBased"));
        // Mapped gradient stop uses the exact CMYK value
        assert!(text.contains("/C1 [1 0.8 0 0]"));
    }

    #[test]
    #[cfg(feature = "cmyk")]
    fn test_cmyk_xobjects_share_one_document() {
        use crate::config::{Color, Gradient};
        use crate::pdf_writer::{Finish, Name, Pdf, Rect, Ref};
        use crate::rendering::CmykPdfOptions;

        let mut pdf = Pdf::new();
        let mut alloc = Ref::new(1);
        let (catalog, tree, page) = (alloc.bump(), alloc.bump(), alloc.bump());
        let options = CmykPdfOptions::new().with_compression(false);

        let mut forms = Vec::new();
        for i in 0..3 {
            // Gradients force each form to carry its own shading resources
            let qr = QRCodeStyling::builder()
                .data(format!("item {}", i))
                .size(120)
                .dots_options(DotsOptions::new(DotType::Rounded).with_gradient(Gradient::simple_radial(Color::BLACK, Color::rgb(0, 0, 200))))
                .build()
                .unwrap();
            forms.push(qr.write_cmyk_xobject(&mut pdf, &mut alloc, &options).unwrap());
        }
        // Ids are unique and the allocator moved past all of them
        let ids: std::collections::HashSet<_> = forms.iter().map(|f| f.id).collect();
        assert_eq!(ids.len(), 3);
        assert!(forms.iter().all(|f| f.id.get() < alloc.get()));
        assert_eq!((forms[0].width, forms[0].height), (120.0, 120.0));

        pdf.catalog(catalog).pages(tree);
        pdf.pages(tree).kids([page]).count(1);
        let mut p = pdf.page(page);
        p.media_box(Rect::new(0.0, 0.0, 400.0, 200.0)).parent(tree);
        let mut res = p.resources();
        let mut xo = res.x_objects();
        for (i, f) in forms.iter().enumerate() {
            xo.pair(Name(format!("Q{}", i).as_bytes()), f.id);
        }
        xo.finish();
        res.finish();
        p.finish();
        let bytes = pdf.finish();
        let text = String::from_utf8_lossy(&bytes);

        assert_eq!(text.matches("/Subtype /Form").count(), 3);
        assert_eq!(text.matches("/BBox [0 0 120 120]").count(), 3);
        // Each form has its own resources with its own shading named Sh0
        assert_eq!(text.matches("/Sh0").count(), 6); // resource entry + `sh` use, per form
    }
}
