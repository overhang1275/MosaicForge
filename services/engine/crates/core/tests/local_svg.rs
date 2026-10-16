use mosaicforge_core::{
    appearance::apply_appearance, contract::GenerationSpec, generate::apply_geometric_transform,
    rules::sample_transform, template::import_svg,
};
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path};

// User artwork is deliberately not embedded in or required by the repository's test suite.
#[test]
#[ignore = "requires root test.svg and scripts/prepare-test-svg.py output"]
fn supplied_svg_exercises_geometry_and_appearance() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    let bytes = fs::read(root.join("test.svg")).unwrap();
    let source = import_svg(&bytes).unwrap();
    let prepared = import_svg(&fs::read(root.join("out/test-prepared.svg")).unwrap()).unwrap();
    assert_eq!(source.normalized_svg, prepared.normalized_svg);
    assert_eq!(source.hash, prepared.hash);
    assert_eq!(
        source.hash,
        "e8cfd87f3c8684427494222a03db48bbc8f2bfa1e5c6170ede7a556046474cac"
    );
    println!(
        "Original template: viewBox={:?}, layers={}, segments={}, hash={}",
        source.view_box,
        source.layer_ids.len(),
        source.segment_count,
        source.hash
    );
    let mut value: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    value["templateHash"] = json!(source.hash);
    value["rules"]["rotationDegrees"] = json!({"min":-15,"max":15});
    value["rules"]["palette"] = json!(["#ed5a33", "#187d9b", "#7346ad"]);
    value["rules"]["opacity"] = json!({"min":0.35,"max":1});
    value["rules"]["shuffleLayers"] = json!(true);
    value["rules"]["crop"] = json!({"x":12,"y":12,"width":120,"height":120});
    let spec: GenerationSpec = serde_json::from_value(value).unwrap();
    for index in [0, 1, 17] {
        let transformed =
            apply_geometric_transform(&source, &sample_transform(&spec, index, 0).unwrap())
                .unwrap();
        let result = apply_appearance(&transformed, &spec, index, 0).unwrap();
        assert_eq!(result.segment_count, source.segment_count);
        assert_eq!(result.view_box, [12.0, 12.0, 120.0, 120.0]);
        let mut subset = spec.clone();
        subset.start_index = index.to_string().parse().unwrap();
        subset.count = 1.try_into().unwrap();
        let repeated_geometry =
            apply_geometric_transform(&source, &sample_transform(&subset, index, 0).unwrap())
                .unwrap();
        assert_eq!(
            result.normalized_svg,
            apply_appearance(&repeated_geometry, &subset, index, 0)
                .unwrap()
                .normalized_svg
        );
        let path = root.join(format!("out/test-4.3-{index}.svg"));
        if path.exists() {
            assert_eq!(fs::read(&path).unwrap(), result.normalized_svg.as_bytes());
        } else {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap()
                .write_all(result.normalized_svg.as_bytes())
                .unwrap();
        }
        println!("index={index} hash={} -> {}", result.hash, path.display());
    }
    assert_eq!(fs::read(root.join("test.svg")).unwrap(), bytes);
}

#[test]
#[ignore = "requires the local out/test-4.3 SVG samples"]
fn inspect_supplied_svg_framing() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    for index in [0, 1, 17] {
        let path = root.join(format!("out/test-4.3-{index}.svg"));
        let source = import_svg(&fs::read(&path).unwrap()).unwrap();
        let tree = usvg::Tree::from_str(&source.normalized_svg, &usvg::Options::default()).unwrap();
        let mut image = resvg::tiny_skia::Pixmap::new(512, 512).unwrap();
        let transform = resvg::tiny_skia::Transform::from_scale(
            512.0 / tree.size().width(),
            512.0 / tree.size().height(),
        );
        resvg::render(&tree, transform, &mut image.as_mut());
        image
            .save_png(root.join(format!("out/test-4.3-{index}.png")))
            .unwrap();
        println!(
            "index {index}: {:?}",
            mosaicforge_core::constraints::measure_coverage(&source).unwrap()
        );
    }
}

#[test]
#[ignore = "requires root test.svg and out/test-prepared.svg"]
fn supplied_svg_has_varied_framing_without_blank_corners() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    let source = import_svg(&fs::read(root.join("out/test-prepared.svg")).unwrap()).unwrap();
    let mut value: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    value["templateHash"] = json!(source.hash);
    value["rules"]["rotationDegrees"] = json!({"min":-180,"max":180});
    value["rules"]["translateX"] = json!({"min":-14,"max":14});
    value["rules"]["translateY"] = json!({"min":-14,"max":14});
    value["rules"]["scaleX"] = json!({"min":1.05,"max":1.55});
    value["rules"]["scaleY"] = json!({"min":1.05,"max":1.55});
    value["rules"]["opacity"] = json!({"min":0.8,"max":1});
    value["rules"]["palette"] = json!(["#ed5a33", "#187d9b", "#7346ad"]);
    value["rules"]["crop"] = json!({"x":12,"y":12,"width":120,"height":120});
    let spec: GenerationSpec = serde_json::from_value(value).unwrap();
    let mut hashes = std::collections::BTreeSet::new();
    for index in [0, 1, 17] {
        let decision = mosaicforge_core::generate::generate_variant(&source, &spec, index).unwrap();
        mosaicforge_core::validate_result(&decision.result).unwrap();
        println!(
            "new index={index} attempts={} parameters={:?} coverage={:?} reason={:?}",
            decision.attempts, decision.parameters, decision.coverage, decision.result.reason
        );
        let result = decision.template.expect("sample must pass coverage");
        hashes.insert(result.hash.clone());
        let mut subset = spec.clone();
        subset.start_index = index.to_string().parse().unwrap();
        subset.count = 1.try_into().unwrap();
        assert_eq!(
            result.hash,
            mosaicforge_core::generate::generate_variant(&source, &subset, index)
                .unwrap()
                .template
                .unwrap()
                .hash
        );
        let tree = usvg::Tree::from_str(&result.normalized_svg, &usvg::Options::default()).unwrap();
        let mut image = resvg::tiny_skia::Pixmap::new(512, 512).unwrap();
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::from_scale(
                512.0 / tree.size().width(),
                512.0 / tree.size().height(),
            ),
            &mut image.as_mut(),
        );
        let path = root.join(format!("out/test-4.4-{index}.svg"));
        if path.exists() {
            assert_eq!(fs::read(&path).unwrap(), result.normalized_svg.as_bytes());
        } else {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap()
                .write_all(result.normalized_svg.as_bytes())
                .unwrap();
        }
        image
            .save_png(root.join(format!("out/test-4.4-{index}.png")))
            .unwrap();
    }
    assert_eq!(hashes.len(), 3);
}
