//! Output format variants.

/// Supported output formats for QR code rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum OutputFormat {
    /// SVG vector format.
    #[default]
    Svg,
    /// PNG raster format.
    Png,
    /// JPEG raster format.
    Jpeg,
    /// WebP raster format.
    WebP,
    /// PDF document format.
    Pdf,
}

impl OutputFormat {
    /// Returns the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            OutputFormat::Svg => "image/svg+xml",
            OutputFormat::Png => "image/png",
            OutputFormat::Jpeg => "image/jpeg",
            OutputFormat::WebP => "image/webp",
            OutputFormat::Pdf => "application/pdf",
        }
    }

    /// Returns the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Svg => "svg",
            OutputFormat::Png => "png",
            OutputFormat::Jpeg => "jpeg",
            OutputFormat::WebP => "webp",
            OutputFormat::Pdf => "pdf",
        }
    }
}
