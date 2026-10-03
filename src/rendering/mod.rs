//! Rendering modules for QR code output.

use resvg::usvg;

#[cfg(feature = "cmyk")]
mod cmyk_pdf;
pub(crate) mod scene;
mod svg_renderer;
mod raster_renderer;
#[cfg(feature = "pdf")]
mod pdf_renderer;

pub use svg_renderer::SvgRenderer;
pub use raster_renderer::RasterRenderer;
#[cfg(feature = "cmyk")]
pub use cmyk_pdf::{Cmyk, CmykConverter, CmykPdfOptions};
#[cfg(feature = "pdf")]
pub use pdf_renderer::PdfRenderer;

use std::sync::{Arc, OnceLock};

/// Write a scene as a CMYK PDF.
#[cfg(feature = "cmyk")]
pub(crate) fn cmyk_pdf_write(scene: &scene::Scene<'_>, options: &CmykPdfOptions) -> crate::error::Result<Vec<u8>> {
    cmyk_pdf::CmykPdfWriter::new(options).write(scene)
}

/// System font database, loaded once per process.
///
/// Scanning system fonts takes milliseconds, so it must not happen per render.
/// Fonts are only needed for text, e.g. border decorations.
pub(crate) fn font_db() -> Arc<usvg::fontdb::Database> {
    static FONTS: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    FONTS
        .get_or_init(|| {
            let mut db = usvg::fontdb::Database::new();
            db.load_system_fonts();
            // Map each font file once and keep it. Otherwise every text layout
            // re-maps the file, and parallel renders contend on the OS mmap lock.
            let ids: Vec<_> = db.faces().map(|f| f.id).collect();
            for id in ids {
                // SAFETY: system font files are not modified while the process runs.
                unsafe {
                    db.make_shared_face_data(id);
                }
            }
            Arc::new(db)
        })
        .clone()
}

/// usvg parse options for `svg`. Fonts are only loaded (once) when the SVG has text.
pub(crate) fn usvg_options(svg: &str) -> usvg::Options<'static> {
    if svg.contains("<text") {
        usvg::Options {
            fontdb: font_db(),
            ..Default::default()
        }
    } else {
        usvg::Options::default()
    }
}
