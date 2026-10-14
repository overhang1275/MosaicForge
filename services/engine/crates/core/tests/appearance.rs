use mosaicforge_core::{
    appearance::apply_appearance, contract::GenerationSpec, template::import_svg,
};
use serde_json::{Value, json};

fn spec() -> GenerationSpec {
    serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap()
}
fn configured(rules: Value) -> GenerationSpec {
    let mut value = serde_json::to_value(spec()).unwrap();
    for (key, value_rule) in rules.as_object().unwrap() {
        value["rules"][key] = value_rule.clone();
    }
    serde_json::from_value(value).unwrap()
}
const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="10 20 100 80" fill="#123456"><defs><clipPath id="clip"><rect width="100" height="80"/></clipPath></defs><rect id="outside" width="3" height="3" fill="#abcdef"/><g id="a" opacity="0.4" clip-path="url(#clip)"><path id="p" d="M10 20L30 20L30 40Z" fill="none" stroke="red"/></g><g id="b"><rect width="10" height="10" fill="#abcdef"/></g><g id="c"><circle r="4"/></g></svg>"##;

#[test]
fn identity_preserves_svg_and_hash() {
    let source = import_svg(SVG).unwrap();
    let result = apply_appearance(&source, &spec(), 17, 0).unwrap();
    assert_eq!(source.hash, result.hash);
    assert_eq!(source.normalized_svg, result.normalized_svg);
}

#[test]
fn palette_and_opacity_obey_bounds_and_layer_scope() {
    let source = import_svg(SVG).unwrap();
    let s = configured(json!({"palette":["#00ff00"],"opacity":{"min":0.5,"max":0.5}}));
    let result = apply_appearance(&source, &s, 17, 0).unwrap();
    assert!(result.normalized_svg.contains("id=\"outside\""));
    assert!(
        result
            .normalized_svg
            .contains("fill=\"#abcdef\" height=\"3\" id=\"outside\"")
    );
    assert!(result.normalized_svg.contains("opacity=\"0.2\""));
    assert!(result.normalized_svg.contains("opacity=\"0.5\""));
    assert!(result.normalized_svg.contains("fill=\"none\""));
    assert!(result.normalized_svg.contains("stroke=\"#00ff00\""));
    assert_eq!(result.layer_ids, source.layer_ids);
    assert_eq!(result.segment_count, source.segment_count);
    let tree = usvg::Tree::from_str(&result.normalized_svg, &usvg::Options::default()).unwrap();
    assert_eq!(tree.clip_paths().len(), 1);
}

#[test]
fn layer_order_is_reproducible_and_preserves_nonlayer_slots() {
    let source = import_svg(SVG).unwrap();
    let s = configured(
        json!({"shuffleLayers":true,"palette":["#ff0000","#0000ff"],"opacity":{"min":0.2,"max":0.8}}),
    );
    let mut orders = std::collections::BTreeSet::new();
    for index in 0..20 {
        let result = apply_appearance(&source, &s, index, 0).unwrap();
        orders.insert(result.layer_ids.clone());
        let mut ids = result.layer_ids.clone();
        ids.sort();
        assert_eq!(ids, ["a", "b", "c"]);
        assert!(
            result.normalized_svg.find("id=\"outside\"").unwrap()
                < result.normalized_svg.find("<g").unwrap()
        );
        let mut subset = s.clone();
        subset.start_index = "10".parse().unwrap();
        subset.count = 20.try_into().unwrap();
        assert_eq!(
            result.hash,
            apply_appearance(&source, &subset, index, 0).unwrap().hash
        );
    }
    assert!(orders.len() > 1);
}

#[test]
fn crop_stays_inside_offset_viewbox() {
    let source = import_svg(SVG).unwrap();
    let s = configured(json!({"crop":{"x":15,"y":25,"width":30,"height":40}}));
    let result = apply_appearance(&source, &s, 0, 0).unwrap();
    assert_eq!(result.view_box, [15.0, 25.0, 30.0, 40.0]);
    assert_eq!(result.segment_count, source.segment_count);
    for crop in [
        json!({"x":9,"y":25,"width":30,"height":40}),
        json!({"x":15,"y":25,"width":100,"height":40}),
        json!({"x":15,"y":25,"width":30,"height":80}),
    ] {
        assert!(apply_appearance(&source, &configured(json!({"crop":crop})), 0, 0).is_err());
    }
}

