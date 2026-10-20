use mosaicforge_core::{constraints::measure_coverage, template::import_svg};
use mosaicforge_core::{
    constraints::{fit_transform_to_frame, search_candidate},
    contract::GenerationSpec,
    distortion::distort_template,
    rules::sample_transform,
};
use serde_json::{Value, json};

fn configured(rules: Value) -> GenerationSpec {
    let mut value: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    for (key, rule) in rules.as_object().unwrap() {
        value["rules"][key] = rule.clone();
    }
    serde_json::from_value(value).unwrap()
}

#[test]
fn rules3_preserves_negative_space_and_requested_zoom_but_rejects_empty_or_uncovered_frames() {
    let source = import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="45" width="10" height="100"/><rect y="45" width="100" height="10"/></svg>"##).unwrap();
    let mut value = serde_json::to_value(configured(
        json!({"scaleX":{"min":4,"max":4},"scaleY":{"min":4,"max":4}}),
    ))
    .unwrap();
    value["templateHash"] = json!(source.hash);
    value["rulesVersion"] = json!("3");
    value["rules"]["preserveAspectRatio"] = json!(true);
    let spec: GenerationSpec = serde_json::from_value(value.clone()).unwrap();
    let accepted = search_candidate(&source, &spec, 0).unwrap();
    assert!(
        accepted.template.is_some(),
        "original negative space is valid: {:?}",
        accepted.reason
    );
    let p = accepted.parameters.unwrap();
    assert_eq!((p.scale_x, p.scale_y), (4.0, 4.0));
    value["rulesVersion"] = json!("2");
    assert!(
        search_candidate(&source, &serde_json::from_value(value.clone()).unwrap(), 0)
            .unwrap()
            .template
            .is_none()
    );
    value["rulesVersion"] = json!("3");
    value["rules"]["scaleX"] = json!({"min":1,"max":1});
    value["rules"]["scaleY"] = json!({"min":1,"max":1});
    value["rules"]["rotationDegrees"] = json!({"min":45,"max":45});
    let uncovered =
        search_candidate(&source, &serde_json::from_value(value.clone()).unwrap(), 0).unwrap();
    assert_eq!(uncovered.reason, Some("frame_exceeds_scale_limits"));
    let empty = import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="none"/></svg>"##).unwrap();
    value["templateHash"] = json!(empty.hash);
    value["rules"]["rotationDegrees"] = json!({"min":0,"max":0});
    let decision = search_candidate(&empty, &serde_json::from_value(value).unwrap(), 0).unwrap();
    assert_eq!(decision.reason, Some("empty_viewport"));
}

#[test]
fn coverage_measures_visible_paint_not_bounding_boxes() {
    let full=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="#123456"/></svg>"##).unwrap();
    let stats = measure_coverage(&full).unwrap();
    assert_eq!(stats.painted_pixels, 256 * 256);
    assert_eq!(stats.occupied_tiles, 256);
    assert_eq!(stats.empty_corner_tiles, 0);
    let empty=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="none"/></svg>"##).unwrap();
    assert_eq!(measure_coverage(&empty).unwrap().painted_pixels, 0);
    let sparse=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><path d="M0 0L100 100" fill="none" stroke="red"/></svg>"##).unwrap();
    assert!(measure_coverage(&sparse).unwrap().occupied_tiles < 64);
    let clipped=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><clipPath id="c"><rect x="25" y="25" width="50" height="50"/></clipPath></defs><rect width="100" height="100" clip-path="url(#c)"/></svg>"##).unwrap();
    let stats = measure_coverage(&clipped).unwrap();
    assert_eq!(stats.empty_corner_tiles, 16);
    assert_eq!(stats.occupied_tiles, 64);
}

#[test]
fn final_cover_must_not_accept_an_empty_visible_output() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="49" height="100"/><rect x="51" width="49" height="100"/></g></svg>"##).unwrap();
    let mut spec = configured(json!({}));
    spec.template_hash = source.hash.parse().unwrap();
    for (width, height) in [(1, 4096), (32, 4096)] {
        spec.output.width = width.try_into().unwrap();
        spec.output.height = height.try_into().unwrap();
        let decision = search_candidate(&source, &spec, 0).unwrap();
        assert!(
            decision.template.is_none(),
            "final narrow cover is empty despite a mostly occupied source crop"
        );
        assert_eq!(decision.reason, Some("empty_viewport"));
    }
}

#[test]
fn coverage_respects_group_opacity_and_revalidates_template() {
    let invisible=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g opacity="0"><rect width="100" height="100"/></g></svg>"##).unwrap();
    assert_eq!(measure_coverage(&invisible).unwrap().painted_pixels, 0);
    let mut forged = invisible;
    forged.normalized_svg = "<svg><script/></svg>".into();
    assert!(measure_coverage(&forged).is_err());
}

