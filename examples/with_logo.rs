//! Example demonstrating logo embedding in QR codes.

use qr_code_styling::config::{Color, DotsOptions, ImageOptions};
use qr_code_styling::types::{DotType, OutputFormat};
use qr_code_styling::QRCodeStyling;
use std::path::PathBuf;

fn main() -> qr_code_styling::error::Result<()> {
    // Get the path to logo.png in the examples folder
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let logo_path = PathBuf::from(manifest_dir).join("examples/logo.png");

    println!("Loading logo from: {}", logo_path.display());
    let logo_bytes = std::fs::read(&logo_path)?;
    println!("Logo size: {} bytes", logo_bytes.len());

    std::fs::create_dir_all("output")?;

    println!("\nGenerating QR code with centered logo...");

    let qr = QRCodeStyling::builder()
        .data("https://rust-lang.org")
        .size(400)
        .image(logo_bytes)
        .image_options(
            ImageOptions::default()
                .with_image_size(0.4) // Logo takes 40% of available space
                .with_margin(5) // 5px margin around logo
                .with_hide_background_dots(true), // Hide QR dots behind logo
        )
        .dots_options(DotsOptions::new(DotType::Rounded).with_color(Color::from_hex("#2C3E50").unwrap()))
        .build()?;

    // Save as SVG
    qr.save("output/with_logo.svg", OutputFormat::Svg)?;
    println!("Saved: output/with_logo.svg");

    // Save as PNG
    qr.save("output/with_logo.png", OutputFormat::Png)?;
    println!("Saved: output/with_logo.png");

    println!("\nDone!");

    Ok(())
}
