//! Example demonstrating the border plugin for QR codes with circle shape and logo.

use qr_code_styling::config::{Color, DotsOptions, ImageOptions};
use qr_code_styling::plugins::border::{BorderPlugin, Position, QRBorderOptions};
use qr_code_styling::rendering::PdfRenderer;
use qr_code_styling::types::{DotType, ShapeType};
use qr_code_styling::QRCodeStyling;
use std::path::PathBuf;

fn main() -> qr_code_styling::error::Result<()> {
    // Load logo
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let logo_path = PathBuf::from(manifest_dir).join("examples/logo.png");
    let logo_bytes = std::fs::read(&logo_path)?;
    println!("Loaded logo: {} bytes", logo_bytes.len());

    std::fs::create_dir_all("output")?;

    // Circle QR code with text border and logo
    println!("Generating circle QR code with text border and logo...");

    let qr = QRCodeStyling::builder()
        .data("https://rust-lang.org")
        .size(400)
        .margin(60) // Bigger margin for bigger border
        .shape(ShapeType::Circle) // Circle shape
        .image(logo_bytes.clone())
        .image_options(
            ImageOptions::default()
                .with_image_size(0.3) // Logo takes 30% of available space
                .with_margin(5)
                .with_hide_background_dots(true),
        )
        .dots_options(
            DotsOptions::new(DotType::Rounded).with_color(Color::from_hex("#E74C3C").unwrap()),
        )
        .build()?;

    let svg = qr.render_svg()?;

    // Create circular border with text on all sides
    let text_style = "font-size: 20px; font-family: Arial, sans-serif; fill: #FFFFFF; font-weight: bold;";

    let border_options = QRBorderOptions::new(40.0, "#E74C3C")
        .with_round(1.0) // Fully round = circle
        .with_styled_text(Position::Top, "SCAN ME", text_style)
        .with_styled_text(Position::Bottom, "rust-lang.org", text_style);

    let border_plugin = BorderPlugin::new(border_options);
    let svg_with_border = border_plugin.apply(&svg, 400, 400);

    std::fs::write("output/circle_border.svg", &svg_with_border)?;
    println!("Saved: output/circle_border.svg");

    // Also save as PDF
    let pdf_data = PdfRenderer::render_from_svg(&svg_with_border, 400, 400)?;
    std::fs::write("output/circle_border.pdf", &pdf_data)?;
    println!("Saved: output/circle_border.pdf");

    // Circle with text on all 4 sides
    println!("\nGenerating circle QR with text on all sides...");

    let qr = QRCodeStyling::builder()
        .data("https://example.com")
        .size(450)
        .margin(60)
        .shape(ShapeType::Circle)
        .dots_options(
            DotsOptions::new(DotType::Dots).with_color(Color::from_hex("#3498DB").unwrap()),
        )
        .build()?;

    let svg = qr.render_svg()?;

    let text_style = "font-size: 12px; font-family: 'Arial', sans-serif; fill: #3498DB; font-weight: bold;";

    let border_options = QRBorderOptions::new(30.0, "#3498DB")
        .with_round(1.0)
        .with_styled_text(Position::Top, "SCAN TO VISIT", text_style)
        .with_styled_text(Position::Bottom, "example.com", text_style)
        .with_styled_text(Position::Left, "FAST", text_style)
        .with_styled_text(Position::Right, "EASY", text_style);

    let border_plugin = BorderPlugin::new(border_options);
    let svg_with_border = border_plugin.apply(&svg, 450, 450);

    std::fs::write("output/circle_all_text.svg", &svg_with_border)?;
    println!("Saved: output/circle_all_text.svg");

    println!("\nDone!");
    Ok(())
}
