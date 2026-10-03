//! Print-ready CMYK PDF.
//!
//! Run with: cargo run --example cmyk_pdf --features cmyk

use qr_code_styling::{
    Cmyk, CmykPdfOptions, Color, CornerSquareType, CornersSquareOptions, DotType, DotsOptions,
    QRCodeStyling,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let brand_blue = Color::rgb(0, 59, 209);

    let qr = QRCodeStyling::builder()
        .data("https://github.com/nazrdogan/qr-code-styling-rs")
        .size(300)
        .margin(12)
        .dots_options(DotsOptions::new(DotType::Rounded).with_color(Color::BLACK))
        .corners_square_options(
            CornersSquareOptions::new(CornerSquareType::ExtraRounded).with_color(brand_blue),
        )
        .build()?;

    // Black prints as 100% K by default; give the brand color its exact
    // CMYK value instead of the automatic conversion.
    let options = CmykPdfOptions::new().with_color(brand_blue, Cmyk::new(100.0, 72.0, 0.0, 18.0));

    std::fs::create_dir_all("output")?;
    qr.save_pdf_cmyk("output/qr_cmyk.pdf", &options)?;
    println!("Saved output/qr_cmyk.pdf");
    Ok(())
}
