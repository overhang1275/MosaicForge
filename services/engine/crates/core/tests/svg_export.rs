use mosaicforge_core::{contract::OutputSpec, export::export_svg, template::import_svg};
use serde_json::json;

fn output(background: &str) -> OutputSpec {
    serde_json::from_value(json!({"format":"svg","width":64,"height":32,"background":background}))
        .unwrap()
}

const SVG:&[u8]=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="-10 -20 100 80" transform="translate(3 4)" opacity="0.5" clip-path="url(#clip)" fill-opacity="0.3" stroke="red"><defs><clipPath id="clip"><rect x="-10" y="-20" width="50" height="80"/></clipPath></defs><g id="layer"><path id="mark" d="M-10 -20L90 -20L90 60Z" fill="#123456"/></g></svg>"##;

#[test]
fn exported_svg_is_safe_vectorial_sized_and_repeatable() {
    let bytes = export_svg(SVG, &output("#aAbBcC")).unwrap();
    assert_eq!(bytes, export_svg(SVG, &output("#aAbBcC")).unwrap());
    let result = import_svg(&bytes).unwrap();
    assert_eq!((result.width, result.height), (64.0, 32.0));
    assert_eq!(result.view_box, [-10.0, -20.0, 100.0, 80.0]);
    for fragment in [
        "preserveAspectRatio=\"xMidYMid slice\"",
        "id=\"layer\"",
        "id=\"mark\"",
        "id=\"clip\"",
        "<path",
        "#aabbcc",
    ] {
        assert!(
            result.normalized_svg.contains(fragment),
            "missing {fragment}"
        );
    }
    assert!(!result.normalized_svg.contains("<image"));
}

#[test]
fn transparent_export_keeps_existing_root_effects_without_background_or_wrapper() {
    let source = import_svg(SVG).unwrap();
    let result = import_svg(&export_svg(SVG, &output("transparent")).unwrap()).unwrap();
    assert_eq!(result.segment_count, source.segment_count);
    assert_eq!(result.layer_ids, source.layer_ids);
    assert!(result.normalized_svg.contains("opacity=\"0.5\""));
    assert!(
        result
            .normalized_svg
            .contains("transform=\"translate(3 4)\"")
    );
}

#[test]
fn export_rejects_unsafe_inputs_wrong_format_and_resource_limit_overflow() {
    assert!(export_svg(b"<svg><script/></svg>", &output("transparent")).is_err());
    let mut png = output("transparent");
    png.format = "png".parse().unwrap();
    assert!(export_svg(SVG, &png).is_err());
    let mut oversized = output("transparent");
    oversized.width = 4097.try_into().unwrap();
    assert!(export_svg(SVG, &oversized).is_err());
    let deepest = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\" transform=\"translate(1 0)\">{}<rect width=\"100\" height=\"100\"/>{}</svg>",
        "<g>".repeat(62),
        "</g>".repeat(62)
    );
    assert!(import_svg(deepest.as_bytes()).is_ok());
    assert!(export_svg(deepest.as_bytes(), &output("transparent")).is_ok());
    assert!(export_svg(deepest.as_bytes(), &output("#ffffff")).is_err());
}
