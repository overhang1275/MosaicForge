use mosaicforge_core::{
    contract::GenerationSpec,
    generate::{apply_geometric_transform, geometric_matrix},
    rules::{TransformParameters, sample_transform},
    template::import_svg,
};
use serde_json::{Value, json};

fn identity() -> TransformParameters {
    TransformParameters {
        translate_x: 0.0,
        translate_y: 0.0,
        rotation_degrees: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        mirror_x: false,
        mirror_y: false,
    }
}

fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
}

#[test]
fn identity_preserves_geometry() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/basic.svg").as_slice(),
        include_bytes!("../../../tests/fixtures/layers.svg").as_slice(),
        include_bytes!("../../../tests/fixtures/clip.svg").as_slice(),
    ] {
        let original = import_svg(bytes).unwrap();
        let result = apply_geometric_transform(&original, &identity()).unwrap();
        assert_eq!(result.normalized_svg, original.normalized_svg);
        assert_eq!(result.hash, original.hash);
        assert_eq!(result.layer_ids, original.layer_ids);
        assert_eq!(result.view_box, original.view_box);
    }
    let mut full_turn = identity();
    full_turn.rotation_degrees = 360.0;
    assert_eq!(
        geometric_matrix(&full_turn, [0.0, 0.0, 100.0, 80.0]).unwrap(),
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
    );
}

#[test]
fn transform_order_is_fixed_around_offset_viewbox_center() {
    let p = TransformParameters {
        translate_x: 5.0,
        translate_y: -7.0,
        rotation_degrees: 90.0,
        scale_x: 2.0,
        scale_y: 3.0,
        mirror_x: true,
        mirror_y: false,
    };
    let [a, b, c, d, e, f] = geometric_matrix(&p, [10.0, 20.0, 100.0, 80.0]).unwrap();
    near(a * 70.0 + c * 60.0 + e, 65.0);
    near(b * 70.0 + d * 60.0 + f, 33.0);
    near(a * 60.0 + c * 60.0 + e, 65.0);
    near(b * 60.0 + d * 60.0 + f, 53.0);
}

#[test]
fn root_transform_is_prepended_without_losing_layers_or_clips() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="-10 -20 100 80" transform="translate(3 4) scale(2)" clip-path="url(#clip)"><defs><clipPath id="clip"><rect x="-10" y="-20" width="100" height="80"/></clipPath></defs><g id="layer" transform="translate(2 1)"><path id="mark" d="M0 0 L10 0 L0 10 Z" fill="#123456"/></g></svg>"##).unwrap();
    let mut p = identity();
    p.translate_x = 7.0;
    p.translate_y = -3.0;
    let result = apply_geometric_transform(&source, &p).unwrap();
    assert_eq!(result.layer_ids, ["layer"]);
    assert_eq!(result.view_box, source.view_box);
    assert_eq!(result.segment_count, source.segment_count);
    assert!(result.normalized_svg.contains("clip-path=\"url(#clip)\""));
    let tree = usvg::Tree::from_str(&result.normalized_svg, &usvg::Options::default()).unwrap();
    let transform = tree.node_by_id("mark").unwrap().abs_transform();
    near(f64::from(transform.tx), 24.0);
    near(f64::from(transform.ty), 23.0);
    assert_eq!(tree.clip_paths().len(), 1);
    assert_eq!(
        import_svg(result.normalized_svg.as_bytes()).unwrap().hash,
        result.hash
    );
}

#[test]
fn geometric_output_is_repeatable_across_batch_ranges() {
    let source = import_svg(include_bytes!("../../../tests/fixtures/basic.svg")).unwrap();
    let mut value: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    value["rules"]["translateX"] = json!({"min":-5.0,"max":5.0});
    value["rules"]["rotationDegrees"] = json!({"min":-20.0,"max":20.0});
    let full: GenerationSpec = serde_json::from_value(value.clone()).unwrap();
    value["startIndex"] = json!("100");
    value["count"] = json!(100);
    let subset: GenerationSpec = serde_json::from_value(value).unwrap();
    let expected =
        apply_geometric_transform(&source, &sample_transform(&full, 117, 0).unwrap()).unwrap();
    assert_ne!(expected.hash, source.hash);
    for _ in 0..3 {
        let actual =
            apply_geometric_transform(&source, &sample_transform(&subset, 117, 0).unwrap())
                .unwrap();
        assert_eq!(actual.normalized_svg, expected.normalized_svg);
        assert_eq!(actual.hash, expected.hash);
    }
}

#[test]
fn invalid_parameters_and_tampered_svg_are_rejected() {
    let source = import_svg(include_bytes!("../../../tests/fixtures/basic.svg")).unwrap();
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut p = identity();
        p.scale_x = invalid;
        assert!(apply_geometric_transform(&source, &p).is_err());
    }
    let mut p = identity();
    p.translate_x = f64::NAN;
    assert!(geometric_matrix(&p, source.view_box).is_err());
    assert!(geometric_matrix(&identity(), [0.0, 0.0, 0.0, 80.0]).is_err());
    let mut forged = source;
    forged.normalized_svg = "<svg><script/></svg>".into();
    assert!(apply_geometric_transform(&forged, &identity()).is_err());
}

#[test]
fn output_respects_importer_transform_limits() {
    let source=import_svg(br#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80"><g transform="scale(1000)"><rect width="1" height="1"/></g></svg>"#).unwrap();
    let mut p = identity();
    p.scale_x = 10000.0;
    p.scale_y = 10000.0;
    assert!(apply_geometric_transform(&source, &p).is_err());
    let mut p = identity();
    p.scale_x = 1e-10;
    assert!(apply_geometric_transform(&source, &p).is_err());
}

#[test]
fn vertical_mirror_and_negative_rotation_follow_the_same_order() {
    let mut p = identity();
    p.mirror_y = true;
    p.rotation_degrees = -90.0;
    let [a, b, c, d, e, f] = geometric_matrix(&p, [0.0, 0.0, 100.0, 80.0]).unwrap();
    near(a * 10.0 + c * 20.0 + e, 70.0);
    near(b * 10.0 + d * 20.0 + f, 80.0);
}
