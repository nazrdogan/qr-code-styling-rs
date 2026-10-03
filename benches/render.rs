use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qr_code_styling::{DotType, DotsOptions, OutputFormat, QRCodeStyling};

fn qr(data: &str) -> QRCodeStyling {
    QRCodeStyling::builder()
        .data(data)
        .size(600)
        .dots_options(DotsOptions::new(DotType::Rounded))
        .build()
        .unwrap()
}

fn bench_render(c: &mut Criterion) {
    let long = "x".repeat(1200);
    for (label, data) in [("small", "https://example.com"), ("large", long.as_str())] {
        let code = qr(data);
        let mut group = c.benchmark_group(label);
        group.bench_function("build", |b| b.iter(|| qr(black_box(data))));
        group.bench_function("svg", |b| b.iter(|| code.render_svg().unwrap()));
        group.bench_function("png", |b| b.iter(|| code.render(OutputFormat::Png).unwrap()));
        group.bench_function("pdf", |b| b.iter(|| code.render(OutputFormat::Pdf).unwrap()));
        group.finish();
    }
}

criterion_group!(benches, bench_render);
criterion_main!(benches);
