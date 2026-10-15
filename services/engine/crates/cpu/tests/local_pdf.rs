use mosaicforge_core::{contract::PdfJobSpec, pdf::page_boxes, template::import_svg};
use mosaicforge_cpu::{export::generate_and_export, pdf::generate_pdf};
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path};
fn save(path: &Path, bytes: &[u8]) {
    if path.exists() {
        assert_eq!(fs::read(path).unwrap(), bytes);
    } else {
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap()
            .write_all(bytes)
            .unwrap();
    }
}
#[test]
#[ignore = "requires out/test-prepared.svg and local PDF QA runtime"]
fn supplied_artwork_exports_one_physical_pdf_and_reference_images() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    let source = import_svg(&fs::read(root.join("out/test-prepared.svg")).unwrap()).unwrap();
    let mut g: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    g["templateHash"] = json!(source.hash);
    g["count"] = json!(3);
    g["rules"]["rotationDegrees"] = json!({"min":-180,"max":180});
    g["rules"]["translateX"] = json!({"min":-14,"max":14});
    g["rules"]["translateY"] = json!({"min":-14,"max":14});
    g["rules"]["scaleX"] = json!({"min":1.05,"max":1.55});
    g["rules"]["scaleY"] = json!({"min":1.05,"max":1.55});
    g["rules"]["opacity"] = json!({"min":0.8,"max":1});
    g["rules"]["palette"] = json!(["#ed5a33", "#187d9b", "#7346ad"]);
    g["rules"]["crop"] = json!({"x":12,"y":12,"width":120,"height":120});
    g["output"]["background"] = json!("#ffffff");
    let job:PdfJobSpec=serde_json::from_value(json!({"schemaVersion":"1","generation":g,"pdf":{"exporterVersion":"1","trimWidthMm":100,"trimHeightMm":100,"bleedMm":0}})).unwrap();
    let dir = root.join("out/pdf-5.6-release");
    fs::create_dir_all(&dir).unwrap();
    let target = dir.join("mosaic-100mm.pdf");
    // Test destination, never a user-selected output; product publication belongs to CLI task 6.
    let spool = dir.join(format!("mosaic-100mm.{}.testing.tmp", std::process::id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&spool)
        .unwrap();
    let mut records = Vec::new();
    let result = generate_pdf(&source, &job, 2, &mut file, |r| {
        records.push(r);
        Ok(())
    })
    .unwrap();
    drop(file);
    let bytes = fs::read(&spool).unwrap();
    save(&target, &bytes);
    fs::remove_file(&spool).unwrap();
    assert_eq!(result.page_count, 3);
    save(
        &dir.join("result.json"),
        &serde_json::to_vec_pretty(&result).unwrap(),
    );
    save(
        &dir.join("pages.json"),
        &serde_json::to_vec_pretty(&records).unwrap(),
    );
    save(
        &dir.join("job.json"),
        &serde_json::to_vec_pretty(&job).unwrap(),
    );
    for index in 0..3 {
        let exported = generate_and_export(&source, &job.generation, index, 0).unwrap();
        let svg = exported.artifact.unwrap();
        let optimized = mosaicforge_cpu::pdf::geometry::optimize_page(&svg, &job).unwrap();
        println!("index={index} geometry={:?}", optimized.stats);
        save(&dir.join(format!("source-{index}.svg")), &svg);
        let text = String::from_utf8(svg).unwrap();
        let boxes = page_boxes(&job).unwrap();
        // Match the physical PDF size and Poppler's 300 DPI grid, rather than stretching to ceil pixels.
        let physical = text
            .replacen(
                "width=\"512\"",
                &format!("width=\"{}pt\"", boxes.media[2]),
                1,
            )
            .replacen(
                "height=\"512\"",
                &format!("height=\"{}pt\"", boxes.media[3]),
                1,
            );
        let options = resvg::usvg::Options {
            dpi: 72.0,
            ..Default::default()
        };
        let tree = resvg::usvg::Tree::from_str(&physical, &options).unwrap();
        let mut image = resvg::tiny_skia::Pixmap::new(1182, 1182).unwrap();
        image.fill(resvg::tiny_skia::Color::WHITE);
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(300.0 / 72.0, 300.0 / 72.0),
            &mut image.as_mut(),
        );
        save(
            &dir.join(format!("reference-{index}.png")),
            &image.encode_png().unwrap(),
        );
    }
    println!(
        "PDF pages={} bytes={} hash={}",
        result.page_count,
        bytes.len(),
        result.document_hash.unwrap().as_str()
    );
}
