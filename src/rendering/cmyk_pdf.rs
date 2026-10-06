//! Native PDF output in the DeviceCMYK color space, for print.
//!
//! Unlike [`PdfRenderer`](super::PdfRenderer), which converts the SVG through
//! svg2pdf (sRGB), this writes the QR code's shapes directly with
//! `pdf-writer`, so every color is an exact CMYK value. Pure black maps to
//! 100% K by default, which keeps thin modules sharp on press.
//!
//! Text from the SVG-only `BorderPlugin` is not part of this output.

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fmt;
use std::sync::Arc;

use pdf_writer::types::{FunctionShadingType, LineCapStyle, LineJoinStyle};
use pdf_writer::writers::Resources;
use pdf_writer::{Chunk, Content, Filter, Finish, Name, Pdf, Rect, Ref};

use super::scene::{Background, ImageItem, Paint, Scene};
use crate::config::{Color, ColorStop};
use crate::plugins::BorderPlugin;
use crate::error::{QRError, Result};

/// A CMYK color with components in percent (0–100).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cmyk {
    /// Cyan (0–100).
    pub c: f32,
    /// Magenta (0–100).
    pub m: f32,
    /// Yellow (0–100).
    pub y: f32,
    /// Black (0–100).
    pub k: f32,
}

impl Cmyk {
    /// Create a CMYK color; components are clamped to 0–100.
    pub fn new(c: f32, m: f32, y: f32, k: f32) -> Self {
        let clamp = |v: f32| v.clamp(0.0, 100.0);
        Self {
            c: clamp(c),
            m: clamp(m),
            y: clamp(y),
            k: clamp(k),
        }
    }

    /// Pure black ink only (0/0/0/100).
    pub const BLACK: Cmyk = Cmyk {
        c: 0.0,
        m: 0.0,
        y: 0.0,
        k: 100.0,
    };

    /// No ink (paper white).
    pub const WHITE: Cmyk = Cmyk {
        c: 0.0,
        m: 0.0,
        y: 0.0,
        k: 0.0,
    };

    /// Device-independent conversion with full gray component replacement:
    /// neutral colors (black, grays) use only the K channel. This is not a
    /// color-managed (ICC) conversion; map brand colors explicitly with
    /// [`CmykPdfOptions::with_color`].
    pub fn from_rgb(color: Color) -> Self {
        let r = color.r as f32 / 255.0;
        let g = color.g as f32 / 255.0;
        let b = color.b as f32 / 255.0;
        let k = 1.0 - r.max(g).max(b);
        if k >= 1.0 {
            return Self::BLACK;
        }
        let scale = 100.0 / (1.0 - k);
        Self::new(
            (1.0 - r - k) * scale,
            (1.0 - g - k) * scale,
            (1.0 - b - k) * scale,
            k * 100.0,
        )
    }

    fn components(self) -> [f32; 4] {
        [self.c / 100.0, self.m / 100.0, self.y / 100.0, self.k / 100.0]
    }
}

/// A function converting an RGB color to CMYK, e.g. an ICC-based transform.
pub type CmykConverter = Arc<dyn Fn(Color) -> Cmyk + Send + Sync>;

/// Options for [`QRCodeStyling::render_pdf_cmyk`](crate::QRCodeStyling::render_pdf_cmyk).
///
/// Every RGB color in the output (dots, corners, background, gradient stops
/// and logo pixels) is resolved in this order:
/// 1. the color map ([`with_color`](Self::with_color)), compared by RGB value;
/// 2. the custom converter ([`with_converter`](Self::with_converter)), if set;
/// 3. [`Cmyk::from_rgb`].
///
/// Alpha is kept as PDF transparency.
#[derive(Clone)]
pub struct CmykPdfOptions {
    color_map: Vec<(Color, Cmyk)>,
    converter: Option<CmykConverter>,
    compress: bool,
    compression_level: u8,
    overlays: Vec<Overlay>,
    fonts: Option<Arc<usvg::fontdb::Database>>,
}

/// SVG content drawn on top of the QR code.
#[derive(Debug, Clone)]
enum Overlay {
    Svg(String),
    Border(BorderPlugin),
}

impl Default for CmykPdfOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for CmykPdfOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CmykPdfOptions")
            .field("color_map", &self.color_map)
            .field("converter", &self.converter.as_ref().map(|_| "<fn>"))
            .field("compress", &self.compress)
            .field("compression_level", &self.compression_level)
            .field("overlays", &self.overlays)
            .field("fonts", &self.fonts.as_ref().map(|db| db.len()))
            .finish()
    }
}

impl CmykPdfOptions {
    /// Default options: automatic conversion, compressed content.
    pub fn new() -> Self {
        Self {
            color_map: Vec::new(),
            converter: None,
            compress: true,
            compression_level: DEFAULT_COMPRESSION_LEVEL,
            overlays: Vec::new(),
            fonts: None,
        }
    }

    /// Draw a [`BorderPlugin`] (frame, curved or straight text, image
    /// decorations) on top of the QR code, in CMYK.
    ///
    /// Text is converted to outlines with the font database (system fonts
    /// unless [`with_fonts`](Self::with_fonts) is set), so the PDF needs no
    /// embedded fonts.
    pub fn with_border(mut self, border: BorderPlugin) -> Self {
        self.overlays.push(Overlay::Border(border));
        self
    }

