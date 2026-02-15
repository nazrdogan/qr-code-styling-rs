//! QR Border Plugin
//!
//! A plugin to add customizable borders and decorations to QR codes.
//! Supports outer, main, and inner borders, as well as text or image
//! decorations along each side.

use std::collections::HashMap;

/// Position for specifying where decorations should be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Position {
    Top,
    Bottom,
    Left,
    Right,
}

/// Type of decoration that can be applied to the QR code borders.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DecorationType {
    /// Text decoration with the text content
    Text(String),
    /// Image decoration with image data (base64 or URL)
    Image(String),
}

/// Style options for border elements.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorderOptions {
    /// Border thickness in pixels
    pub thickness: f64,
    /// Border color (hex string like "#000000")
    pub color: String,
    /// Optional dash array for dashed borders (e.g., "5,5")
    pub dasharray: Option<String>,
}

impl Default for BorderOptions {
    fn default() -> Self {
        Self {
            thickness: 10.0,
            color: "#000000".to_string(),
            dasharray: None,
        }
    }
}

impl BorderOptions {
    /// Create new border options.
    pub fn new(thickness: f64, color: impl Into<String>) -> Self {
        Self {
            thickness,
            color: color.into(),
            dasharray: None,
        }
    }

    /// Set dash array for dashed border.
    pub fn with_dasharray(mut self, dasharray: impl Into<String>) -> Self {
        self.dasharray = Some(dasharray.into());
        self
    }
}

/// Decoration configuration for text or image decorations.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BorderDecoration {
    /// Type of decoration (text or image)
    pub decoration_type: DecorationType,
    /// Optional CSS style string
    pub style: Option<String>,
}

impl BorderDecoration {
    /// Create a text decoration.
    pub fn text(value: impl Into<String>) -> Self {
        Self {
            decoration_type: DecorationType::Text(value.into()),
            style: None,
        }
    }

    /// Create an image decoration.
    pub fn image(value: impl Into<String>) -> Self {
        Self {
            decoration_type: DecorationType::Image(value.into()),
            style: None,
        }
    }

    /// Set CSS style.
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }
}

/// Extension options for the QR Border Plugin.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QRBorderOptions {
    /// Main border options
    pub border: BorderOptions,
    /// Corner roundness (0.0 = square, 1.0 = fully round)
    pub round: f64,
    /// Optional inner border
    pub border_inner: Option<BorderOptions>,
    /// Optional outer border
    pub border_outer: Option<BorderOptions>,
    /// Decorations mapped by position
    pub decorations: HashMap<Position, BorderDecoration>,
}

impl Default for QRBorderOptions {
    fn default() -> Self {
        Self {
            border: BorderOptions::default(),
            round: 0.0,
            border_inner: None,
            border_outer: None,
            decorations: HashMap::new(),
        }
    }
}

impl QRBorderOptions {
    /// Create new border options with specified thickness and color.
    pub fn new(thickness: f64, color: impl Into<String>) -> Self {
        Self {
            border: BorderOptions::new(thickness, color),
            ..Default::default()
        }
    }

    /// Set corner roundness.
    pub fn with_round(mut self, round: f64) -> Self {
        self.round = round.clamp(0.0, 1.0);
        self
    }

    /// Set inner border.
    pub fn with_inner_border(mut self, options: BorderOptions) -> Self {
        self.border_inner = Some(options);
        self
    }

    /// Set outer border.
    pub fn with_outer_border(mut self, options: BorderOptions) -> Self {
        self.border_outer = Some(options);
        self
    }

    /// Add a decoration at the specified position.
    pub fn with_decoration(mut self, position: Position, decoration: BorderDecoration) -> Self {
        self.decorations.insert(position, decoration);
        self
    }

    /// Add text decoration at the specified position.
    pub fn with_text(mut self, position: Position, text: impl Into<String>) -> Self {
        self.decorations
            .insert(position, BorderDecoration::text(text));
        self
    }