#[test]
fn invalid_inputs_and_attempts_are_rejected_even_for_identity() {
    let source = import_svg(SVG).unwrap();
    assert!(apply_appearance(&source, &spec(), 0, 8).is_err());
    let mut invalid = spec();
    invalid.rules.opacity.min = f64::NAN;
    assert!(apply_appearance(&source, &invalid, 0, 0).is_err());
    let mut forged = source;
    forged.normalized_svg = "<svg><script/></svg>".into();
    assert!(apply_appearance(&forged, &spec(), 0, 0).is_err());
}

#[test]
fn palette_preserves_paint_alpha_and_inherited_none() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" fill="none" stroke="rgba(255,0,0,0.5)"><g><rect id="mark" width="10" height="10"/></g></svg>"##).unwrap();
    let result =
        apply_appearance(&source, &configured(json!({"palette":["#00ff00"]})), 0, 0).unwrap();
    assert!(result.normalized_svg.contains("<g fill=\"none\""));
    let tree = usvg::Tree::from_str(&result.normalized_svg, &usvg::Options::default()).unwrap();
    let usvg::Node::Path(path) = tree.node_by_id("mark").unwrap() else {
        panic!("path expected")
    };
    assert!(path.fill().is_none());
    let stroke = path.stroke().unwrap();
    assert!(matches!(stroke.paint(),usvg::Paint::Color(c) if c.green==255 && c.red==0));
    assert!((stroke.opacity().get() - 0.5).abs() < 0.01);
}

#[test]
fn shuffle_does_not_change_layer_appearance_or_nested_opacity() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g id="a" opacity="0.4"><g opacity="0.3"><rect width="10" height="10" fill="red"/></g></g><g id="b"><rect width="10" height="10" fill="#ff0000"/></g></svg>"##).unwrap();
    let s = configured(json!({"palette":["#ff3300","#00aaff"],"opacity":{"min":0.2,"max":0.8}}));
    let mut shuffled = s.clone();
    shuffled.rules.shuffle_layers = true;
    for index in 0..20 {
        let before = apply_appearance(&source, &s, index, 0).unwrap();
        let after = apply_appearance(&source, &shuffled, index, 0).unwrap();
        fn layers(svg: &str) -> std::collections::BTreeMap<String, String> {
            let mut result = std::collections::BTreeMap::new();
            for id in ["a", "b"] {
                let marker = format!("id=\"{id}\"");
                let position = svg.find(&marker).unwrap();
                let start = svg[..position].rfind("<g").unwrap();
                // a contains a nested group, b does not.
                let end = svg[position..]
                    .find(if id == "a" { "</g></g>" } else { "</g>" })
                    .unwrap()
                    + position;
                result.insert(id.into(), svg[start..end].into());
            }
            result
        }
        assert_eq!(
            layers(&before.normalized_svg),
            layers(&after.normalized_svg)
        );
        assert!(after.normalized_svg.contains("<g opacity=\"0.3\""));
        let colors: Vec<_> = after
            .normalized_svg
            .split("<rect fill=\"")
            .skip(1)
            .map(|s| s.split('"').next().unwrap())
            .collect();
        assert_eq!(colors.len(), 2);
        assert_eq!(colors[0], colors[1]);
    }
}

#[test]
fn appearance_without_layers_preserves_unscoped_shapes() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10" fill="red"/></svg>"##).unwrap();
    let s =
        configured(json!({"palette":["#00ff00"],"opacity":{"min":0,"max":0},"shuffleLayers":true}));
    assert_eq!(
        source.hash,
        apply_appearance(&source, &s, 0, 0).unwrap().hash
    );
}
