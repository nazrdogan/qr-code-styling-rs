//! Native PDF output in the DeviceCMYK color space, for print.
//!
//! Unlike [`PdfRenderer`](super::PdfRenderer), which converts the SVG through
//! svg2pdf (sRGB), this writes the QR code's shapes directly with
//! `pdf-writer`, so every color is an exact CMYK value. Pure black maps to
//! 100% K by default, which keeps thin modules sharp on press.
//!
//! Text from the SVG-only `BorderPlugin` is not part of this output.

use std::f64::consts::PI;

use pdf_writer::types::FunctionShadingType;
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref};

use super::scene::{Background, ImageItem, Paint, Scene};
use crate::config::{Color, ColorStop};
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

/// Options for [`QRCodeStyling::render_pdf_cmyk`](crate::QRCodeStyling::render_pdf_cmyk).
///
/// Colors from the styling options (dots, corners, background, gradient
/// stops) are looked up in the color map by RGB value; unmapped colors are
/// converted with [`Cmyk::from_rgb`]. Alpha is kept as PDF transparency.
#[derive(Debug, Clone, Default)]
pub struct CmykPdfOptions {
    color_map: Vec<(Color, Cmyk)>,
    compress: bool,
}

impl CmykPdfOptions {
    /// Default options: automatic conversion, compressed content.
    pub fn new() -> Self {
        Self {
            color_map: Vec::new(),
            compress: true,
        }
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

    /// The CMYK value used for `color`.
    pub fn resolve(&self, color: Color) -> Cmyk {
        self.color_map
            .iter()
            .find(|(c, _)| same_rgb(*c, color))
            .map(|(_, cmyk)| *cmyk)
            .unwrap_or_else(|| Cmyk::from_rgb(color))
    }
}

fn same_rgb(a: Color, b: Color) -> bool {
    (a.r, a.g, a.b) == (b.r, b.g, b.b)
}

/// Writes a [`Scene`] as a single-page CMYK PDF.
pub(crate) struct CmykPdfWriter<'o> {
    options: &'o CmykPdfOptions,
    pdf: Pdf,
    next_id: i32,
    ext_states: Vec<(String, Ref)>,
    shadings: Vec<(String, Ref)>,
    images: Vec<(String, Ref)>,
}

impl<'o> CmykPdfWriter<'o> {
    pub fn new(options: &'o CmykPdfOptions) -> Self {
        Self {
            options,
            pdf: Pdf::new(),
            // 1–4 are reserved for catalog, page tree, page and content
            next_id: 5,
            ext_states: Vec::new(),
            shadings: Vec::new(),
            images: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Ref {
        let id = Ref::new(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn write(mut self, scene: &Scene<'_>) -> Result<Vec<u8>> {
        let (catalog_id, tree_id, page_id, content_id) =
            (Ref::new(1), Ref::new(2), Ref::new(3), Ref::new(4));
        let (w, h) = (scene.width as f32, scene.height as f32);

        let mut content = Content::new();
        // PDF's origin is bottom-left; flip so scene (SVG) coordinates apply.
        content.transform([1.0, 0.0, 0.0, -1.0, 0.0, h]);

        self.draw_background(&mut content, &scene.background);
        for shape in &scene.shapes {
            self.fill(&mut content, &shape.paint, shape.even_odd, |c| {
                emit_svg_path(c, &shape.d)
            });
        }
        if let Some(image) = &scene.image {
            self.draw_image(&mut content, image)?;
        }

        let content = content.finish();
        let (data, compressed) = self.maybe_compress(&content);
        let mut stream = self.pdf.stream(content_id, &data);
        if compressed {
            stream.filter(Filter::FlateDecode);
        }
        stream.finish();

        self.pdf.catalog(catalog_id).pages(tree_id);
        self.pdf.pages(tree_id).kids([page_id]).count(1);

        let mut page = self.pdf.page(page_id);
        page.media_box(Rect::new(0.0, 0.0, w, h));
        page.parent(tree_id);
        page.contents(content_id);
        let mut resources = page.resources();
        let mut dict = resources.ext_g_states();
        for (name, id) in &self.ext_states {
            dict.pair(Name(name.as_bytes()), *id);
        }
        dict.finish();
        let mut dict = resources.shadings();
        for (name, id) in &self.shadings {
            dict.pair(Name(name.as_bytes()), *id);
        }
        dict.finish();
        let mut dict = resources.x_objects();
        for (name, id) in &self.images {
            dict.pair(Name(name.as_bytes()), *id);
        }
        dict.finish();
        resources.finish();
        page.finish();

        Ok(self.pdf.finish())
    }

    fn maybe_compress(&self, data: &[u8]) -> (Vec<u8>, bool) {
        if self.options.compress {
            (miniz_oxide::deflate::compress_to_vec_zlib(data, 6), true)
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
        let mut shading = self.pdf.function_shading(shading_id);
        shading.shading_type(kind);
        shading.color_space().device_cmyk();
        shading.coords(coords.iter().copied());
        shading.function(function);
        shading.extend([true, true]);
        shading.finish();

        let name = format!("Sh{}", self.shadings.len());
        self.shadings.push((name.clone(), shading_id));

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
                let mut f = self.pdf.exponential_function(id);
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
        let mut f = self.pdf.stitching_function(id);
        f.domain([0.0, 1.0]);
        f.functions(segments.iter().copied());
        f.bounds(bounds);
        f.encode(segments.iter().flat_map(|_| [0.0, 1.0]));
        f.finish();
        Some(id)
    }

    fn set_alpha(&mut self, c: &mut Content, alpha: u8) {
        if alpha == 255 {
            return;
        }
        let name = format!("Gs{}", alpha);
        if !self.ext_states.iter().any(|(n, _)| *n == name) {
            let id = self.alloc();
            self.pdf
                .ext_graphics(id)
                .non_stroking_alpha(alpha as f32 / 255.0);
            self.ext_states.push((name.clone(), id));
        }
        c.set_parameters(Name(name.as_bytes()));
    }

    fn draw_image(&mut self, c: &mut Content, image: &ImageItem<'_>) -> Result<()> {
        let decoded = image::load_from_memory(image.data)
            .map_err(|e| QRError::ImageLoadError(e.to_string()))?
            .to_rgba8();
        let (iw, ih) = decoded.dimensions();
        if iw == 0 || ih == 0 || image.width <= 0.0 || image.height <= 0.0 {
            return Ok(());
        }

        let mut cmyk = Vec::with_capacity((iw * ih * 4) as usize);
        let mut alpha = Vec::with_capacity((iw * ih) as usize);
        for p in decoded.pixels() {
            let [r, g, b, a] = p.0;
            let v = self.options.resolve(Color::rgb(r, g, b)).components();
            cmyk.extend(v.map(|x| (x * 255.0).round() as u8));
            alpha.push(a);
        }
        let has_alpha = alpha.iter().any(|&a| a != 255);

        let mask_id = if has_alpha {
            let id = self.alloc();
            let (data, compressed) = self.maybe_compress(&alpha);
            let mut mask = self.pdf.image_xobject(id, &data);
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
        let mut xobj = self.pdf.image_xobject(image_id, &data);
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

        let name = format!("Im{}", self.images.len());
        self.images.push((name.clone(), image_id));

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
}
