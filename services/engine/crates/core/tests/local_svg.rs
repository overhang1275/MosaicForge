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
    let error = import_svg(&bytes).unwrap_err();
    println!("Original test.svg rejected: {error}");
    let source = import_svg(&fs::read(root.join("out/test-prepared.svg")).unwrap()).unwrap();
    println!(
        "Prepared template: viewBox={:?}, layers={}, segments={}, hash={}",
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
