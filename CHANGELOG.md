# Changelog

## Unreleased

- `CmykPdfOptions::with_converter`: plug in your own RGB→CMYK conversion
  (e.g. an ICC/lcms transform) for colors that aren't in the color map,
  including gradient stops and logo pixels.
- Fix: `CmykPdfOptions::default()` now compresses streams like `new()`.
- `QRCodeStyling::write_cmyk_xobject` writes a QR code as a self-contained
  CMYK Form XObject into your own `pdf_writer` document (e.g. many codes on
  one sheet). `pdf_writer` is re-exported with the `cmyk` feature. See
  `examples/cmyk_sheet.rs`.
- Opt-in JS compatibility: `QRCodeStylingBuilder::js_compatible(true)`
  produces output identical to JS `qr-code-styling` 1.9.2: the same matrix as
  `qrcode-generator` (single-mode encoding, its mask penalty) and the same
  circle-shape dot ring (rounded center, transposed sampling). Verified
  against 2,224 `qrcode-generator` matrices and 96 pixel-identical JS renders.
  Also available as `QRMatrix::new_js_compatible` and
  `SvgRenderer::js_compatible`.
- `resvg`/`usvg` are now optional. They come in only with the raster
  formats (`png`, `jpeg`, `webp`, via a new `raster` feature) or `pdf`. A
  build with just `cmyk` needs neither. Default features are unchanged.
  Note: with `default-features = false`, `RasterRenderer` now needs one of
  the raster features.

## 0.2.1

### Features
- New optional `cmyk` feature: `render_pdf_cmyk` / `save_pdf_cmyk` write a
  print-ready PDF in DeviceCMYK directly from the QR geometry (no sRGB step).
  `CmykPdfOptions::with_color` maps colors to exact CMYK values; other colors
  are converted with full gray replacement, so black prints as 100% K.
  Supports gradients (axial/radial shadings), transparency and logos.

### Performance
- System font files are memory-mapped once instead of on every text
  layout. Parallel PDF generation with border text: about +70% throughput
  (1,049 → 1,776 PDF/s on 10 cores).

### Docs
- docs.rs builds with all features, so the CMYK API is documented.

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
