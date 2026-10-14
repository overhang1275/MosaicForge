use mosaicforge_core::{
    configuration_hash,
    contract::{GenerationSpec, VariantResult},
    validate_result, validate_spec,
};
use serde_json::{Value, json};

fn identity() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap()
}

fn valid(value: Value) -> bool {
    serde_json::from_value::<GenerationSpec>(value).is_ok_and(|spec| validate_spec(&spec).is_ok())
}

#[test]
fn contract_rejects_bad_bounds_and_versions() {
    assert!(valid(identity()));
    for (pointer, values) in [
        (
            "/seed",
            vec![json!("18446744073709551616"), json!("01"), json!(1)],
        ),
        ("/startIndex", vec![json!("18446744073709551615")]),
        ("/count", vec![json!(0), json!(10001)]),
        ("/output/width", vec![json!(0), json!(4097)]),
        ("/output/height", vec![json!(0), json!(4097)]),
        ("/schemaVersion", vec![json!("2")]),
        ("/engineVersion", vec![json!("0.2.0")]),
        ("/rulesVersion", vec![json!("2")]),
        ("/rules/scaleX/min", vec![json!(0), json!(-1)]),
        ("/rules/opacity/min", vec![json!(-0.1)]),
        ("/rules/opacity/max", vec![json!(1.1)]),
        ("/rules/translateX/min", vec![json!(1)]),
        ("/rules/distortion", vec![json!(-1)]),
    ] {
        for value in values {
            let mut input = identity();
            *input.pointer_mut(pointer).unwrap() = value;
            assert!(!valid(input), "unexpected acceptance: {pointer}");
        }
    }
}

#[test]
fn decimal_u64_roundtrips_without_precision_loss() {
    let mut value = identity();
    value["startIndex"] = json!("18446744073709551615");
    value["count"] = json!(1);
    let spec: GenerationSpec = serde_json::from_value(value).unwrap();
    validate_spec(&spec).unwrap();
    let output = serde_json::to_value(spec).unwrap();
    assert_eq!(output["seed"], "18446744073709551615");
    assert_eq!(output["startIndex"], "18446744073709551615");
}

#[test]
fn contracts_reject_unknown_fields_and_bad_formats() {
    for (pointer, value) in [
        ("/templateHash", json!("bad")),
        ("/output/background", json!("url(file:///secret)")),
        ("/output/format", json!("jpg")),
    ] {
        let mut input = identity();
        *input.pointer_mut(pointer).unwrap() = value;
        assert!(!valid(input));
    }
    let mut input = identity();
    input["secret"] = json!(true);
    assert!(!valid(input));
    let mut input = identity();
    input["rules"]["extra"] = json!(true);
    assert!(!valid(input));
    let mut input = identity();
    input["rules"]["palette"] = json!(["#bad"]);
    assert!(!valid(input));
    assert!(serde_json::from_str::<GenerationSpec>("{\"seed\": NaN}").is_err());
    assert!(serde_json::from_str::<GenerationSpec>("{\"seed\": Infinity}").is_err());
}

#[test]
fn configuration_hash_ignores_batch_range_but_tracks_output_and_rules() {
    let original: GenerationSpec = serde_json::from_value(identity()).unwrap();
    let hash = configuration_hash(&original).unwrap();
    assert_eq!(hash.len(), 64);
    let mut changed = identity();
    changed["startIndex"] = json!("100");
    changed["count"] = json!(100);
    let changed: GenerationSpec = serde_json::from_value(changed).unwrap();
    assert_eq!(hash, configuration_hash(&changed).unwrap());
    for (pointer, value) in [
        ("/output/width", json!(256)),
        ("/rules/mirrorX", json!(true)),
    ] {
        let mut changed = identity();
        *changed.pointer_mut(pointer).unwrap() = value;
        assert_ne!(
            hash,
            configuration_hash(&serde_json::from_value(changed).unwrap()).unwrap()
        );
    }
}

#[test]
fn result_roundtrips_and_rejects_unknown_backend() {
    let result = json!({"schemaVersion":"1", "index":"0", "seed":"18446744073709551615", "templateHash":"0".repeat(64), "configurationHash":"1".repeat(64), "artifactHash":"2".repeat(64), "engineVersion":"0.1.0", "rulesVersion":"1", "backend":"cpu", "device":"cpu", "status":"accepted"});
    let decoded: VariantResult = serde_json::from_value(result.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), result);
    let mut bad = result;
    bad["backend"] = json!("cuda");
    assert!(serde_json::from_value::<VariantResult>(bad).is_err());
}

