use mosaicforge_core::contract::OutputSpec;
use mosaicforge_cpu::render::{MAX_RENDER_BUFFER_BYTES, plan_png_buffers};
use serde_json::json;

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
