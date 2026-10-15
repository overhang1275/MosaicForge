use mosaicforge_core::{
    configuration_hash,
    contract::GenerationSpec,
    export::export_svg,
    generate::generate_variant,
    template::{Template, import_svg},
    validate_result,
};
use mosaicforge_cpu::{
    export::{ExportedVariant, generate_and_export},
    render::{MAX_RENDER_BUFFER_BYTES, render_png},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn setup(format: &str) -> (Template, GenerationSpec) {
    let source = import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100" fill="#123456"/><rect x="25" y="25" width="20" height="30" fill="#fedcba"/></g></svg>"##).unwrap();
    let mut value: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    value["templateHash"] = json!(source.hash);
    value["output"] = json!({"format":format,"width":32,"height":16,"background":"#abcdef"});
    (source, serde_json::from_value(value).unwrap())
}

fn signature(export: ExportedVariant) -> (String, Option<Vec<u8>>) {
    assert_eq!(export.variant.result.status.to_string(), "accepted");
    assert!(export.artifact.is_some() && export.variant.result.artifact_hash.is_some());
    (
        serde_json::to_string(&export.variant.result).unwrap(),
        export.artifact,
    )
}

#[test]
fn accepted_exports_hash_exact_final_svg_and_png_bytes() {
    for format in ["svg", "png"] {
        let (source, spec) = setup(format);
        let candidate = generate_variant(&source, &spec, 17).unwrap();
        let exported = generate_and_export(&source, &spec, 17, MAX_RENDER_BUFFER_BYTES).unwrap();
        let bytes = exported.artifact.as_ref().unwrap();
        let internal = candidate.template.as_ref().unwrap();
        let expected = if format == "svg" {
            export_svg(internal.normalized_svg.as_bytes(), &spec.output).unwrap()
        } else {
            render_png(internal.normalized_svg.as_bytes(), &spec.output).unwrap()
        };
        assert_eq!(bytes, &expected);
        let result = &exported.variant.result;
        validate_result(result).unwrap();
        assert_eq!(
            result.artifact_hash.as_ref().unwrap().as_str(),
            format!("{:x}", Sha256::digest(bytes))
        );
        assert_ne!(
            result.artifact_hash.as_ref().unwrap().as_str(),
            internal.hash
        );
        assert_eq!(
            result.configuration_hash.as_str(),
            configuration_hash(&spec).unwrap()
        );
        assert_eq!(result.template_hash.as_str(), source.hash);
        assert_eq!(result.index.as_str(), "17");
        assert_eq!(result.status.to_string(), "accepted");
        assert!(result.reason.is_none());
        assert_eq!(exported.variant.parameters, candidate.parameters);
        assert_eq!(exported.variant.coverage, candidate.coverage);
        assert_eq!(exported.variant.attempts, candidate.attempts);
        assert!(candidate.result.artifact_hash.is_none());
        if format == "svg" {
            assert_eq!(import_svg(bytes).unwrap().width, 32.0);
        } else {
            let image = resvg::tiny_skia::Pixmap::decode_png(bytes).unwrap();
            assert_eq!((image.width(), image.height()), (32, 16));
        }
    }
}

#[test]
fn rejected_variants_keep_reason_without_hash_or_bytes_even_with_zero_render_budget() {
    for format in ["svg", "png"] {
        let (source, mut spec) = setup(format);
        spec.rules.opacity.min = 0.0;
        spec.rules.opacity.max = 0.0;
        let exported = generate_and_export(&source, &spec, 17, 0).unwrap();
        let result = exported.variant.result;
        validate_result(&result).unwrap();
        assert_eq!(result.status.to_string(), "rejected");
        assert_eq!(result.index.as_str(), "17");
        assert_eq!(result.reason.unwrap().as_str(), "empty_viewport");
        assert!(exported.artifact.is_none() && result.artifact_hash.is_none());
        assert!(exported.variant.template.is_none());
        assert_eq!(exported.variant.attempts, 8);
    }
}

#[test]
fn final_bytes_and_metadata_ignore_range_order_and_worker_schedule() {
    for format in ["svg", "png"] {
        let (source, mut spec) = setup(format);
        spec.count = 3.try_into().unwrap();
        spec.rules.scale_x.min = 1.5;
        spec.rules.scale_x.max = 2.0;
        spec.rules.scale_y.min = 1.5;
        spec.rules.scale_y.max = 2.0;
        spec.rules.rotation_degrees.min = -20.0;
        spec.rules.rotation_degrees.max = 20.0;
        spec.rules.palette = vec!["#ee5500".parse().unwrap(), "#0055ee".parse().unwrap()];
        let expected: Vec<_> = (0..3)
            .map(|i| {
                signature(generate_and_export(&source, &spec, i, MAX_RENDER_BUFFER_BYTES).unwrap())
            })
            .collect();
        for i in (0..3).rev() {
            let mut subset = spec.clone();
            subset.start_index = i.to_string().parse().unwrap();
            subset.count = 1.try_into().unwrap();
            assert_eq!(
                signature(
                    generate_and_export(&source, &subset, i, MAX_RENDER_BUFFER_BYTES).unwrap()
                ),
                expected[i as usize]
            );
        }
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..3)
                .map(|i| {
                    let source = &source;
                    let spec = &spec;
                    scope.spawn(move || {
                        signature(
                            generate_and_export(source, spec, i, MAX_RENDER_BUFFER_BYTES).unwrap(),
                        )
                    })
                })
                .collect();
            for (actual, expected) in handles.into_iter().zip(expected) {
                assert_eq!(actual.join().unwrap(), expected);
            }
        });
    }
}

#[test]
fn invalid_inputs_and_failed_exports_return_errors_instead_of_successful_metadata() {
    let (source, mut spec) = setup("png");
    assert_eq!(
        generate_and_export(&source, &spec, 17, 0)
            .unwrap_err()
            .code(),
        "invalid_spec"
    );
    // A failed export must not poison the next call or publish partial metadata.
    assert!(
        generate_and_export(&source, &spec, 17, MAX_RENDER_BUFFER_BYTES)
            .unwrap()
            .artifact
            .is_some()
    );
    spec.start_index = "100".parse().unwrap();
    assert!(generate_and_export(&source, &spec, 17, MAX_RENDER_BUFFER_BYTES).is_err());
    spec.start_index = "0".parse().unwrap();
    spec.template_hash = "0".repeat(64).parse().unwrap();
    assert!(generate_and_export(&source, &spec, 17, MAX_RENDER_BUFFER_BYTES).is_err());
    spec.template_hash = source.hash.parse().unwrap();
    spec.output.width = 4097.try_into().unwrap();
    assert!(generate_and_export(&source, &spec, 17, MAX_RENDER_BUFFER_BYTES).is_err());
}