    /// Draw an arbitrary SVG document on top of the QR code, in CMYK.
    /// Its user space should match the QR code's size (width × height px).
    ///
    /// Supported: filled/stroked paths (solid colors and linear/radial
    /// gradient fills), text (as outlines), raster images, opacity.
    /// Not supported: clip paths, masks, filters, patterns.
    pub fn with_overlay_svg(mut self, svg: impl Into<String>) -> Self {
        self.overlays.push(Overlay::Svg(svg.into()));
        self
    }

    /// Fonts for overlay text, instead of the system fonts. Use this on
    /// servers without the fonts your border styles name.
    pub fn with_fonts(mut self, fonts: Arc<usvg::fontdb::Database>) -> Self {
        self.fonts = Some(fonts);
        self
    }

    /// Convert colors that aren't in the color map with `converter`
    /// instead of [`Cmyk::from_rgb`], e.g. an ICC (lcms) transform.
    ///
    /// The converter is called once per distinct color per render.
    pub fn with_converter(mut self, converter: impl Fn(Color) -> Cmyk + Send + Sync + 'static) -> Self {
        self.converter = Some(Arc::new(converter));
        self
    }

    /// Use exactly `cmyk` wherever `color` (compared by RGB) appears.
    pub fn with_color(mut self, color: Color, cmyk: Cmyk) -> Self {
        self.color_map.retain(|(c, _)| !same_rgb(*c, color));
        self.color_map.push((color, cmyk));
        self
    }

    /// Whether to Flate-compress streams (default: true).
    pub fn with_compression(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }

    /// Flate compression level, 0–10 (default: 6). Lower is faster and
    /// gives larger files; 1 is several times faster than 6 for bulk
    /// output. 0 turns compression off, like `with_compression(false)`.
    pub fn with_compression_level(mut self, level: u8) -> Self {
        self.compression_level = level.min(10);
        self.compress = level > 0;
        self
    }

    /// The CMYK value used for `color`.
    pub fn resolve(&self, color: Color) -> Cmyk {
        self.color_map
            .iter()
            .find(|(c, _)| same_rgb(*c, color))
            .map(|(_, cmyk)| *cmyk)
            .unwrap_or_else(|| match &self.converter {
                Some(convert) => convert(Color::rgb(color.r, color.g, color.b)),
                None => Cmyk::from_rgb(color),
            })
    }
}

const DEFAULT_COMPRESSION_LEVEL: u8 = 6;

fn same_rgb(a: Color, b: Color) -> bool {
    (a.r, a.g, a.b) == (b.r, b.g, b.b)
}

/// A QR code written as a PDF Form XObject by
/// [`QRCodeStyling::write_cmyk_xobject`](crate::QRCodeStyling::write_cmyk_xobject).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmykXObject {
    /// Reference of the Form XObject in the chunk.
    pub id: Ref,
    /// Width of the form's bounding box in points (the QR code's width in px).
    pub width: f32,
    /// Height of the form's bounding box in points.
    pub height: f32,
}

/// Raster images (logos, border decorations) already written into a PDF,
/// so codes sharing a logo reference one image XObject instead of each
/// decoding and embedding their own copy.
///
/// Pass the same cache to every
/// [`write_cmyk_xobject_cached`](crate::QRCodeStyling::write_cmyk_xobject_cached)
/// call whose objects end up in the same PDF. Don't share it between
/// different PDFs: the cached objects only exist in the first one.
///
/// Images are converted with the colors and compression of the options
/// they were first written with; if the options change, the cache starts
/// over (later codes embed fresh copies).
#[derive(Default)]
pub struct CmykImageCache {
    images: HashMap<Vec<u8>, Option<(Ref, u32, u32)>>,
    /// Color map, converter and compression the images were written with.
    written_with: Option<OptionsKey>,
}

type OptionsKey = (Vec<(Color, Cmyk)>, Option<usize>, Option<u8>);

impl CmykImageCache {
    /// An empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct images written.
    pub fn len(&self) -> usize {
        self.images.values().filter(|v| v.is_some()).count()
    }

    /// Whether no images have been written.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Forget the cached images if `options` would convert them differently.
    fn sync(&mut self, options: &CmykPdfOptions) {
        let key: OptionsKey = (
            options.color_map.clone(),
            options.converter.as_ref().map(|c| Arc::as_ptr(c) as *const () as usize),
            options.compress.then_some(options.compression_level),
        );
        if self.written_with.as_ref() != Some(&key) {
            self.images.clear();
            self.written_with = Some(key);
        }
    }
}

impl fmt::Debug for CmykImageCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CmykImageCache").field("images", &self.len()).finish()
    }
}

/// Write `scene` as a complete single-page PDF.
pub(crate) fn write_pdf(scene: &Scene<'_>, options: &CmykPdfOptions) -> Result<Vec<u8>> {
    let mut pdf = Pdf::new();
    let mut alloc = Ref::new(1);
    let (catalog_id, tree_id, page_id) = (alloc.bump(), alloc.bump(), alloc.bump());

    let mut cache = CmykImageCache::new();
    let mut writer = CmykPdfWriter::new(options, &mut pdf, &mut alloc, &mut cache);
    let content = writer.draw_scene(scene)?;
    let content_id = writer.write_stream(&content);
    let resources = writer.into_resources();

    pdf.catalog(catalog_id).pages(tree_id);
    pdf.pages(tree_id).kids([page_id]).count(1);
    let mut page = pdf.page(page_id);
    page.media_box(Rect::new(0.0, 0.0, scene.width as f32, scene.height as f32));
    page.parent(tree_id);
    page.contents(content_id);
    resources.write(page.resources());
    page.finish();

    Ok(pdf.finish())
}

