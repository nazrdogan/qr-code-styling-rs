//! Generate sample images for README documentation.

use qr_code_styling::config::{
    BackgroundOptions, Color, CornersDotOptions, CornersSquareOptions, DotsOptions,
    Gradient, ImageOptions,
};
use qr_code_styling::plugins::border::{BorderPlugin, Position, QRBorderOptions};
use qr_code_styling::types::{
    CornerDotType, CornerSquareType, DotType, OutputFormat, ShapeType,
};
use qr_code_styling::QRCodeStyling;
use std::path::PathBuf;

fn main() -> qr_code_styling::error::Result<()> {
    let assets = "assets";
    std::fs::create_dir_all(assets)?;

    // 1. Basic - simple default QR
    println!("Generating basic sample...");
    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .build()?;
    qr.save(&format!("{}/basic.png", assets), OutputFormat::Png)?;

    // 2. Rounded dots
    println!("Generating rounded dots sample...");
    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .dots_options(
            DotsOptions::new(DotType::Rounded).with_color(Color::from_hex("#4A90D9").unwrap()),
        )
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::ExtraRounded)
                .with_color(Color::from_hex("#4A90D9").unwrap()),
        )
        .corners_dot_options(
            CornersDotOptions::new(CornerDotType::Dot)
                .with_color(Color::from_hex("#4A90D9").unwrap()),
        )
        .background_options(BackgroundOptions::default().with_color(Color::from_hex("#FFFFFF").unwrap()))
        .build()?;
    qr.save(&format!("{}/rounded.png", assets), OutputFormat::Png)?;

    // 3. Dots style
    println!("Generating dots style sample...");
    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .dots_options(
            DotsOptions::new(DotType::Dots).with_color(Color::from_hex("#E74C3C").unwrap()),
        )
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::Dot)
                .with_color(Color::from_hex("#C0392B").unwrap()),
        )
        .corners_dot_options(
            CornersDotOptions::new(CornerDotType::Dot)
                .with_color(Color::from_hex("#C0392B").unwrap()),
        )
        .background_options(BackgroundOptions::default().with_color(Color::from_hex("#FFFFFF").unwrap()))
        .build()?;
    qr.save(&format!("{}/dots.png", assets), OutputFormat::Png)?;

    // 4. Classy rounded
    println!("Generating classy rounded sample...");
    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .dots_options(
            DotsOptions::new(DotType::ClassyRounded)
                .with_color(Color::from_hex("#2C3E50").unwrap()),
        )
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::ExtraRounded)
                .with_color(Color::from_hex("#2C3E50").unwrap()),
        )
        .corners_dot_options(
            CornersDotOptions::new(CornerDotType::Dot)
                .with_color(Color::from_hex("#2C3E50").unwrap()),
        )
        .background_options(BackgroundOptions::default().with_color(Color::from_hex("#FFFFFF").unwrap()))
        .build()?;
    qr.save(
        &format!("{}/classy_rounded.png", assets),
        OutputFormat::Png,
    )?;

    // 5. With gradient
    println!("Generating gradient sample...");
    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .dots_options(DotsOptions::new(DotType::Rounded).with_gradient(
            Gradient::simple_linear(
                Color::from_hex("#8E2DE2").unwrap(),
                Color::from_hex("#4A00E0").unwrap(),
            ),
        ))
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::ExtraRounded)
                .with_color(Color::from_hex("#8E2DE2").unwrap()),
        )
        .corners_dot_options(
            CornersDotOptions::new(CornerDotType::Dot)
                .with_color(Color::from_hex("#4A00E0").unwrap()),
        )
        .background_options(BackgroundOptions::default().with_color(Color::from_hex("#FFFFFF").unwrap()))
        .build()?;
    qr.save(&format!("{}/gradient.png", assets), OutputFormat::Png)?;

    // 6. With logo
    println!("Generating logo sample...");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let logo_path = PathBuf::from(manifest_dir).join("examples/logo.png");
    let logo_bytes = std::fs::read(&logo_path)?;

    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .image(logo_bytes.clone())
        .image_options(
            ImageOptions::default()
                .with_image_size(0.4)
                .with_margin(5)
                .with_hide_background_dots(true),
        )
        .dots_options(
            DotsOptions::new(DotType::Rounded).with_color(Color::from_hex("#1877F2").unwrap()),
        )
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::ExtraRounded)
                .with_color(Color::from_hex("#1877F2").unwrap()),
        )
        .corners_dot_options(
            CornersDotOptions::new(CornerDotType::Dot)
                .with_color(Color::from_hex("#1877F2").unwrap()),
        )
        .background_options(BackgroundOptions::default().with_color(Color::from_hex("#FFFFFF").unwrap()))
        .build()?;
    qr.save(&format!("{}/with_logo.png", assets), OutputFormat::Png)?;

    // 7. Circle shape
    println!("Generating circle shape sample...");
    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .shape(ShapeType::Circle)
        .dots_options(
            DotsOptions::new(DotType::Rounded).with_color(Color::from_hex("#27AE60").unwrap()),
        )
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::ExtraRounded)
                .with_color(Color::from_hex("#27AE60").unwrap()),
        )
        .corners_dot_options(
            CornersDotOptions::new(CornerDotType::Dot)
                .with_color(Color::from_hex("#27AE60").unwrap()),
        )
        .background_options(BackgroundOptions::default().with_color(Color::from_hex("#FFFFFF").unwrap()))
        .build()?;
    qr.save(&format!("{}/circle.png", assets), OutputFormat::Png)?;

    // 8. With border
    println!("Generating border sample...");
    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .margin(50)
        .shape(ShapeType::Circle)
        .image(logo_bytes)
        .image_options(
            ImageOptions::default()
                .with_image_size(0.3)
                .with_margin(5)
                .with_hide_background_dots(true),
        )
        .dots_options(
            DotsOptions::new(DotType::Rounded).with_color(Color::from_hex("#E74C3C").unwrap()),
        )
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::ExtraRounded)
                .with_color(Color::from_hex("#E74C3C").unwrap()),
        )
        .corners_dot_options(
            CornersDotOptions::new(CornerDotType::Dot)
                .with_color(Color::from_hex("#E74C3C").unwrap()),
        )
        .build()?;

    let svg = qr.render_svg()?;

    let text_style =
        "font-size: 18px; font-family: Arial, sans-serif; fill: #FFFFFF; font-weight: bold;";

    let border_options = QRBorderOptions::new(35.0, "#E74C3C")
        .with_round(1.0)
        .with_styled_text(Position::Top, "SCAN ME", text_style)
        .with_styled_text(Position::Bottom, "qr-code-styling", text_style);

    let bordered_svg = BorderPlugin::new(border_options).apply(&svg, 300, 300);

    // Convert bordered SVG to PNG via saving as SVG then rendering
    std::fs::write(&format!("{}/with_border.svg", assets), &bordered_svg)?;

    println!("\nAll samples generated in assets/!");
    Ok(())
}
