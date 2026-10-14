use mosaicforge_core::{
    configuration_hash,
    contract::GenerationSpec,
    generate::{Variant, generate_variant},
    template::{Template, import_svg},
    validate_result,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const SVG:&[u8]=br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><g id="base"><path d="M0 0L100 0L100 100L0 100Z" fill="#123456"/></g><g id="detail"><rect x="30" y="30" width="40" height="40" fill="#abcdef"/></g></svg>"##;

fn setup() -> (Template, GenerationSpec) {
    let source = import_svg(SVG).unwrap();
    let mut value: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    value["templateHash"] = json!(source.hash);
    (source, serde_json::from_value(value).unwrap())
}

fn signature(v: Variant) -> String {
    format!(
        "{}|{:?}|{:?}|{}|{}",
        serde_json::to_string(&v.result).unwrap(),
        v.parameters,
        v.coverage,
        v.attempts,
        v.template.map_or(String::new(), |t| t.normalized_svg)
    )
}

#[test]
fn accepted_variant_has_valid_bound_metadata_and_internal_svg() {
    let (source, spec) = setup();
    let v = generate_variant(&source, &spec, 17).unwrap();
    validate_result(&v.result).unwrap();
    let wire = serde_json::to_value(&v.result).unwrap();
    assert_eq!(wire["status"], "accepted");
    assert_eq!(wire["index"], "17");
    assert_eq!(wire["seed"], "18446744073709551615");
    assert_eq!(wire["templateHash"], source.hash);
    assert_eq!(
        wire["configurationHash"],
        configuration_hash(&spec).unwrap()
    );
    assert_eq!(wire["backend"], "cpu");
    assert_eq!(wire["device"], "cpu");
    assert!(v.result.reason.is_none());
    // Artifact bytes (dimensions/background/encoding) are produced by the exporter in task 5.
    assert!(v.result.artifact_hash.is_none());
    assert_eq!(v.template.unwrap().hash, source.hash);
    assert!(v.parameters.is_some() && v.coverage.is_some());
    assert_eq!(v.attempts, 1);
}

#[test]
fn generator_checks_template_binding_and_index_interval() {
    let (source, mut spec) = setup();
    spec.start_index = "100".parse().unwrap();
    spec.count = 100.try_into().unwrap();
    assert!(generate_variant(&source, &spec, 99).is_err());
    assert!(generate_variant(&source, &spec, 200).is_err());
    let valid_hash = spec.template_hash.clone();
    spec.template_hash = "0".repeat(64).parse().unwrap();
    assert!(generate_variant(&source, &spec, 100).is_err());
    spec.template_hash = valid_hash;
    let mut forged = source;
    forged.hash = "0".repeat(64);
    forged.view_box = [0.0; 4];
    assert!(generate_variant(&forged, &spec, 100).is_ok());
    forged.normalized_svg = "<svg><script/></svg>".into();
    assert!(generate_variant(&forged, &spec, 100).is_err());
}

#[test]
fn range_and_schedule_do_not_change_variants() {
    let (source, mut spec) = setup();
    spec.rules.rotation_degrees.min = -180.0;
    spec.rules.rotation_degrees.max = 180.0;
    spec.rules.scale_x.max = 2.0;
    spec.rules.scale_y.max = 2.0;
    spec.rules.opacity.min = 0.5;
    spec.rules.palette = vec!["#ee5500".parse().unwrap(), "#0055ee".parse().unwrap()];
    spec.rules.shuffle_layers = true;
    spec.rules.distortion = 0.25;
    let expected: BTreeMap<_, _> = (0..200)
        .map(|i| (i, signature(generate_variant(&source, &spec, i).unwrap())))
        .collect();
    let mut subset = spec.clone();
    subset.start_index = "100".parse().unwrap();
    subset.count = 100.try_into().unwrap();
    for i in 100..200 {
        assert_eq!(
            signature(generate_variant(&source, &subset, i).unwrap()),
            expected[&i]
        );
    }
    for block in (0..200).step_by(17).collect::<Vec<_>>().into_iter().rev() {
        let mut group = spec.clone();
        group.start_index = block.to_string().parse().unwrap();
        group.count = (200 - block).min(17).try_into().unwrap();
        for i in (block..block + group.count.get()).rev() {
            assert_eq!(
                signature(generate_variant(&source, &group, i).unwrap()),
                expected[&i]
            );
        }
    }
    for workers in [1, 2] {
        let actual = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..workers)
                .map(|worker| {
                    let source = &source;
                    let spec = &spec;
                    scope.spawn(move || {
                        (worker..200)
                            .step_by(workers as usize)
                            .map(|i| (i, signature(generate_variant(source, spec, i).unwrap())))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles
                .into_iter()
                .flat_map(|h| h.join().unwrap())
                .collect::<BTreeMap<_, _>>()
        });
        assert_eq!(expected, actual);
    }
}

#[test]
fn rejected_result_keeps_index_and_does_not_advance_the_next_variant() {
    let (source, spec) = setup();
    let expected = signature(generate_variant(&source, &spec, 18).unwrap());
    let mut invisible = spec.clone();
    invisible.rules.opacity.min = 0.0;
    invisible.rules.opacity.max = 0.0;
    let rejected = generate_variant(&source, &invisible, 17).unwrap();
    validate_result(&rejected.result).unwrap();
    let wire = serde_json::to_value(&rejected.result).unwrap();
    assert_eq!(wire["index"], "17");
    assert_eq!(wire["status"], "rejected");
    assert_eq!(wire["reason"], "empty_viewport");
    assert!(
        rejected.template.is_none()
            && rejected.parameters.is_none()
            && rejected.result.artifact_hash.is_none()
    );
    assert_eq!(rejected.attempts, 8);
    assert_eq!(
        signature(generate_variant(&source, &spec, 18).unwrap()),
        expected
    );
}

#[test]
fn maximum_u64_index_and_png_request_keep_correct_metadata() {
    let (source, mut spec) = setup();
    spec.start_index = u64::MAX.to_string().parse().unwrap();
    spec.count = 1.try_into().unwrap();
    spec.output.format = "png".parse().unwrap();
    let v = generate_variant(&source, &spec, u64::MAX).unwrap();
    assert_eq!(v.result.index.to_string(), u64::MAX.to_string());
    assert!(v.template.is_some() && v.result.artifact_hash.is_none());
    validate_result(&v.result).unwrap();
}