/// Write `scene` as a self-contained Form XObject into `chunk`.
pub(crate) fn write_xobject(
    scene: &Scene<'_>,
    options: &CmykPdfOptions,
    chunk: &mut Chunk,
    alloc: &mut Ref,
    cache: &mut CmykImageCache,
) -> Result<CmykXObject> {
    let id = alloc.bump();
    let mut writer = CmykPdfWriter::new(options, chunk, alloc, cache);
    let content = writer.draw_scene(scene)?;
    let (data, compressed) = writer.maybe_compress(&content);
    let resources = writer.into_resources();

    let (width, height) = (scene.width as f32, scene.height as f32);
    let mut form = chunk.form_xobject(id, &data);
    if compressed {
        form.filter(Filter::FlateDecode);
    }
    form.bbox(Rect::new(0.0, 0.0, width, height));
    resources.write(form.resources());
    form.finish();

    Ok(CmykXObject { id, width, height })
}

/// Named resources a drawing refers to.
#[derive(Default)]
struct ResourceNames {
    ext_states: Vec<(String, Ref)>,
    shadings: Vec<(String, Ref)>,
    images: Vec<(String, Ref)>,
}

impl ResourceNames {
    fn write(&self, mut resources: Resources<'_>) {
        for (kind, entries) in [
            (0, &self.ext_states),
            (1, &self.shadings),
            (2, &self.images),
        ] {
            if entries.is_empty() {
                continue;
            }
            let mut dict = match kind {
                0 => resources.ext_g_states(),
                1 => resources.shadings(),
                _ => resources.x_objects(),
            };
            for (name, id) in entries {
                dict.pair(Name(name.as_bytes()), *id);
            }
            dict.finish();
        }
        resources.finish();
    }
}

/// Draws a [`Scene`] in DeviceCMYK, writing its resources (shadings,
/// functions, images, graphics states) into a chunk.
struct CmykPdfWriter<'a> {
    options: &'a CmykPdfOptions,
    chunk: &'a mut Chunk,
    alloc: &'a mut Ref,
    images: &'a mut CmykImageCache,
    names: ResourceNames,
}

impl<'a> CmykPdfWriter<'a> {
    fn new(options: &'a CmykPdfOptions, chunk: &'a mut Chunk, alloc: &'a mut Ref, images: &'a mut CmykImageCache) -> Self {
        images.sync(options);
        Self {
            options,
            chunk,
            alloc,
            images,
            names: ResourceNames::default(),
        }
    }

    fn alloc(&mut self) -> Ref {
        self.alloc.bump()
    }

    fn into_resources(self) -> ResourceNames {
        self.names
    }

    /// Write a content stream (compressed if enabled) and return its id.
    fn write_stream(&mut self, content: &[u8]) -> Ref {
        let id = self.alloc();
        let (data, compressed) = self.maybe_compress(content);
        let mut stream = self.chunk.stream(id, &data);
        if compressed {
            stream.filter(Filter::FlateDecode);
        }
        stream.finish();
        id
    }

    /// Draw the scene; returns the uncompressed content stream. Coordinates
    /// are flipped so the drawing fills `[0 0 width height]` upright.
    fn draw_scene(&mut self, scene: &Scene<'_>) -> Result<Vec<u8>> {
        let mut content = Content::new();
        // PDF's origin is bottom-left; flip so scene (SVG) coordinates apply.
        content.transform([1.0, 0.0, 0.0, -1.0, 0.0, scene.height as f32]);

        self.draw_background(&mut content, &scene.background);
        for shape in &scene.shapes {
            self.fill(&mut content, &shape.paint, shape.even_odd, |c| {
                emit_svg_path(c, &shape.d)
            });
        }
        if let Some(image) = &scene.image {
            self.draw_image(&mut content, image)?;
        }
        for overlay in &self.options.overlays {
            let svg = match overlay {
                Overlay::Svg(svg) => svg.clone(),
                Overlay::Border(border) => border.overlay_svg(scene.width, scene.height),
            };
            self.draw_overlay(&mut content, &svg)?;
        }
        Ok(content.finish())
    }

    fn maybe_compress(&self, data: &[u8]) -> (Vec<u8>, bool) {
        if self.options.compress {
            (miniz_oxide::deflate::compress_to_vec_zlib(data, self.options.compression_level), true)
        } else {
            (data.to_vec(), false)
        }
    }

    fn draw_background(&mut self, c: &mut Content, bg: &Background) {
        let (x, y, w, h) = (bg.x, bg.y, bg.width as f64, bg.height as f64);
        let rx = bg.rx.min(w / 2.0).min(h / 2.0);
        self.fill(c, &bg.paint, false, |c| {
            if rx > 0.0 {
                // Rounded rectangle, clockwise from the top-left edge
                c.move_to((x + rx) as f32, y as f32);
                c.line_to((x + w - rx) as f32, y as f32);
                arc_to(c, x + w - rx, y, rx, false, true, x + w, y + rx);
                c.line_to((x + w) as f32, (y + h - rx) as f32);
                arc_to(c, x + w, y + h - rx, rx, false, true, x + w - rx, y + h);
                c.line_to((x + rx) as f32, (y + h) as f32);
                arc_to(c, x + rx, y + h, rx, false, true, x, y + h - rx);
                c.line_to(x as f32, (y + rx) as f32);
                arc_to(c, x, y + rx, rx, false, true, x + rx, y);
                c.close_path();
            } else {
                c.rect(x as f32, y as f32, w as f32, h as f32);
            }
        });
    }

