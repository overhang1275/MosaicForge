use mosaicforge_core::contract::OutputSpec;
use mosaicforge_cpu::render::{MAX_RENDER_BUFFER_BYTES, plan_png_buffers};
use mosaicforge_cpu::render::{plan_png_render, render_png, render_png_with_budget};
use serde_json::json;

const SVG:&[u8]=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect x="25" y="25" width="50" height="50" fill="#ff0000"/></svg>"##;

fn output(width: u64, height: u64, background: &str) -> OutputSpec {
    serde_json::from_value(
        json!({"format":"png","width":width,"height":height,"background":background}),
    )
    .unwrap()
}

#[test]
fn base_buffer_plan_admits_supported_dimensions_and_backgrounds() {
    for (w, h) in [(1, 1), (512, 256), (4096, 4096)] {
        for bg in ["transparent", "#00aAfF"] {
            let plan = plan_png_buffers(&output(w, h, bg), MAX_RENDER_BUFFER_BYTES).unwrap();
            assert_eq!(u64::from(plan.width), w);
            assert_eq!(u64::from(plan.height), h);
            assert_eq!(plan.rgba_bytes, w * h * 4);
            assert!(plan.reserved_bytes >= plan.rgba_bytes * 4);
            assert!(plan.reserved_bytes <= MAX_RENDER_BUFFER_BYTES);
        }
    }
}

#[test]
fn budget_boundary_and_concurrent_reservations_are_checked() {
    let out = output(4096, 4096, "transparent");
    let plan = plan_png_buffers(&out, MAX_RENDER_BUFFER_BYTES).unwrap();
    assert!(plan_png_buffers(&out, plan.reserved_bytes).is_ok());
    assert!(plan_png_buffers(&out, plan.reserved_bytes - 1).is_err());
    assert!(plan_png_buffers(&out, MAX_RENDER_BUFFER_BYTES - plan.reserved_bytes).is_err());
    assert!(plan_png_buffers(&out, 0).is_err());
    assert!(plan_png_buffers(&out, MAX_RENDER_BUFFER_BYTES + 1).is_err());
}

#[test]
fn invalid_dimensions_background_and_wrong_format_are_rejected() {
    for (w, h) in [(4097, 1), (1, 4097), (u64::MAX, u64::MAX)] {
        assert!(plan_png_buffers(&output(w, h, "transparent"), MAX_RENDER_BUFFER_BYTES).is_err());
    }
    for bg in ["red", "#fff", "#gggggg", ""] {
        assert!(
            serde_json::from_value::<OutputSpec>(
                json!({"format":"png","width":1,"height":1,"background":bg})
            )
            .is_err()
        );
    }
    assert!(
        serde_json::from_value::<OutputSpec>(
            json!({"format":"png","width":0,"height":1,"background":"transparent"})
        )
        .is_err()
    );
    let mut svg = output(1, 1, "transparent");
    svg.format = "svg".parse().unwrap();
    assert!(plan_png_buffers(&svg, MAX_RENDER_BUFFER_BYTES).is_err());
}

#[test]
fn png_has_requested_dimensions_and_background() {
    for (bg, expected) in [("transparent", [0, 0, 0, 0]), ("#00ff00", [0, 255, 0, 255])] {
        let bytes = render_png(SVG, &output(64, 32, bg)).unwrap();
        let image = resvg::tiny_skia::Pixmap::decode_png(&bytes).unwrap();
        assert_eq!((image.width(), image.height()), (64, 32));
        let corner = image.pixels()[0];
        assert_eq!(
            [corner.red(), corner.green(), corner.blue(), corner.alpha()],
            expected
        );
        let center = image.pixels()[16 * 64 + 32];
        assert_eq!(
            [center.red(), center.green(), center.blue(), center.alpha()],
            [255, 0, 0, 255]
        );
    }
}

#[test]
fn png_changes_with_content_and_is_repeatable() {
    let out = output(64, 64, "transparent");
    let first = render_png(SVG, &out).unwrap();
    assert_eq!(first, render_png(SVG, &out).unwrap());
    let blue = String::from_utf8(SVG.to_vec())
        .unwrap()
        .replace("#ff0000", "#0000ff");
    assert_ne!(first, render_png(blue.as_bytes(), &out).unwrap());
}

#[test]
fn crop_of_another_proportion_covers_viewport_without_letterboxing() {
    let svg=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 50"><rect width="100" height="50" fill="#ff0000"/></svg>"##;
    let image = resvg::tiny_skia::Pixmap::decode_png(
        &render_png(svg, &output(40, 80, "transparent")).unwrap(),
    )
    .unwrap();
    assert!(
        image
            .pixels()
            .iter()
            .all(|p| p.alpha() == 255 && p.red() == 255)
    );
}

