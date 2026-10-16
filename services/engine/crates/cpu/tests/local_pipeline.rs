use mosaicforge_core::{contract::GenerationSpec, template::import_svg, validate_result};
use mosaicforge_cpu::{
    export::generate_and_export,
    render::{MAX_RENDER_BUFFER_BYTES, render_png},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};

fn save(path: &Path, bytes: &[u8]) {
    if path.exists() {
        assert_eq!(fs::read(path).unwrap(), bytes, "{}", path.display());
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
#[ignore = "requires root test.svg, out/test-prepared.svg and out/test-5.3-{0,1,17}.{svg,png}"]
fn supplied_artwork_runs_full_generation_export_and_hash_pipeline() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    let original = fs::read(root.join("test.svg")).unwrap();
    let source = import_svg(&original).unwrap();
    let prepared = fs::read(root.join("out/test-prepared.svg")).unwrap();
    assert_eq!(
        source.normalized_svg,
        import_svg(&prepared).unwrap().normalized_svg
    );
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
    let mut spec: GenerationSpec = serde_json::from_value(value).unwrap();
    spec.output.background = "#ffffff".parse().unwrap();
    for index in [0, 1, 17] {
        let mut svg_bytes = Vec::new();
        for format in ["svg", "png"] {
            spec.output.format = format.parse().unwrap();
            let exported =
                generate_and_export(&source, &spec, index, MAX_RENDER_BUFFER_BYTES).unwrap();
            let result = &exported.variant.result;
            validate_result(result).unwrap();
            assert_eq!(result.status.to_string(), "accepted");
            let artifact = exported.artifact.as_ref().unwrap();
            assert_eq!(
                result.artifact_hash.as_ref().unwrap().as_str(),
                format!("{:x}", Sha256::digest(artifact))
            );
            assert_eq!(
                artifact,
                &fs::read(root.join(format!("out/test-5.3-{index}.{format}"))).unwrap()
            );
            let mut subset = spec.clone();
            subset.start_index = index.to_string().parse().unwrap();
            subset.count = 1.try_into().unwrap();
            let repeated =
                generate_and_export(&source, &subset, index, MAX_RENDER_BUFFER_BYTES).unwrap();
            assert_eq!(repeated.artifact.as_ref(), Some(artifact));
            assert_eq!(
                serde_json::to_value(&repeated.variant.result).unwrap(),
                serde_json::to_value(result).unwrap()
            );
            if format == "svg" {
                let safe = import_svg(artifact).unwrap();
                assert_eq!((safe.width, safe.height), (512.0, 512.0));
                svg_bytes = artifact.clone();
            } else {
                let image = resvg::tiny_skia::Pixmap::decode_png(artifact).unwrap();
                assert_eq!((image.width(), image.height()), (512, 512));
                assert!(image.pixels().iter().all(|pixel| pixel.alpha() == 255));
                let mut transparent = spec.output.clone();
                transparent.background = "transparent".parse().unwrap();
                let from_svg = render_png(&svg_bytes, &transparent).unwrap();
                assert_eq!(
                    image.data(),
                    resvg::tiny_skia::Pixmap::decode_png(&from_svg)
                        .unwrap()
                        .data()
                );
            }
            save(
                &root.join(format!("out/test-5.5-{index}.{format}")),
                artifact,
            );
            save(
                &root.join(format!("out/test-5.5-{index}-{format}.result.json")),
                &serde_json::to_vec_pretty(result).unwrap(),
            );
            println!(
                "index={index} format={format} bytes={} hash={} attempts={}",
                artifact.len(),
                result.artifact_hash.as_ref().unwrap().as_str(),
                exported.variant.attempts
            );
        }
    }
    for format in ["svg", "png"] {
        spec.output.format = format.parse().unwrap();
        save(
            &root.join(format!("out/test-5.5-{format}.spec.json")),
            &serde_json::to_vec_pretty(&spec).unwrap(),
        );
    }
    spec.rules.opacity.min = 0.0;
    spec.rules.opacity.max = 0.0;
    let rejected = generate_and_export(&source, &spec, 17, 0).unwrap();
    assert_eq!(rejected.variant.result.status.to_string(), "rejected");
    assert!(rejected.artifact.is_none() && rejected.variant.result.artifact_hash.is_none());
    assert_eq!(
        rejected.variant.result.reason.as_ref().unwrap().as_str(),
        "empty_viewport"
    );
    println!(
        "rejected index={} reason={} attempts={}",
        rejected.variant.result.index.as_str(),
        rejected.variant.result.reason.as_ref().unwrap().as_str(),
        rejected.variant.attempts
    );
    assert_eq!(fs::read(root.join("test.svg")).unwrap(), original);
    assert_eq!(
        fs::read(root.join("out/test-prepared.svg")).unwrap(),
        prepared
    );
    println!("source SHA256={:x}; unchanged", Sha256::digest(original));
}