    /// Add text decoration with style at the specified position.
    pub fn with_styled_text(
        mut self,
        position: Position,
        text: impl Into<String>,
        style: impl Into<String>,
    ) -> Self {
        self.decorations
            .insert(position, BorderDecoration::text(text).with_style(style));
        self
    }
}

/// Border plugin for adding borders and decorations to QR codes.
pub struct BorderPlugin {
    options: QRBorderOptions,
}

impl BorderPlugin {
    /// Create a new border plugin with the given options.
    pub fn new(options: QRBorderOptions) -> Self {
        Self { options }
    }

    /// Apply the border to an SVG string.
    /// Returns the modified SVG with borders and decorations.
    pub fn apply(&self, svg: &str, width: u32, height: u32) -> String {
        let width = width as f64;
        let height = height as f64;

        let mut defs_content = String::new();
        let mut elements_content = String::new();

        // Create main border
        let main_attrs = self.generate_rect_attributes(width, height, &self.options.border);
        elements_content.push_str(&self.create_rect(&main_attrs));

        // Create inner border if specified
        if let Some(ref inner) = self.options.border_inner {
            let mut inner_attrs = self.generate_rect_attributes(width, height, inner);

            // Adjust inner border position and size
            inner_attrs.x =
                inner_attrs.x - inner.thickness + self.options.border.thickness;
            inner_attrs.y =
                inner_attrs.y - inner.thickness + self.options.border.thickness;
            inner_attrs.width =
                inner_attrs.width + 2.0 * (inner.thickness - self.options.border.thickness);
            inner_attrs.height =
                inner_attrs.height + 2.0 * (inner.thickness - self.options.border.thickness);
            inner_attrs.rx = (inner_attrs.rx + inner.thickness - self.options.border.thickness)
                .max(0.0);

            elements_content.push_str(&self.create_rect(&inner_attrs));
        }

        // Create outer border if specified
        if let Some(ref outer) = self.options.border_outer {
            let outer_attrs = self.generate_rect_attributes(width, height, outer);
            elements_content.push_str(&self.create_rect(&outer_attrs));
        }

        // Add decorations
        for (position, decoration) in &self.options.decorations {
            match &decoration.decoration_type {
                DecorationType::Text(text) => {
                    let (path_def, text_elem) = self.create_text_decoration(
                        *position,
                        text,
                        decoration.style.as_deref(),
                        width,
                        height,
                    );
                    defs_content.push_str(&path_def);
                    elements_content.push_str(&text_elem);
                }
                DecorationType::Image(src) => {
                    let image_elem = self.create_image_decoration(
                        *position,
                        src,
                        decoration.style.as_deref(),
                        width,
                        height,
                    );
                    elements_content.push_str(&image_elem);
                }
            }
        }

        // Inject into existing SVG
        self.inject_into_svg(svg, &defs_content, &elements_content)
    }

    fn generate_rect_attributes(&self, width: f64, height: f64, options: &BorderOptions) -> RectAttributes {
        let size = width.min(height);
        let rx = ((size / 2.0) * self.options.round - options.thickness / 2.0).max(0.0);

        RectAttributes {
            fill: "none".to_string(),
            x: (width - size + options.thickness) / 2.0,
            y: (height - size + options.thickness) / 2.0,
            width: size - options.thickness,
            height: size - options.thickness,
            stroke: options.color.clone(),
            stroke_width: options.thickness,
            stroke_dasharray: options.dasharray.clone().unwrap_or_default(),
            rx,
        }
    }