#[test]
fn rotated_canvas_is_zoomed_to_cover_all_corners_within_scale_limits() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="10 20 100 100"><g><rect x="10" y="20" width="100" height="100"/></g></svg>"##).unwrap();
    let s = configured(
        json!({"scaleX":{"min":1,"max":2},"scaleY":{"min":1,"max":2},"rotationDegrees":{"min":45,"max":45}}),
    );
    let mut p = sample_transform(&s, 0, 0).unwrap();
    p.scale_x = 1.0;
    p.scale_y = 1.0;
    assert!(fit_transform_to_frame(&source, &s, &mut p).unwrap());
    assert!(
        (p.scale_x - 2f64.sqrt()).abs() < 1e-12,
        "scale={}",
        p.scale_x
    );
    assert_eq!(p.scale_x, p.scale_y);
    let restricted = configured(json!({"rotationDegrees":{"min":45,"max":45}}));
    let mut p = sample_transform(&restricted, 0, 0).unwrap();
    assert!(!fit_transform_to_frame(&source, &restricted, &mut p).unwrap());
    assert_eq!(p.scale_x, 1.0);
}

#[test]
fn distortion_is_bounded_repeatable_and_preserves_clip_geometry() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><clipPath id="clip"><path d="M0 0L100 0L100 100L0 100Z"/></clipPath></defs><g clip-path="url(#clip)"><path d="M10 20L30 40Q50 60 70 80C10 20 30 40 50 60Z"/></g></svg>"##).unwrap();
    let s = configured(json!({"distortion":3}));
    let result = distort_template(&source, &s, 17, 0).unwrap();
    assert_ne!(result.hash, source.hash);
    assert!(
        result
            .normalized_svg
            .contains("d=\"M0 0L100 0L100 100L0 100Z\"")
    );
    fn artwork(svg: &str) -> Vec<(f64, f64)> {
        let mut r = quick_xml::Reader::from_str(svg);
        let mut paths = Vec::new();
        while let Ok(event) = r.read_event() {
            match event {
                quick_xml::events::Event::Start(e) if e.name().as_ref() == "path" => {
                    let d = e
                        .attributes()
                        .map(Result::unwrap)
                        .find(|a| a.key.as_ref() == "d")
                        .unwrap()
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .unwrap()
                        .into_owned();
                    paths.push(d);
                }
                quick_xml::events::Event::Eof => break,
                _ => (),
            }
        }
        let mut pts = Vec::new();
        for segment in svgtypes::SimplifyingPathParser::from(paths.last().unwrap().as_str()) {
            use svgtypes::SimplePathSegment::*;
            match segment.unwrap() {
                MoveTo { x, y } | LineTo { x, y } => pts.push((x, y)),
                Quadratic { x1, y1, x, y } => pts.extend([(x1, y1), (x, y)]),
                CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                } => pts.extend([(x1, y1), (x2, y2), (x, y)]),
                ClosePath => (),
            }
        }
        pts
    }
    for (a, b) in artwork(&source.normalized_svg)
        .iter()
        .zip(artwork(&result.normalized_svg))
    {
        assert!((a.0 - b.0).hypot(a.1 - b.1) <= 3.0 + 1e-10);
    }
    let identity = configured(json!({}));
    assert_eq!(
        distort_template(&source, &identity, 17, 0).unwrap().hash,
        source.hash
    );
    assert_eq!(
        distort_template(&source, &s, 17, 0).unwrap().hash,
        result.hash
    );
    assert!(distort_template(&source, &s, 17, 8).is_err());
}

#[test]
fn rejection_preserves_next_index_and_is_limited_to_eight_attempts() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100"/></g></svg>"##).unwrap();
    let full = configured(
        json!({"rotationDegrees":{"min":-180,"max":180},"scaleX":{"min":1,"max":2},"scaleY":{"min":1,"max":2}}),
    );
    let expected = search_candidate(&source, &full, 117).unwrap();
    assert!(expected.template.is_some());
    assert!((1..=8).contains(&expected.attempts));
    let invisible = configured(json!({"opacity":{"min":0,"max":0}}));
    let rejected = search_candidate(&source, &invisible, 116).unwrap();
    assert!(rejected.template.is_none());
    assert_eq!(rejected.attempts, 8);
    assert_eq!(rejected.reason, Some("empty_viewport"));
    let mut subset = full.clone();
    subset.start_index = "117".parse().unwrap();
    subset.count = 1.try_into().unwrap();
    let actual = search_candidate(&source, &subset, 117).unwrap();
    assert_eq!(
        expected.template.unwrap().hash,
        actual.template.unwrap().hash
    );
    let sparse=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><circle cx="50" cy="50" r="2"/></g></svg>"##).unwrap();
    assert!(
        search_candidate(&sparse, &configured(json!({})), 0)
            .unwrap()
            .template
            .is_none()
    );
}

