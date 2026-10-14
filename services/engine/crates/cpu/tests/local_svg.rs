use mosaicforge_core::contract::OutputSpec;
use mosaicforge_cpu::render::{MAX_RENDER_BUFFER_BYTES, plan_png_render, render_png};
use serde_json::json;
use std::{fs, io::Write, path::Path};

#[test]
#[ignore = "requires local out/test-4.4-{0,1,17}.svg samples"]
fn supplied_svg_renders_repeatable_real_pngs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    for index in [0, 1, 17] {
        let svg = fs::read(root.join(format!("out/test-4.4-{index}.svg"))).unwrap();
        let output: OutputSpec = serde_json::from_value(
            json!({"format":"png","width":512,"height":512,"background":"#ffffff"}),
        )
        .unwrap();
        let plan = plan_png_render(&svg, &output, MAX_RENDER_BUFFER_BYTES).unwrap();
        let png = render_png(&svg, &output).unwrap();
        assert_eq!(png, render_png(&svg, &output).unwrap());
        let image = resvg::tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!((image.width(), image.height()), (512, 512));
        assert!(image.pixels().iter().all(|p| p.alpha() == 255));
        assert!(
            image
                .pixels()
                .iter()
                .filter(|p| p.red() != 255 || p.green() != 255 || p.blue() != 255)
                .count()
                > 1000
        );
        let path = root.join(format!("out/test-5.2-{index}.png"));
        if path.exists() {
            assert_eq!(fs::read(&path).unwrap(), png);
        } else {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap()
                .write_all(&png)
                .unwrap();
        }
        println!(
            "index={index} PNG={} bytes, base={}, temporaries={}, reservation={} -> {}",
            png.len(),
            plan.base.reserved_bytes,
            plan.temporary_bytes,
            plan.reserved_bytes,
            path.display()
        );
    }
}
