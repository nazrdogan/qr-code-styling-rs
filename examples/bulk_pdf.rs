//! Example demonstrating bulk PDF generation (10,000 QR codes in one PDF).

use lopdf::{Document, Object, ObjectId};
use qr_code_styling::config::{Color, DotsOptions, ImageOptions};
use qr_code_styling::plugins::border::{BorderPlugin, Position, QRBorderOptions};
use qr_code_styling::rendering::PdfRenderer;
use qr_code_styling::types::{DotType, ShapeType};
use qr_code_styling::QRCodeStyling;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let total_count = 10_000;
    let output_dir = Path::new("output/bulk_pdf");

    // Load logo once and share across threads
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let logo_path = PathBuf::from(manifest_dir).join("examples/logo.png");
    let logo_bytes = Arc::new(fs::read(&logo_path)?);
    println!("Loaded logo: {} bytes", logo_bytes.len());

    // Create output directory
    fs::create_dir_all(output_dir)?;
    println!("Output directory: {}", output_dir.display());

    println!("\nGenerating {} QR codes into single PDF...\n", total_count);

    let start = Instant::now();

    // Generate individual PDFs first (in parallel for speed)
    println!("Step 1: Generating individual QR code pages...");

    use rayon::prelude::*;

    let pdfs: Vec<_> = (0..total_count)
        .into_par_iter()
        .map(|i| {
            let result = generate_qr_pdf_bytes(i, &logo_bytes);
            if (i + 1) % 1000 == 0 {
                println!("  Generated {}/{} pages", i + 1, total_count);
            }
            (i, result)
        })
        .collect();

    println!("\nStep 2: Merging {} pages into single PDF...", total_count);

    // Merge all PDFs into one
    let merged = merge_pdfs(pdfs)?;

    let output_path = output_dir.join("all_qr_codes.pdf");
    fs::write(&output_path, merged)?;

    let duration = start.elapsed();

    println!("\n========================================");
    println!("Bulk PDF Generation Complete!");
    println!("========================================");
    println!("Total pages: {}", total_count);
    println!("Time elapsed: {:.2?}", duration);
    println!("Output file: {}", output_path.display());
    println!("File size: {:.2} MB", fs::metadata(&output_path)?.len() as f64 / 1024.0 / 1024.0);

    Ok(())
}

fn generate_qr_pdf_bytes(index: usize, logo_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Generate unique data for each QR code
    let data = format!("https://example.com/item/{:05}", index);

    // Vary colors based on index
    let colors = ["#E74C3C", "#3498DB", "#2ECC71", "#9B59B6", "#F39C12"];
    let color = colors[index % colors.len()];

    // Build QR code with logo
    let qr = QRCodeStyling::builder()
        .data(&data)
        .size(300)
        .margin(40)
        .shape(ShapeType::Circle)
        .image(logo_bytes.to_vec())
        .image_options(
            ImageOptions::default()
                .with_image_size(0.3)
                .with_margin(5)
                .with_hide_background_dots(true),
        )
        .dots_options(
            DotsOptions::new(DotType::Rounded)
                .with_color(Color::from_hex(color).unwrap()),
        )
        .build()?;

    let svg = qr.render_svg()?;

    // Add border with text
    let text_style = "font-size: 14px; font-family: Arial, sans-serif; fill: #FFFFFF; font-weight: bold;";

    let border_options = QRBorderOptions::new(30.0, color)
        .with_round(1.0)
        .with_styled_text(Position::Top, "SCAN ME", text_style)
        .with_styled_text(Position::Bottom, &format!("#{:05}", index), text_style);

    let border_plugin = BorderPlugin::new(border_options);
    let svg_with_border = border_plugin.apply(&svg, 300, 300);

    // Convert to PDF
    let pdf_data = PdfRenderer::render_from_svg(&svg_with_border, 300, 300)?;

    Ok(pdf_data)
}

fn merge_pdfs(pdfs: Vec<(usize, Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>)>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Sort by index to ensure correct order
    let mut sorted_pdfs: Vec<_> = pdfs.into_iter()
        .filter_map(|(i, r)| r.ok().map(|data| (i, data)))
        .collect();
    sorted_pdfs.sort_by_key(|(i, _)| *i);

    if sorted_pdfs.is_empty() {
        return Err("No PDFs to merge".into());
    }

    // Start with first PDF as base
    let (_, first_pdf) = sorted_pdfs.remove(0);
    let mut merged_doc = Document::load_mem(&first_pdf)?;

    let mut page_count = 1;

    // Add pages from remaining PDFs
    for (idx, pdf_data) in sorted_pdfs {
        if let Ok(doc) = Document::load_mem(&pdf_data) {
            // Get pages from source document
            let pages = doc.get_pages();

            for (_, &page_id) in pages.iter() {
                // Clone page and its resources to merged document
                if let Ok(_) = clone_page_to_document(&doc, &mut merged_doc, page_id) {
                    page_count += 1;
                }
            }
        }

        if (idx + 1) % 1000 == 0 {
            println!("  Merged {}/{} pages", idx + 1, page_count);
        }
    }

    // Save merged document
    let mut buffer = Vec::new();
    merged_doc.save_to(&mut buffer)?;

    Ok(buffer)
}

