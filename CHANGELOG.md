# Changelog

## Unreleased

### Features
- New optional `cmyk` feature: `render_pdf_cmyk` / `save_pdf_cmyk` write a
  print-ready PDF in DeviceCMYK directly from the QR geometry (no sRGB step).
  `CmykPdfOptions::with_color` maps colors to exact CMYK values; other colors
  are converted with full gray replacement, so black prints as 100% K.
  Supports gradients (axial/radial shadings), transparency and logos.

## 0.2.0

### Performance
- Each layer is drawn as a single filled element instead of a full-canvas
  rect clipped by a `clipPath`; all dots share one `<path>`.
  PNG is 2–3x faster, and SVG output is about half the size.
- System fonts load once per process, and only for SVGs containing text.
  Small PDFs are about 13x faster.
- `SvgRenderer` borrows options instead of cloning them, and the logo is
  embedded once instead of twice.

### Fixes
- Oversized `margin` no longer panics. The builder returns `CanvasTooSmall`.
- `Color::from_hex` no longer panics on non-ASCII input.
- PNG/WebP keep transparent backgrounds; semi-transparent colors are no
  longer darkened (premultiplied alpha).
- Border text, style and image `src` are XML-escaped.
- The logo area uses the image's real aspect ratio.
- Border decorations are emitted in a deterministic order.
- PNG output now renders border text (fonts were previously only loaded for PDF).

### Features
- `QROptions::mode` is now honored. An explicit mode encodes all data in
  that mode and errors on unsupported characters.
- `type_number > 40` returns `InvalidVersion`.
- The `png`, `jpeg` and `webp` features now actually gate the encoders.
- New default feature `pdf` (`PdfRenderer`, `OutputFormat::Pdf`).

### Breaking changes
- `QRDot::draw`, `QRCornerSquare::draw` and `QRCornerDot::draw` return a
  `<path>` element instead of `<circle>`/`<rect>`. New `push_path` methods
  append raw path data.
- `SvgRenderer` has a lifetime parameter. `SvgRenderer::from_ref` borrows
  the options; `SvgRenderer::new` still takes owned options.
- `PdfRenderer` requires the `pdf` feature (enabled by default).
- SVG markup changed (no `clipPath`s; coordinates rounded to 3 decimals).
  Rendered output is visually identical.

### Dependencies
- `resvg` 0.45 and `svg2pdf` 0.13 now share one `usvg`/`fontdb`.
- Removed the unused `tiny-skia`, `serde_json` and `proptest`; `lopdf` is
  now a dev-dependency.

## 0.1.1
- Sample images and README examples.
