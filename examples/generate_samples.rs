//! Generate sample images for README documentation.

use qr_code_styling::config::{
    BackgroundOptions, Color, CornersDotOptions, CornersSquareOptions, DotsOptions, Gradient,
    ImageOptions,
};
use qr_code_styling::plugins::border::{BorderPlugin, Position, QRBorderOptions};
use qr_code_styling::types::{CornerDotType, CornerSquareType, DotType, OutputFormat, ShapeType};
use qr_code_styling::{ColorStop, QRCodeStyling, QRCodeStylingBuilder};
use std::f64::consts::PI;
use std::path::{Path, PathBuf};

/// Use the Repository's URL as the QR's data.
const SAMPLE_DATA: &str = "https://github.com/nazrdogan/qr-code-styling-rs";

fn main() -> qr_code_styling::error::Result<()> {
    let assets = "assets";
    std::fs::create_dir_all(assets)?;

    // 1. Basic - simple default QR
    println!("Generating basic sample...");
    let qr = QRCodeStyling::builder()
        .data(SAMPLE_DATA)
        .size(300)
        .build()?;
    qr.save(&format!("{}/basic.png", assets), OutputFormat::Png)?;

    // 2. Rounded dots
    println!("Generating rounded dots sample...");
    let qr = QRCodeStyling::builder()
        .data(SAMPLE_DATA)
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
        .data(SAMPLE_DATA)
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
        .data(SAMPLE_DATA)
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
        .data(SAMPLE_DATA)
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
        .data(SAMPLE_DATA)
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
        .data(SAMPLE_DATA)
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
        .data(SAMPLE_DATA)
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

    // Initialization of Style parameters.
    const QR_SIZE: u32 = 300;
    let base_background =
        BackgroundOptions::default().with_color(Color::from_hex("#333333").unwrap());
    let base_builder = QRCodeStyling::builder()
        .data(SAMPLE_DATA)
        .size(QR_SIZE)
        .corners_dot_options(
            CornersDotOptions::default().with_color(Color::from_hex("#FFA544").expect("valid hex")),
        )
        .corners_square_options(
            CornersSquareOptions::default()
                .with_color(Color::from_hex("#FFA544").expect("valid hex")),
        )
        .dots_options(
            DotsOptions::default().with_color(Color::from_hex("#FFA544").expect("valid hex")),
        )
        .background_options(base_background.clone());

    let root: &Path = assets.as_ref();

    // 9. Gradient Samples
    let gradient_samples =
        samples_background_gradients(base_builder.clone(), base_background.clone())?;

    let gradients_path = root.join("background_gradients");
    save(gradient_samples, &gradients_path)?;

    let dots_root = root.join("dots_options");
    let base_dot_options =
        DotsOptions::default().with_color(Color::from_hex("#044389").expect("valid hex"));

    let color_treatments = vec![
        (
            "solid",
            ColorTreatment::solid(Color::from_hex("#AA1155").expect("valid hex")),
        ),
        (
            "gradient",
            ColorTreatment::Gradient(vec![
                ColorStop::new(0.0, Color::from_hex("#FFEE88").expect("valid hex")),
                ColorStop::new(0.5, Color::from_hex("#AA1155").expect("valid hex")),
                ColorStop::new(1.0, Color::from_hex("#00CC99").expect("valid hex")),
            ]),
        ),
    ];

    // 10. Dot Types Samples
    let dot_samples = samples_dot_types(base_builder.clone(), base_dot_options.clone())?;
    let dots_types_path = dots_root.join("dots_types");
    save(dot_samples, &dots_types_path)?;

    // 11. Dot Colors Samples
    let dot_colors_samples = sample_dots_colors(base_builder.clone(), base_dot_options.clone())?;
    let dots_colors_path = dots_root.join("dots_colors");
    save(dot_colors_samples, &dots_colors_path)?;

    // 12. Corner Dots Samples
    let corners_dots_samples =
        samples_corners_dots_comprehensive(base_builder.clone(), color_treatments.clone())?;
    let corners_dots_path = dots_root.join("corners_dots");
    save(corners_dots_samples, &corners_dots_path)?;

    // 13. Corner Squares Samples
    let corners_squares_samples =
        samples_corners_squares_comprehensive(base_builder.clone(), color_treatments.clone())?;
    let corners_squares_path = dots_root.join("corners_squares");
    save(corners_squares_samples, &corners_squares_path)?;

    // 14. Shape Samples
    let shape_samples = samples_shapes(base_builder.clone())?;
    let shapes_path = root.join("shapes");
    save(shape_samples, &shapes_path)?;

    println!("\nAll samples generated in assets/!");

    Ok(())
}

struct Sample {
    /// Name of the sample (used for filename)
    name: String,
    /// QR code configuration for this sample
    style: QRCodeStyling,
}

