//! Traits for figure drawing.

use std::f64::consts::PI;

/// Trait for drawing QR code figures.
pub trait FigureDrawer {
    /// Draw the figure and return SVG element string.
    fn draw<F>(&self, x: f64, y: f64, size: f64, get_neighbor: Option<F>) -> String
    where
        F: Fn(i32, i32) -> bool;
}

/// Legacy type alias for compatibility.
pub type NeighborFn = dyn Fn(i32, i32) -> bool;

/// Helper to apply rotation transform to SVG element.
pub fn rotate_transform(x: f64, y: f64, size: f64, rotation: f64) -> Option<String> {
    if rotation.abs() < 0.0001 {
        return None;
    }
    let cx = x + size / 2.0;
    let cy = y + size / 2.0;
    let degrees = (180.0 * rotation) / PI;
    Some(format!("rotate({},{},{})", degrees, cx, cy))
}

/// Helper to create SVG circle element.
pub fn svg_circle(cx: f64, cy: f64, r: f64, transform: Option<&str>) -> String {
    match transform {
        Some(t) => format!(
            r#"<circle cx="{}" cy="{}" r="{}" transform="{}"/>"#,
            cx, cy, r, t
        ),
        None => format!(r#"<circle cx="{}" cy="{}" r="{}"/>"#, cx, cy, r),
    }
}

/// Helper to create SVG rect element.
pub fn svg_rect(x: f64, y: f64, width: f64, height: f64, transform: Option<&str>) -> String {
    match transform {
        Some(t) => format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" transform="{}"/>"#,
            x, y, width, height, t
        ),
        None => format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}"/>"#,
            x, y, width, height
        ),
    }
}

/// Helper to create SVG path element.
pub fn svg_path(d: &str, clip_rule: Option<&str>, transform: Option<&str>) -> String {
    let mut attrs = format!(r#"d="{}""#, d);
    if let Some(rule) = clip_rule {
        attrs.push_str(&format!(r#" clip-rule="{}""#, rule));
    }
    if let Some(t) = transform {
        attrs.push_str(&format!(r#" transform="{}""#, t));
    }
    format!(r#"<path {}/>"#, attrs)
}
