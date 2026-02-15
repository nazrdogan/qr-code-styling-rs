//! Basic example demonstrating QR code styling.

use qr_code_styling::config::{
    BackgroundOptions, Color, CornersDotOptions, CornersSquareOptions, DotsOptions,
};
use qr_code_styling::types::{CornerDotType, CornerSquareType, DotType, OutputFormat};
use qr_code_styling::QRCodeStyling;

fn main() -> qr_code_styling::error::Result<()> {
    std::fs::create_dir_all("output")?;

    // Example 1: Basic QR code with default settings
    println!("Generating basic QR code...");
    let qr = QRCodeStyling::builder()
        .data("https://example.com")
        .width(300)
        .shape(qr_code_styling::ShapeType::Circle)
        .height(300)
        .build()?;

    let svg = qr.render_svg()?;
    println!("Generated SVG with {} bytes", svg.len());
    std::fs::write("output/basic.svg", &svg)?;
    println!("Saved: output/basic.svg");

    // Example 2: Styled QR code with dots
    println!("\nGenerating styled QR code with dots...");
    let qr_dots = QRCodeStyling::builder()
        .data("https://rust-lang.org")
        .width(400)
        .height(400)
        .margin(20)
        .dots_options(
            DotsOptions::new(DotType::Dots).with_color(Color::from_hex("#4A90D9").unwrap()),
        )
        .background_options(
            BackgroundOptions::default().with_color(Color::from_hex("#FFFFFF").unwrap()),
        )
        .build()?;

    qr_dots.save("output/dots.svg", OutputFormat::Svg)?;
    println!("Saved: output/dots.svg");

    // Example 3: Rounded style with custom corners
    println!("\nGenerating rounded QR code...");
    let qr_rounded = QRCodeStyling::builder()
        .data("Hello, Rust QR Code Styling!")
        .width(350)
        .height(350)
        .dots_options(
            DotsOptions::new(DotType::Rounded).with_color(Color::from_hex("#2C3E50").unwrap()),
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

    qr_rounded.save("output/rounded.svg", OutputFormat::Svg)?;
    qr_rounded.save("output/rounded.png", OutputFormat::Png)?;
    println!("Saved: output/rounded.svg, output/rounded.png");

    // Example 4: Classy rounded style
    println!("\nGenerating classy rounded QR code...");
    let qr_classy = QRCodeStyling::builder()
        .data("Classy QR Code")
        .width(300)
        .height(300)
        .dots_options(DotsOptions::new(DotType::ClassyRounded))
        .build()?;

    qr_classy.save("output/classy.svg", OutputFormat::Svg)?;
    println!("Saved: output/classy.svg");

    // Example 5: Extra rounded dots
    println!("\nGenerating extra rounded QR code...");
    let qr_extra = QRCodeStyling::builder()
        .data("Extra Rounded!")
        .size(300) // size() sets both width and height
        .dots_options(DotsOptions::new(DotType::ExtraRounded))
        .build()?;

    qr_extra.save("output/extra_rounded.svg", OutputFormat::Svg)?;
    println!("Saved: output/extra_rounded.svg");

    // Example 6: All output formats (including PDF)
    println!("\nGenerating all output formats...");
    let qr_all = QRCodeStyling::builder()
        .data("All formats test")
        .size(200)
        .build()?;

    qr_all.save("output/formats.svg", OutputFormat::Svg)?;
    qr_all.save("output/formats.png", OutputFormat::Png)?;
    qr_all.save("output/formats.jpg", OutputFormat::Jpeg)?;
    qr_all.save("output/formats.webp", OutputFormat::WebP)?;
    qr_all.save("output/formats.pdf", OutputFormat::Pdf)?;
    println!("Saved: output/formats.svg, .png, .jpg, .webp, .pdf");

    println!("\nAll examples generated successfully!");
    Ok(())
}