/// Represents different ways to color QR code elements.
#[derive(Clone)]
enum ColorTreatment {
    /// A single solid color
    Solid(Color),
    /// A gradient with multiple color stops
    Gradient(Vec<ColorStop>),
}

impl ColorTreatment {
    /// Create a solid color treatment
    fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    /// Apply this color treatment to corner dot options
    fn apply_to_corners_dot(&self, options: CornersDotOptions) -> CornersDotOptions {
        match self {
            ColorTreatment::Solid(color) => options.with_color(*color),
            ColorTreatment::Gradient(stops) => {
                options.with_gradient(Gradient::linear(stops.clone()))
            }
        }
    }

    /// Apply this color treatment to corner square options
    fn apply_to_corners_square(&self, options: CornersSquareOptions) -> CornersSquareOptions {
        match self {
            ColorTreatment::Solid(color) => options.with_color(*color),
            ColorTreatment::Gradient(stops) => {
                options.with_gradient(Gradient::linear(stops.clone()))
            }
        }
    }

    /// Apply this color treatment to dot options
    #[expect(unused)] // The `apply_to_dots` method is currently not used in the sample generation, but left for potential future use.
    fn apply_to_dots(&self, options: DotsOptions) -> DotsOptions {
        match self {
            ColorTreatment::Solid(color) => options.with_color(*color),
            ColorTreatment::Gradient(stops) => {
                options.with_gradient(Gradient::linear(stops.clone()))
            }
        }
    }
}

/// Receives a list of samples and saves them to the specified root path.
///
/// Each sample's name is used to generate the filename (e.g., "{root_path}/linear_gradient.png").
fn save(samples: Vec<Sample>, root_path: &Path) -> qr_code_styling::error::Result<()> {
    std::fs::create_dir_all(root_path)?;
    println!(
        "Saving ({}) samples to {}...",
        samples.len(),
        root_path.display()
    );
    for sample in samples {
        let file_path = root_path.join(format!("{}.png", sample.name));
        sample.style.save(&file_path, OutputFormat::Png)?;
    }
    Ok(())
}

/// Returns a list of sample QR code configurations with different background [gradients types](GradientType).
///
/// Receives a basic configurations to act as the base styling for generated samples.
fn samples_background_gradients(
    base_styling: QRCodeStylingBuilder,
    base_background: BackgroundOptions,
) -> qr_code_styling::error::Result<Vec<Sample>> {
    use qr_code_styling::ColorStop;
    let stop_0 = ColorStop::new(0.0, Color::from_hex("#FFEE88").unwrap());
    let stop_1 = ColorStop::new(0.5, Color::from_hex("#AA1155").unwrap());
    let stop_2 = ColorStop::new(1.0, Color::from_hex("#00CC99").unwrap());

    let linear_gradient = Gradient::linear(vec![stop_0.clone(), stop_1.clone(), stop_2.clone()]);
    let linear_gradient_rotated = Gradient::linear_rotated(
        PI / 4.0,
        vec![stop_0.clone(), stop_1.clone(), stop_2.clone()],
    );
    let radial_gradient = Gradient::radial(vec![stop_0.clone(), stop_1.clone(), stop_2.clone()]);
    let gradient_options = vec![
        ("linear_gradient", linear_gradient),
        ("linear_gradient_rotated", linear_gradient_rotated),
        ("radial_gradient", radial_gradient),
    ];

    let mut styles = vec![];
    for (name, gradient) in gradient_options {
        let style = base_styling
            .clone()
            .background_options(base_background.clone().with_gradient(gradient))
            .build()?;

        styles.push(Sample {
            name: name.to_string(),
            style,
        });
    }

    Ok(styles)
}

/// Returns a list of sample QR code configurations with different dot coloring.
fn sample_dots_colors(
    base_styling: QRCodeStylingBuilder,
    base_dot_options: DotsOptions,
) -> qr_code_styling::error::Result<Vec<Sample>> {
    let blue = Color::from_hex("#044389").unwrap();
    let cherry = Color::from_hex("#AA1155").unwrap();
    let yellow = Color::from_hex("#FFEE88").unwrap();
    let green = Color::from_hex("#00CC99").unwrap();

    let solid_color = base_dot_options.clone().with_color(blue.clone());
    let color_stops = vec![
        ColorStop::new(0.0, blue),
        ColorStop::new(0.5, cherry),
        ColorStop::new(1.0, yellow),
    ];

    let gradient = Gradient::linear_rotated(PI / 6.0, color_stops.clone());
    let gradient_color = base_dot_options.clone().with_gradient(gradient.clone());
    let round_whole_pixels_on = base_dot_options.clone().with_round_size(true);
    let round_whole_pixels_off = base_dot_options.clone().with_round_size(false);
    let solid_and_gradient = base_dot_options
        .clone()
        .with_color(green)
        .with_gradient(gradient);

    let mut styles = Vec::new();
    for (name, dot_options) in vec![
        ("solid_color", solid_color),
        ("gradient_color", gradient_color),
        ("round_whole_pixels_on", round_whole_pixels_on),
        ("round_whole_pixels_off", round_whole_pixels_off),
        ("solid_and_gradient", solid_and_gradient),
    ] {
        let style = base_styling.clone().dots_options(dot_options).build()?;
        styles.push(Sample {
            name: name.to_string(),
            style,
        });
    }

    Ok(styles)
}