    fn create_rect(&self, attrs: &RectAttributes) -> String {
        let dasharray_attr = if attrs.stroke_dasharray.is_empty() {
            String::new()
        } else {
            format!(r#" stroke-dasharray="{}""#, attrs.stroke_dasharray)
        };

        format!(
            r#"<rect fill="{}" x="{}" y="{}" width="{}" height="{}" stroke="{}" stroke-width="{}"{} rx="{}"/>
"#,
            attrs.fill,
            attrs.x,
            attrs.y,
            attrs.width,
            attrs.height,
            attrs.stroke,
            attrs.stroke_width,
            dasharray_attr,
            attrs.rx
        )
    }

    fn create_text_decoration(
        &self,
        position: Position,
        text: &str,
        style: Option<&str>,
        width: f64,
        height: f64,
    ) -> (String, String) {
        let thickness = self.options.border.thickness;
        let round = self.options.round;
        let size = width.min(height);

        // Center of the QR code
        let cx = width / 2.0;
        let cy = height / 2.0;

        // Text radius - on the center of the border stroke
        let text_radius = (size - thickness) / 2.0;

        let path_id = format!("{:?}-text-path", position).to_lowercase();

        // Create style with default values
        let base_style = style.unwrap_or("font-size: 14px; font-family: Arial, sans-serif;");

        // For circular borders, use curved text paths
        if round >= 0.5 {
            // Create arc path for text to follow
            let path_d = match position {
                Position::Top => {
                    // Arc from left to right along the top (text reads left-to-right)
                    format!(
                        "M {},{} A {},{} 0 0 1 {},{}",
                        cx - text_radius, cy,
                        text_radius, text_radius,
                        cx + text_radius, cy
                    )
                }
                Position::Bottom => {
                    // Arc from left to right along the bottom (sweep flag 0 for bottom arc)
                    format!(
                        "M {},{} A {},{} 0 0 0 {},{}",
                        cx - text_radius, cy,
                        text_radius, text_radius,
                        cx + text_radius, cy
                    )
                }
                Position::Left => {
                    // Arc from top to bottom along the left (sweep flag 0)
                    format!(
                        "M {},{} A {},{} 0 0 0 {},{}",
                        cx, cy - text_radius,
                        text_radius, text_radius,
                        cx, cy + text_radius
                    )
                }
                Position::Right => {
                    // Arc from top to bottom along the right
                    format!(
                        "M {},{} A {},{} 0 0 1 {},{}",
                        cx, cy - text_radius,
                        text_radius, text_radius,
                        cx, cy + text_radius
                    )
                }
            };

            let path_def = format!(
                "<path id=\"{}\" d=\"{}\" fill=\"none\"/>\n",
                path_id, path_d
            );

            let text_elem = format!(
                "<text style=\"{}\">\n  <textPath xlink:href=\"#{}\" href=\"#{}\" startOffset=\"50%\" text-anchor=\"middle\" dominant-baseline=\"central\">{}</textPath>\n</text>\n",
                base_style, path_id, path_id, text
            );

            (path_def, text_elem)
        } else {
            // For rectangular borders, use straight text
            let border_offset = thickness / 2.0;
            let half_size = (size - thickness) / 2.0;

            let (x, y, rotation) = match position {
                Position::Top => (cx, cy - half_size - border_offset, 0.0),
                Position::Bottom => (cx, cy + half_size + border_offset, 0.0),
                Position::Left => (cx - half_size - border_offset, cy, -90.0),
                Position::Right => (cx + half_size + border_offset, cy, 90.0),
            };

            let transform = if rotation != 0.0 {
                format!(r#" transform="rotate({},{},{})""#, rotation, x, y)
            } else {
                String::new()
            };

            let text_elem = format!(
                r#"<text x="{}" y="{}" text-anchor="middle" dominant-baseline="middle" style="{}"{}>{}</text>
"#,
                x, y, base_style, transform, text
            );

            (String::new(), text_elem)
        }
    }

    fn create_image_decoration(
        &self,
        position: Position,
        src: &str,
        style: Option<&str>,
        width: f64,
        height: f64,
    ) -> String {
        let thickness = self.options.border.thickness;
        let size = width.min(height);

        let mut x = (width - size + thickness) / 2.0;
        let mut y = (height - size + thickness) / 2.0;

        match position {
            Position::Top => {
                x += (size - thickness) / 2.0;
            }
            Position::Right => {
                x += size - thickness;
                y += (size - thickness) / 2.0;
            }
            Position::Bottom => {
                x += (size - thickness) / 2.0;
                y += size - thickness;
            }
            Position::Left => {
                y += (size - thickness) / 2.0;
            }
        }

        let style_attr = style
            .map(|s| format!(r#" style="{}""#, s))
            .unwrap_or_default();

        format!(
            r#"<image href="{}" xlink:href="{}" x="{}" y="{}"{}/>"#,
            src, src, x, y, style_attr
        )
    }

    fn inject_into_svg(&self, svg: &str, defs_content: &str, elements_content: &str) -> String {
        // Find the closing </svg> tag and insert before it
        if let Some(close_pos) = svg.rfind("</svg>") {
            let mut result = String::with_capacity(svg.len() + defs_content.len() + elements_content.len() + 100);
            result.push_str(&svg[..close_pos]);

            // Add to defs if we have path definitions
            if !defs_content.is_empty() {
                // Check if there's already a <defs> section
                if let Some(defs_close) = svg.find("</defs>") {
                    // Insert before </defs>
                    let before_defs_close = &svg[..defs_close];
                    let after_defs_close = &svg[defs_close..close_pos];
                    result.clear();
                    result.push_str(before_defs_close);
                    result.push_str(defs_content);
                    result.push_str(after_defs_close);
                } else {
                    // No defs section, add one
                    result.push_str("<defs>\n");
                    result.push_str(defs_content);
                    result.push_str("</defs>\n");
                }
            }

            result.push_str(elements_content);
            result.push_str("</svg>");
            result
        } else {
            // Fallback: return original SVG with border content appended
            format!("{}\n{}", svg.trim_end(), elements_content)
        }
    }
}

/// Internal struct for rectangle attributes.
struct RectAttributes {
    fill: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    stroke: String,
    stroke_width: f64,
    stroke_dasharray: String,
    rx: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_border_options_default() {
        let opts = BorderOptions::default();
        assert_eq!(opts.thickness, 10.0);
        assert_eq!(opts.color, "#000000");
        assert!(opts.dasharray.is_none());
    }

    #[test]
    fn test_qr_border_options_builder() {
        let opts = QRBorderOptions::new(15.0, "#FF0000")
            .with_round(0.5)
            .with_text(Position::Top, "SCAN ME")
            .with_inner_border(BorderOptions::new(5.0, "#00FF00"));

        assert_eq!(opts.border.thickness, 15.0);
        assert_eq!(opts.border.color, "#FF0000");
        assert_eq!(opts.round, 0.5);
        assert!(opts.border_inner.is_some());
        assert!(opts.decorations.contains_key(&Position::Top));
    }

    #[test]
    fn test_border_plugin_apply() {
        let svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="300" height="300">
<defs></defs>
<rect x="0" y="0" width="300" height="300" fill="white"/>
</svg>"#;

        let options = QRBorderOptions::new(10.0, "#000000").with_round(0.2);
        let plugin = BorderPlugin::new(options);
        let result = plugin.apply(svg, 300, 300);

        assert!(result.contains("stroke=\"#000000\""));
        assert!(result.contains("stroke-width=\"10\""));
    }

    #[test]
    fn test_border_with_text_decoration() {
        let svg = r#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" width="300" height="300">
<defs></defs>
</svg>"#;

        // Use round >= 0.5 to trigger textPath-based text decoration
        let options = QRBorderOptions::new(20.0, "#333333")
            .with_round(0.5)
            .with_styled_text(Position::Top, "SCAN ME", "font-size: 14px; fill: #333;");

        let plugin = BorderPlugin::new(options);
        let result = plugin.apply(svg, 300, 300);

        assert!(result.contains("SCAN ME"));
        assert!(result.contains("textPath"));
        assert!(result.contains("top-text-path"));
    }
}