#[test]
fn framing_keeps_mirrored_offset_corners_inside_source_and_checks_parameters() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="10 20 100 80"><g><rect x="10" y="20" width="100" height="80"/></g></svg>"##).unwrap();
    let s = configured(
        json!({"crop":{"x":30,"y":35,"width":50,"height":40},"rotationDegrees":{"min":-180,"max":180},"translateX":{"min":-5,"max":5},"translateY":{"min":-5,"max":5},"scaleX":{"min":1,"max":4},"scaleY":{"min":1,"max":4},"mirrorX":true,"mirrorY":true}),
    );
    for index in 0..20 {
        let mut p = sample_transform(&s, index, 0).unwrap();
        if !fit_transform_to_frame(&source, &s, &mut p).unwrap() {
            continue;
        }
        assert!((1.0..=4.0).contains(&p.scale_x) && (1.0..=4.0).contains(&p.scale_y));
        let [a, b, c, d, e, f] =
            mosaicforge_core::generate::geometric_matrix(&p, source.view_box).unwrap();
        let det = a * d - b * c;
        for x in [30.0, 80.0] {
            for y in [35.0, 75.0] {
                let ix = (d * (x - e) - c * (y - f)) / det;
                let iy = (-b * (x - e) + a * (y - f)) / det;
                assert!((10.0 - 1e-10..=110.0 + 1e-10).contains(&ix));
                assert!((20.0 - 1e-10..=100.0 + 1e-10).contains(&iy));
            }
        }
    }
    let mut invalid = sample_transform(&s, 0, 0).unwrap();
    invalid.scale_x = 0.5;
    assert!(fit_transform_to_frame(&source, &s, &mut invalid).is_err());
}

#[test]
fn invalid_candidates_are_retried_but_bad_configuration_is_an_error() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100"/></g></svg>"##).unwrap();
    for rules in [
        json!({"translateX":{"min":1e20,"max":1e20}}),
        json!({"distortion":1e20}),
    ] {
        let rejected = search_candidate(&source, &configured(rules), 0).unwrap();
        assert!(rejected.template.is_none());
        assert_eq!(rejected.attempts, 8);
        assert_eq!(rejected.reason, Some("invalid_geometry"));
    }
    let bad = configured(json!({"crop":{"x":90,"y":0,"width":20,"height":100}}));
    assert!(search_candidate(&source, &bad, 0).is_err());
}

#[test]
fn search_can_accept_a_later_deterministic_attempt() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100"/></g></svg>"##).unwrap();
    let s = configured(
        json!({"rotationDegrees":{"min":-180,"max":180},"scaleX":{"min":1,"max":1.3},"scaleY":{"min":1,"max":1.3}}),
    );
    let mut accepted_after_retry = false;
    for index in 0..20 {
        let decision = search_candidate(&source, &s, index).unwrap();
        if let Some(template) = decision.template
            && decision.attempts > 1
        {
            accepted_after_retry = true;
            let repeated = search_candidate(&source, &s, index).unwrap();
            assert_eq!(decision.attempts, repeated.attempts);
            assert_eq!(template.hash, repeated.template.unwrap().hash);
            break;
        }
    }
    assert!(accepted_after_retry);
}

#[test]
fn coverage_uses_cropped_viewbox_not_original_viewport_proportions() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><g><rect width="100" height="50"/></g></svg>"##).unwrap();
    let s = configured(json!({"crop":{"x":0,"y":0,"width":100,"height":50}}));
    let decision = search_candidate(&source, &s, 0).unwrap();
    assert!(decision.template.is_some(), "{:?}", decision.reason);
    let coverage = decision.coverage.unwrap();
    assert_eq!(coverage.occupied_tiles, 256);
    assert_eq!(coverage.empty_corner_tiles, 0);
    assert_eq!(decision.template.unwrap().width, 100.0);
}

#[test]
fn exact_half_and_quarter_turns_fit_without_exceeding_scale_maximum() {
    let source=import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100"/></g></svg>"##).unwrap();
    for angle in [180, 270] {
        let s = configured(json!({"rotationDegrees":{"min":angle,"max":angle}}));
        let decision = search_candidate(&source, &s, 0).unwrap();
        assert!(
            decision.template.is_some(),
            "angle={angle}, reason={:?}",
            decision.reason
        );
        let p = decision.parameters.unwrap();
        assert_eq!(p.scale_x, 1.0);
        assert_eq!(p.scale_y, 1.0);
    }
    let real_excess = configured(json!({"rotationDegrees":{"min":1,"max":1}}));
    let decision = search_candidate(&source, &real_excess, 0).unwrap();
    assert!(decision.template.is_none());
    assert_eq!(decision.reason, Some("frame_exceeds_scale_limits"));
}