    /// Fill the path produced by `path` with `paint`.
    fn fill(&mut self, c: &mut Content, paint: &Paint, even_odd: bool, path: impl Fn(&mut Content)) {
        match paint {
            Paint::Solid(color) => {
                if color.a == 0 {
                    return;
                }
                c.save_state();
                self.set_alpha(c, color.a);
                let [cy, m, y, k] = self.options.resolve(*color).components();
                c.set_fill_cmyk(cy, m, y, k);
                path(c);
                if even_odd {
                    c.fill_even_odd();
                } else {
                    c.fill_nonzero();
                }
                c.restore_state();
            }
            Paint::Linear { x1, y1, x2, y2, stops, .. } => {
                let coords = [*x1 as f32, *y1 as f32, *x2 as f32, *y2 as f32];
                self.fill_gradient(c, FunctionShadingType::Axial, &coords, stops, even_odd, path);
            }
            Paint::Radial { cx, cy, r, stops, .. } => {
                let (cx, cy, r) = (*cx as f32, *cy as f32, *r as f32);
                let coords = [cx, cy, 0.0, cx, cy, r];
                self.fill_gradient(c, FunctionShadingType::Radial, &coords, stops, even_odd, path);
            }
        }
    }

    fn fill_gradient(
        &mut self,
        c: &mut Content,
        kind: FunctionShadingType,
        coords: &[f32],
        stops: &[ColorStop],
        even_odd: bool,
        path: impl Fn(&mut Content),
    ) {
        let Some(function) = self.write_gradient_function(stops) else {
            return;
        };
        let shading_id = self.alloc();
        let mut shading = self.chunk.function_shading(shading_id);
        shading.shading_type(kind);
        shading.color_space().device_cmyk();
        shading.coords(coords.iter().copied());
        shading.function(function);
        shading.extend([true, true]);
        shading.finish();

        let name = format!("Sh{}", self.names.shadings.len());
        self.names.shadings.push((name.clone(), shading_id));

        // Paint the shading clipped to the shape
        c.save_state();
        path(c);
        if even_odd {
            c.clip_even_odd();
        } else {
            c.clip_nonzero();
        }
        c.end_path();
        c.shading(Name(name.as_bytes()));
        c.restore_state();
    }

    /// Write a function mapping t ∈ [0, 1] to CMYK along the stops.
    /// Returns `None` if there are no stops.
    fn write_gradient_function(&mut self, stops: &[ColorStop]) -> Option<Ref> {
        let mut points: Vec<(f32, [f32; 4])> = stops
            .iter()
            .map(|s| (s.offset as f32, self.options.resolve(s.color).components()))
            .collect();
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        let first = *points.first()?;
        let last = *points.last()?;
        // Hold the end colors flat outside the stop range
        if first.0 > 0.0 {
            points.insert(0, (0.0, first.1));
        }
        if last.0 < 1.0 {
            points.push((1.0, last.1));
        }
        if points.len() == 1 {
            points.push((1.0, points[0].1));
        }

        let segments: Vec<Ref> = points
            .windows(2)
            .map(|pair| {
                let id = self.alloc();
                let mut f = self.chunk.exponential_function(id);
                f.domain([0.0, 1.0]);
                f.c0(pair[0].1);
                f.c1(pair[1].1);
                f.n(1.0);
                f.finish();
                id
            })
            .collect();

        if segments.len() == 1 {
            return Some(segments[0]);
        }

        let id = self.alloc();
        let bounds: Vec<f32> = points[1..points.len() - 1].iter().map(|p| p.0).collect();
        let mut f = self.chunk.stitching_function(id);
        f.domain([0.0, 1.0]);
        f.functions(segments.iter().copied());
        f.bounds(bounds);
        f.encode(segments.iter().flat_map(|_| [0.0, 1.0]));
        f.finish();
        Some(id)
    }

    fn set_alpha(&mut self, c: &mut Content, alpha: u8) {
        self.set_alpha_for(c, alpha, false);
    }

    /// Set fill (`ca`) or stroke (`CA`) opacity via a shared ExtGState.
    fn set_alpha_for(&mut self, c: &mut Content, alpha: u8, stroke: bool) {
        if alpha == 255 {
            return;
        }
        let name = format!("{}{}", if stroke { "GS" } else { "Gs" }, alpha);
        if !self.names.ext_states.iter().any(|(n, _)| *n == name) {
            let id = self.alloc();
            let mut gs = self.chunk.ext_graphics(id);
            if stroke {
                gs.stroking_alpha(alpha as f32 / 255.0);
            } else {
                gs.non_stroking_alpha(alpha as f32 / 255.0);
            }
            gs.finish();
            self.names.ext_states.push((name.clone(), id));
        }
        c.set_parameters(Name(name.as_bytes()));
    }

    fn draw_image(&mut self, c: &mut Content, image: &ImageItem<'_>) -> Result<()> {
        if image.width <= 0.0 || image.height <= 0.0 {
            return Ok(());
        }
        let Some((name, iw, ih)) = self.embed_image(image.data)? else {
            return Ok(());
        };

        // Fit inside the box, centered (SVG `xMidYMid meet`)
        let scale = (image.width / iw as f64).min(image.height / ih as f64);
        let (dw, dh) = (iw as f64 * scale, ih as f64 * scale);
        let dx = image.x + (image.width - dw) / 2.0;
        let dy = image.y + (image.height - dh) / 2.0;

        c.save_state();
        // The page is y-flipped, so flip the unit square back: the image's
        // top row lands at `dy`.
        c.transform([dw as f32, 0.0, 0.0, -dh as f32, dx as f32, (dy + dh) as f32]);
        c.x_object(Name(name.as_bytes()));
        c.restore_state();
        Ok(())
    }