#[test]
fn opacity_and_clip_are_rendered_on_requested_background() {
    let svg=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><clipPath id="c"><rect width="50" height="100"/></clipPath></defs><g opacity="0.5" clip-path="url(#c)"><rect width="100" height="100" fill="#ff0000"/></g></svg>"##;
    let image = resvg::tiny_skia::Pixmap::decode_png(
        &render_png(svg, &output(100, 100, "#0000ff")).unwrap(),
    )
    .unwrap();
    let left = image.pixels()[50 * 100 + 25];
    let right = image.pixels()[50 * 100 + 75];
    assert!(
        (127..=128).contains(&left.red())
            && (127..=128).contains(&left.blue())
            && left.alpha() == 255
    );
    assert_eq!([right.red(), right.blue(), right.alpha()], [0, 255, 255]);
}

#[test]
fn nested_layers_and_clip_chains_are_budgeted_before_rendering() {
    let flat = plan_png_render(
        SVG,
        &output(4096, 4096, "transparent"),
        MAX_RENDER_BUFFER_BYTES,
    )
    .unwrap();
    assert_eq!(flat.temporary_bytes, 0);
    let layered=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g opacity="0.5"><rect width="100" height="100" fill="#123456"/><g opacity="0.5"><rect width="100" height="100" fill="#abcdef"/><circle cx="50" cy="50" r="20"/></g></g></svg>"##;
    let plan = plan_png_render(
        layered,
        &output(256, 256, "transparent"),
        MAX_RENDER_BUFFER_BYTES,
    )
    .unwrap();
    assert!(plan.temporary_bytes >= 2 * plan.base.rgba_bytes);
    assert!(
        render_png_with_budget(
            layered,
            &output(256, 256, "transparent"),
            plan.reserved_bytes - 1
        )
        .is_err()
    );
    assert!(
        render_png_with_budget(
            layered,
            &output(256, 256, "transparent"),
            plan.reserved_bytes
        )
        .is_ok()
    );
    let chain=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><clipPath id="a"><rect width="100" height="100"/></clipPath><clipPath id="b" clip-path="url(#a)"><rect width="100" height="100"/></clipPath><clipPath id="c" clip-path="url(#b)"><rect width="100" height="100"/></clipPath></defs><g clip-path="url(#c)"><rect width="100" height="100"/></g></svg>"##;
    let small = plan_png_render(
        chain,
        &output(64, 64, "transparent"),
        MAX_RENDER_BUFFER_BYTES,
    )
    .unwrap();
    assert!(small.temporary_bytes >= small.base.rgba_bytes * 3);
    assert!(
        plan_png_render(
            chain,
            &output(4096, 4096, "transparent"),
            MAX_RENDER_BUFFER_BYTES
        )
        .is_err()
    );
}

#[test]
fn rasterizer_rejects_unsafe_svg_and_bad_requests() {
    for svg in [br#"<svg><script/></svg>"#.as_slice(),br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><image href="file:///private.png"/></svg>"#.as_slice()] {
        assert!(render_png(svg,&output(64,64,"transparent")).is_err());
    }
    assert!(render_png(SVG, &output(4097, 64, "transparent")).is_err());
    assert!(render_png_with_budget(SVG, &output(64, 64, "transparent"), 0).is_err());
}

#[test]
fn minimum_output_and_implicit_viewbox_scale_correctly() {
    let svg=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="100" height="100" fill="#ff0000"/></svg>"##;
    for (w, h) in [(1, 1), (32, 16)] {
        let image = resvg::tiny_skia::Pixmap::decode_png(
            &render_png(svg, &output(w, h, "transparent")).unwrap(),
        )
        .unwrap();
        assert!(
            image
                .pixels()
                .iter()
                .all(|p| p.alpha() == 255 && p.red() == 255)
        );
    }
    let error = mosaicforge_core::EngineError::RenderFailed("PNG encoding failed");
    assert_eq!(error.code(), "render_failed");
    assert_eq!(error.to_string(), "PNG encoding failed");
}

#[test]
fn unsupported_huge_layer_bounds_are_rejected_instead_of_silently_dropped() {
    let svg=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g opacity="0.5" transform="scale(1000000)"><rect width="1000000" height="1000000"/></g></svg>"##;
    assert!(plan_png_render(svg, &output(64, 64, "transparent"), MAX_RENDER_BUFFER_BYTES).is_err());
}
