use mosaicforge_core::{contract::OutputSpec, export::export_svg};
use mosaicforge_cpu::render::render_png;
use serde_json::json;

fn output(background: &str) -> OutputSpec {
    serde_json::from_value(json!({"format":"svg","width":80,"height":40,"background":background}))
        .unwrap()
}

#[test]
fn svg_and_png_have_identical_pixels_with_root_effects_and_offset_viewbox() {
    let sources=[
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80" viewBox="-10 -20 100 80" transform="rotate(20 40 20)" opacity="0.5" fill-opacity="0.4" stroke="#abcdef" clip-rule="evenodd" clip-path="url(#c)"><defs><clipPath id="c"><path d="M-10 -20L90 -20L90 60L-10 60Z M10 0L30 0L30 20L10 20Z"/></clipPath></defs><g id="layer"><rect x="-10" y="-20" width="100" height="80" fill="#ff0000"/></g></svg>"##.as_slice(),
        br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" fill="none" fill-opacity="0.2" stroke="red" stroke-width="4"><rect x="25" y="25" width="50" height="50" fill="#0000ff"/></svg>"##.as_slice(),
    ];
    for svg in sources {
        for background in ["transparent", "#00ff00"] {
            let spec = output(background);
            let bytes = export_svg(svg, &spec).unwrap();
            let text = std::str::from_utf8(&bytes).unwrap();
            let tree = resvg::usvg::Tree::from_str(text, &resvg::usvg::Options::default()).unwrap();
            let mut image = resvg::tiny_skia::Pixmap::new(80, 40).unwrap();
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::identity(),
                &mut image.as_mut(),
            );
            let mut png = spec;
            png.format = "png".parse().unwrap();
            let expected =
                resvg::tiny_skia::Pixmap::decode_png(&render_png(svg, &png).unwrap()).unwrap();
            assert_eq!(image.data(), expected.data(), "background={background}");
        }
    }
}