    /// Add raster image data to this drawing's resources, embedding it
    /// unless the image cache already has it. Returns its resource name
    /// and pixel size.
    fn embed_image(&mut self, data: &[u8]) -> Result<Option<(String, u32, u32)>> {
        let cached = match self.images.images.get(data) {
            Some(entry) => *entry,
            None => {
                let entry = self.write_image(data)?;
                self.images.images.insert(data.to_vec(), entry);
                entry
            }
        };
        let Some((image_id, iw, ih)) = cached else {
            return Ok(None);
        };
        let name = match self.names.images.iter().find(|(_, id)| *id == image_id) {
            Some((name, _)) => name.clone(),
            None => {
                let name = format!("Im{}", self.names.images.len());
                self.names.images.push((name.clone(), image_id));
                name
            }
        };
        Ok(Some((name, iw, ih)))
    }

    /// Write raster image data as a CMYK image XObject (with an alpha soft
    /// mask if needed). Returns its id and pixel size.
    fn write_image(&mut self, data: &[u8]) -> Result<Option<(Ref, u32, u32)>> {
        let decoded = image::load_from_memory(data)
            .map_err(|e| QRError::ImageLoadError(e.to_string()))?
            .to_rgba8();
        let (iw, ih) = decoded.dimensions();
        if iw == 0 || ih == 0 {
            return Ok(None);
        }

        let mut cmyk = Vec::with_capacity((iw * ih * 4) as usize);
        let mut alpha = Vec::with_capacity((iw * ih) as usize);
        // Logos have few distinct colors; resolve each once (custom
        // converters such as ICC transforms can be slow per call).
        let mut cache: HashMap<[u8; 3], [u8; 4]> = HashMap::new();
        for p in decoded.pixels() {
            let [r, g, b, a] = p.0;
            let v = *cache.entry([r, g, b]).or_insert_with(|| {
                self.options
                    .resolve(Color::rgb(r, g, b))
                    .components()
                    .map(|x| (x * 255.0).round() as u8)
            });
            cmyk.extend(v);
            alpha.push(a);
        }
        let has_alpha = alpha.iter().any(|&a| a != 255);

        let mask_id = if has_alpha {
            let id = self.alloc();
            let (data, compressed) = self.maybe_compress(&alpha);
            let mut mask = self.chunk.image_xobject(id, &data);
            if compressed {
                mask.filter(Filter::FlateDecode);
            }
            mask.width(iw as i32);
            mask.height(ih as i32);
            mask.color_space().device_gray();
            mask.bits_per_component(8);
            mask.finish();
            Some(id)
        } else {
            None
        };

        let image_id = self.alloc();
        let (data, compressed) = self.maybe_compress(&cmyk);
        let mut xobj = self.chunk.image_xobject(image_id, &data);
        if compressed {
            xobj.filter(Filter::FlateDecode);
        }
        xobj.width(iw as i32);
        xobj.height(ih as i32);
        xobj.color_space().device_cmyk();
        xobj.bits_per_component(8);
        if let Some(mask) = mask_id {
            xobj.s_mask(mask);
        }
        xobj.finish();
        Ok(Some((image_id, iw, ih)))
    }

    /// Parse `svg` with usvg (text becomes outlines) and draw it in CMYK.
    fn draw_overlay(&mut self, c: &mut Content, svg: &str) -> Result<()> {
        let fonts = if svg.contains("<text") {
            self.options.fonts.clone().unwrap_or_else(super::font_db)
        } else {
            Arc::new(usvg::fontdb::Database::new())
        };
        let options = usvg::Options {
            fontdb: fonts,
            ..Default::default()
        };
        let tree = usvg::Tree::from_str(svg, &options).map_err(|e| QRError::SvgError(e.to_string()))?;
        self.draw_usvg_group(c, tree.root(), 1.0, usvg::Transform::identity())
    }

    /// Draw a group, accumulating group transforms like resvg does.
    /// (`abs_transform` of a text's flattened paths lacks the text's own
    /// transform, e.g. the rotation of vertical border text.)
    fn draw_usvg_group(&mut self, c: &mut Content, group: &usvg::Group, opacity: f32, parent: usvg::Transform) -> Result<()> {
        let ts = parent.pre_concat(group.transform());
        let opacity = opacity * group.opacity().get();
        for node in group.children() {
            match node {
                usvg::Node::Group(g) => self.draw_usvg_group(c, g, opacity, ts)?,
                usvg::Node::Path(path) => self.draw_usvg_path(c, path, opacity, ts),
                usvg::Node::Text(text) => self.draw_usvg_group(c, text.flattened(), opacity, ts)?,
                usvg::Node::Image(image) => self.draw_usvg_image(c, image, ts, opacity)?,
            }
        }
        Ok(())
    }

    fn draw_usvg_path(&mut self, c: &mut Content, path: &usvg::Path, opacity: f32, ts: usvg::Transform) {
        if !path.is_visible() {
            return;
        }
        let matrix = [ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty];
        let fill_first = path.paint_order() == usvg::PaintOrder::FillAndStroke;

        let draw_fill = |w: &mut Self, c: &mut Content| {
            if let Some(fill) = path.fill() {
                let alpha = to_alpha(opacity * fill.opacity().get());
                w.fill_usvg(c, fill.paint(), alpha, fill.rule() == usvg::FillRule::EvenOdd, matrix, path.data());
            }
        };
        let draw_stroke = |w: &mut Self, c: &mut Content| {
            if let Some(stroke) = path.stroke() {
                w.stroke_usvg(c, stroke, opacity, matrix, path.data());
            }
        };

        if fill_first {
            draw_fill(self, c);
            draw_stroke(self, c);
        } else {
            draw_stroke(self, c);
            draw_fill(self, c);
        }
    }

