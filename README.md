# qr-code-styling

A Rust library for generating styled QR codes with customizable dots, corners, gradients, and logo embedding.

## Features

- **6 dot styles** — Square, Dots, Rounded, Classy, ClassyRounded, ExtraRounded
- **3 corner square styles** — Square, Dot, ExtraRounded
- **2 corner dot styles** — Dot, Square
- **Gradient support** — Linear and radial gradients for dots, corners, and background
- **Logo embedding** — Center an image inside the QR code with automatic dot hiding
- **Circle shape** — Render QR codes in a circular frame
- **Border plugin** — Add decorative borders with text labels
- **Multiple output formats** — SVG, PNG, JPEG, WebP, PDF
- **Optional serde support** — Serialize/deserialize configuration with the `serde` feature

## Installation

```toml
[dependencies]
qr-code-styling = "0.1"
```

## Quick Start

```rust
use qr_code_styling::{QRCodeStyling, OutputFormat};
use qr_code_styling::config::{DotsOptions, Color};
use qr_code_styling::types::DotType;

let qr = QRCodeStyling::builder()
    .data("https://example.com")
    .size(300)
    .dots_options(DotsOptions::new(DotType::Rounded).with_color(Color::rgb(0, 0, 128)))
    .build()
    .unwrap();

// Render as SVG string
let svg = qr.render_svg().unwrap();

// Save to file
qr.save("qr.png", OutputFormat::Png).unwrap();
```

## Examples

### Styled dots with custom corners

```rust
use qr_code_styling::{QRCodeStyling, OutputFormat};
use qr_code_styling::config::{DotsOptions, CornersSquareOptions, CornersDotOptions, Color};
use qr_code_styling::types::{DotType, CornerSquareType, CornerDotType};

let qr = QRCodeStyling::builder()
    .data("Hello, Rust!")
    .size(350)
    .dots_options(
        DotsOptions::new(DotType::Rounded)
            .with_color(Color::from_hex("#2C3E50").unwrap()),
    )
    .corners_square_options(
        CornersSquareOptions::new(CornerSquareType::ExtraRounded)
            .with_color(Color::from_hex("#E74C3C").unwrap()),
    )
    .corners_dot_options(
        CornersDotOptions::new(CornerDotType::Dot)
            .with_color(Color::from_hex("#E74C3C").unwrap()),
    )
    .build()
    .unwrap();

qr.save("styled.svg", OutputFormat::Svg).unwrap();
```

### With logo

```rust
use qr_code_styling::{QRCodeStyling, OutputFormat};
use qr_code_styling::config::{DotsOptions, ImageOptions, Color};
use qr_code_styling::types::DotType;

let logo_bytes = std::fs::read("logo.png").unwrap();

let qr = QRCodeStyling::builder()
    .data("https://rust-lang.org")
    .size(400)
    .image(logo_bytes)
    .image_options(
        ImageOptions::default()
            .with_image_size(0.4)
            .with_margin(5)
            .with_hide_background_dots(true),
    )
    .dots_options(
        DotsOptions::new(DotType::Rounded)
            .with_color(Color::from_hex("#2C3E50").unwrap()),
    )
    .build()
    .unwrap();

qr.save("with_logo.png", OutputFormat::Png).unwrap();
```

### Circle shape with border

```rust
use qr_code_styling::{QRCodeStyling, ShapeType, OutputFormat};
use qr_code_styling::config::{DotsOptions, Color};
use qr_code_styling::plugins::border::{BorderPlugin, Position, QRBorderOptions};
use qr_code_styling::types::DotType;

let qr = QRCodeStyling::builder()
    .data("https://example.com")
    .size(400)
    .margin(60)
    .shape(ShapeType::Circle)
    .dots_options(
        DotsOptions::new(DotType::Rounded)
            .with_color(Color::from_hex("#E74C3C").unwrap()),
    )
    .build()
    .unwrap();

let svg = qr.render_svg().unwrap();

let border_options = QRBorderOptions::new(40.0, "#E74C3C")
    .with_round(1.0)
    .with_styled_text(
        Position::Top,
        "SCAN ME",
        "font-size: 20px; font-family: Arial; fill: #FFFFFF; font-weight: bold;",
    )
    .with_styled_text(
        Position::Bottom,
        "example.com",
        "font-size: 20px; font-family: Arial; fill: #FFFFFF; font-weight: bold;",
    );

let bordered_svg = BorderPlugin::new(border_options).apply(&svg, 400, 400);
std::fs::write("bordered.svg", &bordered_svg).unwrap();
```

## Output Formats

| Format | Method | Feature |
|--------|--------|---------|
| SVG | `render_svg()` / `save(_, Svg)` | always available |
| PNG | `save(_, Png)` | `png` (default) |
| JPEG | `save(_, Jpeg)` | `jpeg` (default) |
| WebP | `save(_, WebP)` | `webp` (default) |
| PDF | `save(_, Pdf)` | always available |

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `png` | yes | PNG raster output |
| `jpeg` | yes | JPEG raster output |
| `webp` | yes | WebP raster output |
| `serde` | no | Serialize/deserialize support |

## License

MIT