/// Returns a list of sample QR code configurations with different [dot types](DotType).
///
/// Receives a basic configuration to act as the base styling for generated samples.
fn samples_dot_types(
    base_styling: QRCodeStylingBuilder,
    base_dot_options: DotsOptions,
) -> qr_code_styling::error::Result<Vec<Sample>> {
    let dot_types = DotType::all();
    let mut styles = vec![];

    for dot_type in dot_types {
        let name = match dot_type {
            DotType::Square => "square",
            DotType::Dots => "dots",
            DotType::Rounded => "rounded",
            DotType::Classy => "classy",
            DotType::ClassyRounded => "classy_rounded",
            DotType::ExtraRounded => "extra_rounded",
        };
        let dot_options = base_dot_options.clone().with_type(*dot_type);

        let style = base_styling.clone().dots_options(dot_options).build()?;

        styles.push(Sample {
            name: name.to_string(),
            style,
        });
    }

    Ok(styles)
}

/// Returns a comprehensive list of sample QR code configurations with different corner dot types
/// and color schemes (solid, gradient, and combinations).
///
/// Receives a basic configuration to act as the base styling for generated samples,
/// base dot options for consistent dot styling, and a list of color treatments to apply.
fn samples_corners_dots_comprehensive(
    base_styling: QRCodeStylingBuilder,
    color_treatments: Vec<(&'static str, ColorTreatment)>,
) -> qr_code_styling::error::Result<Vec<Sample>> {
    let corner_dot_types = CornerDotType::all();
    let mut styles = vec![];

    for corner_dot_type in corner_dot_types {
        let type_name = match corner_dot_type {
            CornerDotType::Dot => "dot",
            CornerDotType::Square => "square",
        };

        for (treatment_name, treatment) in &color_treatments {
            let base_options = CornersDotOptions::new(*corner_dot_type);
            let corners_dot_options = treatment.apply_to_corners_dot(base_options);

            let style = base_styling
                .clone()
                .corners_dot_options(corners_dot_options)
                .build()?;

            styles.push(Sample {
                name: format!("{}_{}", type_name, treatment_name),
                style,
            });
        }
    }

    Ok(styles)
}

/// Returns a comprehensive list of sample QR code configurations with different corner square types
/// and color schemes (solid, gradient, and combinations).
///
/// Receives a basic configuration to act as the base styling for generated samples,
/// base dot options for consistent dot styling, and a list of color treatments to apply.
fn samples_corners_squares_comprehensive(
    base_styling: QRCodeStylingBuilder,
    color_treatments: Vec<(&'static str, ColorTreatment)>,
) -> qr_code_styling::error::Result<Vec<Sample>> {
    let corner_square_types = CornerSquareType::all();
    let mut styles = vec![];

    for corner_square_type in corner_square_types {
        let type_name = match corner_square_type {
            CornerSquareType::Square => "square",
            CornerSquareType::Dot => "dot",
            CornerSquareType::ExtraRounded => "extra_rounded",
        };

        for (treatment_name, treatment) in &color_treatments {
            let base_options = CornersSquareOptions::new(*corner_square_type);
            let corners_square_options = treatment.apply_to_corners_square(base_options);

            let style = base_styling
                .clone()
                .corners_square_options(corners_square_options)
                .build()?;

            styles.push(Sample {
                name: format!("{}_{}", type_name, treatment_name),
                style,
            });
        }
    }

    Ok(styles)
}

/// Returns a list of sample QR code configurations with different [shape types](ShapeType).
///
/// Receives a basic configuration to act as the base styling for generated samples.
fn samples_shapes(
    base_styling: QRCodeStylingBuilder,
) -> qr_code_styling::error::Result<Vec<Sample>> {
    let shape_types = vec![ShapeType::Square, ShapeType::Circle];
    let mut styles = vec![];

    for shape_type in shape_types {
        let name = match shape_type {
            ShapeType::Square => "square",
            ShapeType::Circle => "circle",
        };

        let style = base_styling
            .clone()
            .shape(shape_type)
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
            .build()?;

        styles.push(Sample {
            name: name.to_string(),
            style,
        });
    }

    Ok(styles)
}