    fn fill_usvg(
        &mut self,
        c: &mut Content,
        paint: &usvg::Paint,
        alpha: u8,
        even_odd: bool,
        matrix: [f32; 6],
        data: &usvg::tiny_skia_path::Path,
    ) {
        if alpha == 0 {
            return;
        }
        match paint {
            usvg::Paint::LinearGradient(g) => {
                let stops = usvg_stops(g.stops());
                let coords = [g.x1(), g.y1(), g.x2(), g.y2()];
                self.fill_usvg_gradient(c, FunctionShadingType::Axial, &coords, &stops, g.transform(), alpha, even_odd, matrix, data);
            }
            usvg::Paint::RadialGradient(g) => {
                let stops = usvg_stops(g.stops());
                let coords = [g.fx(), g.fy(), 0.0, g.cx(), g.cy(), g.r().get()];
                self.fill_usvg_gradient(c, FunctionShadingType::Radial, &coords, &stops, g.transform(), alpha, even_odd, matrix, data);
            }
            other => {
                c.save_state();
                c.transform(matrix);
                self.set_alpha_for(c, alpha, false);
                let [cy, m, y, k] = self.options.resolve(solid_color(other)).components();
                c.set_fill_cmyk(cy, m, y, k);
                emit_usvg_path(c, data);
                if even_odd {
                    c.fill_even_odd();
                } else {
                    c.fill_nonzero();
                }
                c.restore_state();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn fill_usvg_gradient(
        &mut self,
        c: &mut Content,
        kind: FunctionShadingType,
        coords: &[f32],
        stops: &[ColorStop],
        gradient_transform: usvg::Transform,
        alpha: u8,
        even_odd: bool,
        matrix: [f32; 6],
        data: &usvg::tiny_skia_path::Path,
    ) {
        let Some(function) = self.write_gradient_function(stops) else {
            return;
        };
        let shading_id = self.alloc();
        let mut shading = self.chunk.function_shading(shading_id);
        shading.shading_type(kind);
        shading.color_space().device_cmyk();
        shading.coords(coords.iter().copied());
        shading.function(function);
        shading.extend([true, true]);
        shading.finish();
        let name = format!("Sh{}", self.names.shadings.len());
        self.names.shadings.push((name.clone(), shading_id));

        let g = gradient_transform;
        c.save_state();
        c.transform(matrix);
        self.set_alpha_for(c, alpha, false);
        emit_usvg_path(c, data);
        if even_odd {
            c.clip_even_odd();
        } else {
            c.clip_nonzero();
        }
        c.end_path();
        c.transform([g.sx, g.ky, g.kx, g.sy, g.tx, g.ty]);
        c.shading(Name(name.as_bytes()));
        c.restore_state();
    }

    fn stroke_usvg(
        &mut self,
        c: &mut Content,
        stroke: &usvg::Stroke,
        opacity: f32,
        matrix: [f32; 6],
        data: &usvg::tiny_skia_path::Path,
    ) {
        let alpha = to_alpha(opacity * stroke.opacity().get());
        if alpha == 0 {
            return;
        }
        c.save_state();
        c.transform(matrix);
        self.set_alpha_for(c, alpha, true);
        // Gradient strokes are drawn in their first stop's color
        let [cy, m, y, k] = self.options.resolve(solid_color(stroke.paint())).components();
        c.set_stroke_cmyk(cy, m, y, k);
        c.set_line_width(stroke.width().get());
        c.set_line_cap(match stroke.linecap() {
            usvg::LineCap::Butt => LineCapStyle::ButtCap,
            usvg::LineCap::Round => LineCapStyle::RoundCap,
            usvg::LineCap::Square => LineCapStyle::ProjectingSquareCap,
        });
        c.set_line_join(match stroke.linejoin() {
            usvg::LineJoin::Round => LineJoinStyle::RoundJoin,
            usvg::LineJoin::Bevel => LineJoinStyle::BevelJoin,
            _ => LineJoinStyle::MiterJoin,
        });
        c.set_miter_limit(stroke.miterlimit().get());
        if let Some(dashes) = stroke.dasharray() {
            c.set_dash_pattern(dashes.iter().copied(), stroke.dashoffset());
        }
        emit_usvg_path(c, data);
        c.stroke();
        c.restore_state();
    }

    fn draw_usvg_image(&mut self, c: &mut Content, image: &usvg::Image, ts: usvg::Transform, opacity: f32) -> Result<()> {
        if !image.is_visible() {
            return Ok(());
        }
        let data = match image.kind() {
            usvg::ImageKind::JPEG(d) | usvg::ImageKind::PNG(d) | usvg::ImageKind::GIF(d) | usvg::ImageKind::WEBP(d) => d,
            usvg::ImageKind::SVG(_) => return Ok(()),
        };
        let Some((name, _, _)) = self.embed_image(data)? else {
            return Ok(());
        };
        let size = image.size();
        c.save_state();
        c.transform([ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty]);
        self.set_alpha_for(c, to_alpha(opacity), false);
        // Unit square -> image box, top row first in the y-flipped space
        c.transform([size.width(), 0.0, 0.0, -size.height(), 0.0, size.height()]);
        c.x_object(Name(name.as_bytes()));
        c.restore_state();
        Ok(())
    }
}

fn to_alpha(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn usvg_color(color: usvg::Color) -> Color {
    Color::rgb(color.red, color.green, color.blue)
}

/// A paint's color for solid drawing; gradients/patterns use their first stop.
fn solid_color(paint: &usvg::Paint) -> Color {
    match paint {
        usvg::Paint::Color(color) => usvg_color(*color),
        usvg::Paint::LinearGradient(g) => g.stops().first().map_or(Color::BLACK, |s| usvg_color(s.color())),
        usvg::Paint::RadialGradient(g) => g.stops().first().map_or(Color::BLACK, |s| usvg_color(s.color())),
        usvg::Paint::Pattern(_) => Color::BLACK,
    }
}

fn usvg_stops(stops: &[usvg::Stop]) -> Vec<ColorStop> {
    stops
        .iter()
        .map(|s| ColorStop::new(s.offset().get() as f64, usvg_color(s.color())))
        .collect()
}

/// Emit a usvg (tiny-skia) path; quadratic segments become cubics.
fn emit_usvg_path(c: &mut Content, path: &usvg::tiny_skia_path::Path) {
    use usvg::tiny_skia_path::PathSegment;
    let mut last = (0.0f32, 0.0f32);
    for segment in path.segments() {
        match segment {
            PathSegment::MoveTo(p) => {
                c.move_to(p.x, p.y);
                last = (p.x, p.y);
            }
            PathSegment::LineTo(p) => {
                c.line_to(p.x, p.y);
                last = (p.x, p.y);
            }
            PathSegment::QuadTo(p1, p) => {
                let c1 = (last.0 + 2.0 / 3.0 * (p1.x - last.0), last.1 + 2.0 / 3.0 * (p1.y - last.1));
                let c2 = (p.x + 2.0 / 3.0 * (p1.x - p.x), p.y + 2.0 / 3.0 * (p1.y - p.y));
                c.cubic_to(c1.0, c1.1, c2.0, c2.1, p.x, p.y);
                last = (p.x, p.y);
            }
            PathSegment::CubicTo(p1, p2, p) => {
                c.cubic_to(p1.x, p1.y, p2.x, p2.y, p.x, p.y);
                last = (p.x, p.y);
            }
            PathSegment::Close => {
                c.close_path();
            }
        }
    }
}

/// Convert path data from `PathBuilder` (`M m l a z`, relative except `M`)
/// into PDF path operators. Arcs become cubic Béziers.
fn emit_svg_path(c: &mut Content, d: &str) {
    let mut tokens = PathTokens { s: d.as_bytes(), i: 0 };
    let (mut cx, mut cy) = (0.0f64, 0.0f64);
    let (mut sx, mut sy) = (0.0f64, 0.0f64);

    while let Some(cmd) = tokens.command() {
        match cmd {
            b'M' | b'm' => {
                let (Some(x), Some(y)) = (tokens.number(), tokens.number()) else { break };
                if cmd == b'M' {
                    (cx, cy) = (x, y);
                } else {
                    (cx, cy) = (cx + x, cy + y);
                }
                (sx, sy) = (cx, cy);
                c.move_to(cx as f32, cy as f32);
            }
            b'l' => {
                let (Some(dx), Some(dy)) = (tokens.number(), tokens.number()) else { break };
                (cx, cy) = (cx + dx, cy + dy);
                c.line_to(cx as f32, cy as f32);
            }
            b'a' => {
                let nums: Vec<f64> = (0..7).filter_map(|_| tokens.number()).collect();
                let [r, _, _, large, sweep, dx, dy] = nums[..] else { break };
                let (x2, y2) = (cx + dx, cy + dy);
                arc_to(c, cx, cy, r, large != 0.0, sweep != 0.0, x2, y2);
                (cx, cy) = (x2, y2);
            }
            b'z' | b'Z' => {
                c.close_path();
                (cx, cy) = (sx, sy);
            }
            _ => break,
        }
    }
}

struct PathTokens<'a> {
    s: &'a [u8],
    i: usize,
}

impl PathTokens<'_> {
    fn skip_separators(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b',' | b'\n') {
            self.i += 1;
        }
    }

    fn command(&mut self) -> Option<u8> {
        self.skip_separators();
        let b = *self.s.get(self.i)?;
        if b.is_ascii_alphabetic() {
            self.i += 1;
            Some(b)
        } else {
            None
        }
    }

    fn number(&mut self) -> Option<f64> {
        self.skip_separators();
        let start = self.i;
        if matches!(self.s.get(self.i), Some(b'-' | b'+')) {
            self.i += 1;
        }
        while matches!(self.s.get(self.i), Some(b'0'..=b'9' | b'.')) {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[start..self.i]).ok()?.parse().ok()
    }
}

/// Append a circular arc from (x1, y1) to (x2, y2) as cubic Béziers,
/// following the SVG endpoint parameterization (SVG 1.1, appendix F.6.5).
#[allow(clippy::too_many_arguments)]
fn arc_to(c: &mut Content, x1: f64, y1: f64, r: f64, large: bool, sweep: bool, x2: f64, y2: f64) {
    if (x1 - x2).abs() < 1e-9 && (y1 - y2).abs() < 1e-9 {
        return;
    }
    let mut r = r.abs();
    if r < 1e-9 {
        c.line_to(x2 as f32, y2 as f32);
        return;
    }

    let x1p = (x1 - x2) / 2.0;
    let y1p = (y1 - y2) / 2.0;
    let d2 = x1p * x1p + y1p * y1p;
    // Radius too small for the endpoints: scale it up (F.6.6)
    if d2 > r * r {
        r = d2.sqrt();
    }
    let sign = if large == sweep { -1.0 } else { 1.0 };
    let coef = sign * ((r * r - d2) / d2).max(0.0).sqrt();
    let cxp = coef * y1p;
    let cyp = -coef * x1p;
    let cx = cxp + (x1 + x2) / 2.0;
    let cy = cyp + (y1 + y2) / 2.0;

    let angle = |ux: f64, uy: f64, vx: f64, vy: f64| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let theta1 = angle(1.0, 0.0, (x1p - cxp) / r, (y1p - cyp) / r);
    let mut dtheta = angle((x1p - cxp) / r, (y1p - cyp) / r, (-x1p - cxp) / r, (-y1p - cyp) / r);
    if !sweep && dtheta > 0.0 {
        dtheta -= 2.0 * PI;
    } else if sweep && dtheta < 0.0 {
        dtheta += 2.0 * PI;
    }

    let segments = (dtheta.abs() / (PI / 2.0)).ceil().max(1.0) as usize;
    let delta = dtheta / segments as f64;
    let t = 4.0 / 3.0 * (delta / 4.0).tan();
    for i in 0..segments {
        let a1 = theta1 + i as f64 * delta;
        let a2 = a1 + delta;
        let (s1, c1) = a1.sin_cos();
        let (s2, c2) = a2.sin_cos();
        let (p1x, p1y) = (cx + r * c1, cy + r * s1);
        let (mut p2x, mut p2y) = (cx + r * c2, cy + r * s2);
        if i == segments - 1 {
            (p2x, p2y) = (x2, y2);
        }
        c.cubic_to(
            (p1x - t * r * s1) as f32,
            (p1y + t * r * c1) as f32,
            (p2x + t * r * s2) as f32,
            (p2y - t * r * c2) as f32,
            p2x as f32,
            p2y as f32,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_rgb_neutrals_use_k_only() {
        assert_eq!(Cmyk::from_rgb(Color::BLACK), Cmyk::BLACK);
        assert_eq!(Cmyk::from_rgb(Color::WHITE), Cmyk::WHITE);
        let gray = Cmyk::from_rgb(Color::rgb(128, 128, 128));
        assert_eq!((gray.c, gray.m, gray.y), (0.0, 0.0, 0.0));
        assert!((gray.k - 49.8).abs() < 0.1);
        let cyan = Cmyk::from_rgb(Color::rgb(0, 255, 255));
        assert_eq!(cyan, Cmyk::new(100.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn test_color_map_overrides_conversion() {
        let brand = Cmyk::new(100.0, 72.0, 0.0, 18.0);
        let options = CmykPdfOptions::new()
            .with_color(Color::rgb(0, 59, 209), brand)
            .with_color(Color::BLACK, Cmyk::new(60.0, 40.0, 40.0, 100.0));
        assert_eq!(options.resolve(Color::rgb(0, 59, 209)), brand);
        // Alpha doesn't affect the lookup
        assert_eq!(options.resolve(Color::rgba(0, 59, 209, 10)), brand);
        assert_eq!(options.resolve(Color::BLACK).c, 60.0);
        assert_eq!(options.resolve(Color::WHITE), Cmyk::WHITE);
    }

    #[test]
    fn test_path_tokens() {
        let mut t = PathTokens { s: b"M10 0l-2.5 3a1 1 0 1 0 2 0z", i: 0 };
        assert_eq!(t.command(), Some(b'M'));
        assert_eq!((t.number(), t.number()), (Some(10.0), Some(0.0)));
        assert_eq!(t.command(), Some(b'l'));
        assert_eq!((t.number(), t.number()), (Some(-2.5), Some(3.0)));
        assert_eq!(t.command(), Some(b'a'));
        let nums: Vec<f64> = (0..7).filter_map(|_| t.number()).collect();
        assert_eq!(nums, [1.0, 1.0, 0.0, 1.0, 0.0, 2.0, 0.0]);
        assert_eq!(t.command(), Some(b'z'));
        assert_eq!(t.command(), None);
    }

    #[test]
    fn test_semicircle_arc_endpoints() {
        // Half circle of radius 5 from (0,5) to (10,5) ends exactly at the endpoint
        let mut c = Content::new();
        c.move_to(0.0, 5.0);
        arc_to(&mut c, 0.0, 5.0, 5.0, true, false, 10.0, 5.0);
        let ops = String::from_utf8(c.finish()).unwrap();
        assert_eq!(ops.matches(" c").count(), 2);
        assert!(ops.trim_end().ends_with("10 5 c"));
    }

    #[test]
    fn test_custom_converter() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let options = CmykPdfOptions::new()
            .with_color(Color::BLACK, Cmyk::new(60.0, 40.0, 40.0, 100.0))
            .with_converter(move |c| {
                counter.fetch_add(1, Ordering::Relaxed);
                // Pretend ICC transform: everything becomes 10% of each ink
                assert_eq!(c.a, 255, "converter receives opaque colors");
                Cmyk::new(10.0, 10.0, 10.0, 10.0)
            });
        // Color map wins over the converter
        assert_eq!(options.resolve(Color::BLACK).c, 60.0);
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        // Unmapped colors (alpha ignored) go through the converter
        assert_eq!(options.resolve(Color::rgba(200, 10, 10, 50)), Cmyk::new(10.0, 10.0, 10.0, 10.0));
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        // Options stay cloneable and debuggable
        let cloned = options.clone();
        assert!(format!("{:?}", cloned).contains("<fn>"));
    }

    #[test]
    fn test_default_matches_new() {
        let d = format!("{:?}", CmykPdfOptions::default());
        assert_eq!(d, format!("{:?}", CmykPdfOptions::new()));
        assert!(d.contains("compress: true"));
    }
}
