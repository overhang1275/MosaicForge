use mosaicforge_core::{
    contract::GenerationSpec,
    rng::{map_u32_to_range, sample_u32},
    rules::{TransformParameters, sample_transform},
};
use serde_json::{Value, json};

fn input() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap()
}

#[test]
fn aspect_lock_uses_one_deterministic_sample_and_keeps_legacy_bytes() {
    let legacy: GenerationSpec = serde_json::from_value(input()).unwrap();
    assert!(
        !serde_json::to_value(&legacy).unwrap()["rules"]
            .as_object()
            .unwrap()
            .contains_key("preserveAspectRatio")
    );
    let mut value = varied();
    value["rulesVersion"] = json!("2");
    value["rules"]["preserveAspectRatio"] = json!(true);
    value["rules"]["scaleY"] = value["rules"]["scaleX"].clone();
    let spec: GenerationSpec = serde_json::from_value(value.clone()).unwrap();
    for index in [0, 1, 2, 42, u64::MAX] {
        for attempt in 0..8 {
            let parameters = sample_transform(&spec, index, attempt).unwrap();
            assert_eq!(parameters.scale_x.to_bits(), parameters.scale_y.to_bits());
            assert_eq!(parameters, sample_transform(&spec, index, attempt).unwrap());
        }
    }
    assert_ne!(
        sample_transform(&spec, 0, 0).unwrap().scale_x,
        sample_transform(&spec, 1, 0).unwrap().scale_x
    );
    value["rules"]["preserveAspectRatio"] = json!(false);
    let unlocked: GenerationSpec = serde_json::from_value(value.clone()).unwrap();
    assert_ne!(
        sample_transform(&unlocked, 42, 0).unwrap().scale_x,
        sample_transform(&unlocked, 42, 0).unwrap().scale_y
    );
    value["rules"]["preserveAspectRatio"] = json!(true);
    value["rules"]["scaleY"]["max"] = json!(3.0);
    assert!(sample_transform(&serde_json::from_value(value).unwrap(), 0, 0).is_err());
}
fn varied() -> Value {
    let mut value = input();
    for (field, min, max) in [
        ("translateX", -10.0, 10.0),
        ("translateY", -20.0, 20.0),
        ("rotationDegrees", -180.0, 180.0),
        ("scaleX", 0.5, 2.0),
        ("scaleY", 0.75, 3.0),
    ] {
        value["rules"][field] = json!({"min":min,"max":max});
    }
    value
}

#[test]
fn identity_parameters_and_explicit_mirrors_are_preserved() {
    let spec: GenerationSpec = serde_json::from_value(input()).unwrap();
    assert_eq!(
        sample_transform(&spec, 0, 0).unwrap(),
        TransformParameters {
            translate_x: 0.0,
            translate_y: 0.0,
            rotation_degrees: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            mirror_x: false,
            mirror_y: false
        }
    );
    let mut value = input();
    value["rules"]["mirrorX"] = json!(true);
    value["rules"]["mirrorY"] = json!(true);
    let params = sample_transform(&serde_json::from_value(value).unwrap(), 99, 7).unwrap();
    assert!(params.mirror_x && params.mirror_y);
}

#[test]
fn samples_use_frozen_rule_ids_and_stay_in_ranges() {
    let spec: GenerationSpec = serde_json::from_value(varied()).unwrap();
    for index in [0, 100, u64::MAX] {
        for attempt in 0..8 {
            let p = sample_transform(&spec, index, attempt).unwrap();
            for (actual, id, min, max) in [
                (p.translate_x, 1, -10.0, 10.0),
                (p.translate_y, 2, -20.0, 20.0),
                (p.rotation_degrees, 3, -180.0, 180.0),
                (p.scale_x, 4, 0.5, 2.0),
                (p.scale_y, 5, 0.75, 3.0),
            ] {
                let expected = map_u32_to_range(
                    sample_u32(u64::MAX, index, id, attempt, 0).unwrap(),
                    min,
                    max,
                )
                .unwrap();
                assert_eq!(actual, expected);
                assert!((min..=max).contains(&actual));
            }
        }
    }
}

#[test]
fn batch_range_and_evaluation_order_do_not_change_parameters() {
    let spec: GenerationSpec = serde_json::from_value(varied()).unwrap();
    let mut value = varied();
    value["startIndex"] = json!("100");
    value["count"] = json!(100);
    let subset: GenerationSpec = serde_json::from_value(value).unwrap();
    for index in (100..200).rev() {
        let first = sample_transform(&spec, index, 0).unwrap();
        sample_transform(&spec, 1, 7).unwrap();
        assert_eq!(first, sample_transform(&subset, index, 0).unwrap());
    }
}

#[test]
fn changing_one_range_does_not_change_other_rules() {
    let spec: GenerationSpec = serde_json::from_value(varied()).unwrap();
    let mut value = varied();
    value["rules"]["translateX"] = json!({"min":50.0,"max":60.0});
    let changed: GenerationSpec = serde_json::from_value(value).unwrap();
    let a = sample_transform(&spec, 42, 3).unwrap();
    let b = sample_transform(&changed, 42, 3).unwrap();
    assert_ne!(a.translate_x, b.translate_x);
    assert_eq!(
        (a.translate_y, a.rotation_degrees, a.scale_x, a.scale_y),
        (b.translate_y, b.rotation_degrees, b.scale_x, b.scale_y)
    );
}

#[test]
fn invalid_attempts_and_rust_constructed_invalid_ranges_fail() {
    let mut spec: GenerationSpec = serde_json::from_value(input()).unwrap();
    assert!(sample_transform(&spec, 0, 8).is_err());
    assert!(sample_transform(&spec, 0, u8::MAX).is_err());
    spec.rules.scale_x.min = f64::NAN;
    assert!(sample_transform(&spec, 0, 0).is_err());
    spec.rules.scale_x.min = 2.0;
    assert!(sample_transform(&spec, 0, 0).is_err());
}
