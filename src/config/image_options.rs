//! Image/logo embedding options.

/// Options for embedding an image/logo in the QR code.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImageOptions {
    /// Size of the image relative to the QR code (0.0 to 1.0).
    pub image_size: f64,
    /// Whether to hide dots behind the image.
    pub hide_background_dots: bool,
    /// Margin around the image in modules.
    pub margin: u32,
    /// Cross-origin setting for loading images (browser context).
    pub cross_origin: Option<String>,
    /// Whether to save the image as a data URL in SVG.
    pub save_as_blob: bool,
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            image_size: 0.4,
            hide_background_dots: true,
            margin: 0,
            cross_origin: None,
            save_as_blob: true,
        }
    }
}

impl ImageOptions {
    /// Create new image options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the image size ratio.
    pub fn with_image_size(mut self, size: f64) -> Self {
        self.image_size = size.clamp(0.0, 1.0);
        self
    }

    /// Set whether to hide background dots.
    pub fn with_hide_background_dots(mut self, hide: bool) -> Self {
        self.hide_background_dots = hide;
        self
    }

    /// Set the margin around the image.
    pub fn with_margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }

    /// Set cross-origin setting.
    pub fn with_cross_origin(mut self, cross_origin: impl Into<String>) -> Self {
        self.cross_origin = Some(cross_origin.into());
        self
    }

    /// Set whether to save as blob/data URL.
    pub fn with_save_as_blob(mut self, save: bool) -> Self {
        self.save_as_blob = save;
        self
    }
}