fn clone_page_to_document(
    source: &Document,
    target: &mut Document,
    page_id: ObjectId,
) -> Result<ObjectId, Box<dyn std::error::Error>> {
    // Get all objects that need to be copied
    let mut objects_to_copy: BTreeMap<ObjectId, Object> = BTreeMap::new();
    collect_page_objects(source, page_id, &mut objects_to_copy)?;

    // Create ID mapping for copied objects
    let mut id_map: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();

    // Allocate new IDs in target document
    for old_id in objects_to_copy.keys() {
        let new_id = target.new_object_id();
        id_map.insert(*old_id, new_id);
    }

    // Copy objects with updated references
    for (old_id, obj) in objects_to_copy {
        let new_obj = remap_object_references(&obj, &id_map);
        let new_id = id_map[&old_id];
        target.objects.insert(new_id, new_obj);
    }

    // Add new page to target's page tree
    let new_page_id = id_map[&page_id];

    // Get target's pages object
    if let Ok(pages_id) = target.catalog()?.get(b"Pages")?.as_reference() {
        if let Ok(pages) = target.get_object_mut(pages_id) {
            if let Object::Dictionary(ref mut dict) = pages {
                // Update Kids array
                if let Ok(kids) = dict.get_mut(b"Kids") {
                    if let Object::Array(ref mut arr) = kids {
                        arr.push(Object::Reference(new_page_id));
                    }
                }
                // Update Count
                if let Ok(count) = dict.get_mut(b"Count") {
                    if let Object::Integer(ref mut n) = count {
                        *n += 1;
                    }
                }
            }
        }

        // Update page's Parent reference
        if let Ok(page) = target.get_object_mut(new_page_id) {
            if let Object::Dictionary(ref mut dict) = page {
                dict.set("Parent", Object::Reference(pages_id));
            }
        }
    }

    Ok(new_page_id)
}

fn collect_page_objects(
    doc: &Document,
    obj_id: ObjectId,
    collected: &mut BTreeMap<ObjectId, Object>,
) -> Result<(), Box<dyn std::error::Error>> {
    if collected.contains_key(&obj_id) {
        return Ok(());
    }

    if let Ok(obj) = doc.get_object(obj_id) {
        collected.insert(obj_id, obj.clone());

        // Recursively collect referenced objects
        collect_references(doc, obj, collected)?;
    }

    Ok(())
}

fn collect_references(
    doc: &Document,
    obj: &Object,
    collected: &mut BTreeMap<ObjectId, Object>,
) -> Result<(), Box<dyn std::error::Error>> {
    match obj {
        Object::Reference(id) => {
            // Don't follow Parent references to avoid cycles
            collect_page_objects(doc, *id, collected)?;
        }
        Object::Array(arr) => {
            for item in arr {
                collect_references(doc, item, collected)?;
            }
        }
        Object::Dictionary(dict) => {
            for (key, value) in dict.iter() {
                // Skip Parent to avoid circular references
                if key != b"Parent" {
                    collect_references(doc, value, collected)?;
                }
            }
        }
        Object::Stream(stream) => {
            for (key, value) in stream.dict.iter() {
                if key != b"Parent" {
                    collect_references(doc, value, collected)?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn remap_object_references(obj: &Object, id_map: &BTreeMap<ObjectId, ObjectId>) -> Object {
    match obj {
        Object::Reference(id) => {
            if let Some(new_id) = id_map.get(id) {
                Object::Reference(*new_id)
            } else {
                obj.clone()
            }
        }
        Object::Array(arr) => {
            Object::Array(arr.iter().map(|item| remap_object_references(item, id_map)).collect())
        }
        Object::Dictionary(dict) => {
            let mut new_dict = lopdf::Dictionary::new();
            for (key, value) in dict.iter() {
                new_dict.set(key.clone(), remap_object_references(value, id_map));
            }
            Object::Dictionary(new_dict)
        }
        Object::Stream(stream) => {
            let mut new_dict = lopdf::Dictionary::new();
            for (key, value) in stream.dict.iter() {
                new_dict.set(key.clone(), remap_object_references(value, id_map));
            }
            Object::Stream(lopdf::Stream::new(new_dict, stream.content.clone()))
        }
        _ => obj.clone(),
    }
}