#[test]
fn rules_reject_nonfinite_values_even_when_constructed_in_rust() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut spec: GenerationSpec = serde_json::from_value(identity()).unwrap();
        spec.rules.translate_x.min = value;
        assert!(validate_spec(&spec).is_err());
        assert!(configuration_hash(&spec).is_err());
        let mut spec: GenerationSpec = serde_json::from_value(identity()).unwrap();
        spec.rules.distortion = value;
        assert!(validate_spec(&spec).is_err());
    }
}

#[test]
fn crop_palette_and_nested_objects_obey_limits() {
    for crop in [
        json!({"x":-1,"y":0,"width":1,"height":1}),
        json!({"x":0,"y":0,"width":0,"height":1}),
        json!({"x":1e308,"y":0,"width":1e308,"height":1}),
        json!({"x":0,"y":0,"width":1,"height":1,"extra":true}),
    ] {
        let mut input = identity();
        input["rules"]["crop"] = crop;
        assert!(!valid(input));
    }
    let mut input = identity();
    input["rules"]["crop"] = json!({"x":0,"y":0,"width":1,"height":1});
    input["rules"]["palette"] = json!(["#FFFFFF"]);
    assert!(valid(input));
    let mut input = identity();
    input["rules"]["palette"] = json!(vec!["#abcdef"; 257]);
    assert!(!valid(input));
    let mut input = identity();
    input["rules"]["palette"] = json!(Vec::<String>::new());
    assert!(valid(input));
    let mut input = identity();
    input["output"]["extra"] = json!(true);
    assert!(!valid(input));
    let mut input = identity();
    input["rules"]["scaleX"]["extra"] = json!(true);
    assert!(!valid(input));
}

#[test]
fn hash_is_stable_and_covers_reproduction_metadata() {
    let spec: GenerationSpec = serde_json::from_value(identity()).unwrap();
    assert_eq!(
        configuration_hash(&spec).unwrap(),
        configuration_hash(&spec).unwrap()
    );
    for (pointer, value) in [
        ("/seed", json!("1")),
        ("/templateHash", json!("a".repeat(64))),
    ] {
        let mut changed = identity();
        *changed.pointer_mut(pointer).unwrap() = value;
        let changed: GenerationSpec = serde_json::from_value(changed).unwrap();
        assert_ne!(
            configuration_hash(&spec).unwrap(),
            configuration_hash(&changed).unwrap()
        );
    }
    let error = mosaicforge_core::EngineError::InvalidSpec("input exceeds limits");
    assert_eq!(error.code(), "invalid_spec");
    assert_eq!(error.to_string(), "input exceeds limits");
}

fn result_fixture() -> Value {
    json!({"schemaVersion":"1", "index":"18446744073709551615", "seed":"18446744073709551615", "templateHash":"0".repeat(64), "configurationHash":"1".repeat(64), "artifactHash":"2".repeat(64), "engineVersion":"0.1.0", "rulesVersion":"1", "backend":"cpu", "device":"cpu", "status":"accepted"})
}

#[test]
fn result_rejects_u64_overflow_and_excessive_metadata() {
    let valid: VariantResult = serde_json::from_value(result_fixture()).unwrap();
    validate_result(&valid).unwrap();
    for (field, value) in [
        ("index", json!("18446744073709551616")),
        ("seed", json!("18446744073709551616")),
        ("device", json!("x".repeat(257))),
        ("reason", json!("x".repeat(257))),
    ] {
        let mut value_fixture = result_fixture();
        value_fixture[field] = value;
        let rejected = match serde_json::from_value::<VariantResult>(value_fixture) {
            Ok(result) => validate_result(&result).is_err(),
            Err(_) => true,
        };
        assert!(rejected, "unexpected acceptance: {field}");
    }
}

#[test]
fn optional_nulls_are_explicitly_supported_and_normalized() {
    let mut input = identity();
    input["rules"]["crop"] = Value::Null;
    assert!(valid(input.clone()));
    let spec: GenerationSpec = serde_json::from_value(input).unwrap();
    let original: GenerationSpec = serde_json::from_value(identity()).unwrap();
    assert_eq!(
        configuration_hash(&spec).unwrap(),
        configuration_hash(&original).unwrap()
    );
    let mut input = result_fixture();
    input["artifactHash"] = Value::Null;
    input["reason"] = Value::Null;
    let result: VariantResult = serde_json::from_value(input).unwrap();
    validate_result(&result).unwrap();
    assert!(
        serde_json::to_value(result)
            .unwrap()
            .get("artifactHash")
            .is_none()
    );
}
