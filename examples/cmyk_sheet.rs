//! Lay out 25 CMYK QR codes on one A4 sheet using Form XObjects.
//!
//! Run with: cargo run --example cmyk_sheet --features cmyk

use qr_code_styling::pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref};
use qr_code_styling::{CmykImageCache, CmykPdfOptions, CornerSquareType, CornersSquareOptions, DotType, DotsOptions, QRCodeStyling};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const A4: (f32, f32) = (595.0, 842.0);
    const COLS: usize = 5;
    const ROWS: usize = 5;
    const CELL: f32 = 100.0; // sticker size in pt
    const GAP: f32 = 12.0;

    let mut pdf = Pdf::new();
    let mut alloc = Ref::new(1);
    let (catalog, tree, page, contents) = (alloc.bump(), alloc.bump(), alloc.bump(), alloc.bump());

    let options = CmykPdfOptions::new();
    // Images shared by several codes (e.g. a logo) are embedded once
    let mut images = CmykImageCache::new();
    let grid_w = COLS as f32 * CELL + (COLS - 1) as f32 * GAP;
    let grid_h = ROWS as f32 * CELL + (ROWS - 1) as f32 * GAP;
    let (left, top) = ((A4.0 - grid_w) / 2.0, (A4.1 + grid_h) / 2.0);

    let mut content = Content::new();
    let mut forms = Vec::new();
    for i in 0..COLS * ROWS {
        let qr = QRCodeStyling::builder()
            .data(format!("https://example.com/item/{:05}", i))
            .size(300)
            .margin(10)
            .dots_options(DotsOptions::new(DotType::Rounded))
            .corners_square_options(CornersSquareOptions::new(CornerSquareType::ExtraRounded))
            .build()?;
        let form = qr.write_cmyk_xobject_cached(&mut pdf, &mut alloc, &options, &mut images)?;

        let (col, row) = (i % COLS, i / COLS);
        let x = left + col as f32 * (CELL + GAP);
        let y = top - (row + 1) as f32 * CELL - row as f32 * GAP; // PDF y grows upward
        let name = format!("Qr{}", i);
        content.save_state();
        content.transform([CELL / form.width, 0.0, 0.0, CELL / form.height, x, y]);
        content.x_object(Name(name.as_bytes()));
        content.restore_state();
        forms.push((name, form.id));
    }
    pdf.stream(contents, &content.finish());

    pdf.catalog(catalog).pages(tree);
    pdf.pages(tree).kids([page]).count(1);
    let mut p = pdf.page(page);
    p.media_box(Rect::new(0.0, 0.0, A4.0, A4.1));
    p.parent(tree);
    p.contents(contents);
    let mut resources = p.resources();
    let mut x_objects = resources.x_objects();
    for (name, id) in &forms {
        x_objects.pair(Name(name.as_bytes()), *id);
    }
    x_objects.finish();
    resources.finish();
    p.finish();

    std::fs::create_dir_all("output")?;
    std::fs::write("output/qr_cmyk_sheet.pdf", pdf.finish())?;
    println!("Saved output/qr_cmyk_sheet.pdf ({} codes)", forms.len());
    Ok(())
}
