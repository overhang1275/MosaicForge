//! Local measurement only; production rendering never calls this module.
use super::*;
use mosaicforge_core::{
    appearance::apply_appearance,
    constraints::{fit_transform_to_frame, measure_viewport_coverage},
    contract::GenerationSpec,
    distortion::distort_template,
    export::export_svg,
    generate::{apply_geometric_transform, generate_variant},
    rules::sample_transform,
    template::import_svg,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, io::Write, path::Path, time::Instant};

fn timed<T>(
    times: &mut BTreeMap<&'static str, f64>,
    stage: &'static str,
    f: impl FnOnce() -> T,
) -> T {
    let start = Instant::now();
    let result = f();
    times.insert(stage, start.elapsed().as_secs_f64() * 1000.0);
    result
}
fn save(path: &Path, bytes: &[u8]) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

#[test]
#[ignore = "requires out/test-prepared.svg and out/phase1-6.6/spec-svg.json; writes exclusive stage measurements"]
#[allow(clippy::assertions_on_constants)] // Compile in debug; reject debug only if this local measurement is run.
fn supplied_artwork_profiles_stages_in_release() {
    assert!(
        !cfg!(debug_assertions),
        "run this measurement with --release"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap();
    let directory = root.join("out/phase1-6.6/stages");
    fs::create_dir(&directory).unwrap();
    let spec: GenerationSpec =
        serde_json::from_slice(&fs::read(root.join("out/phase1-6.6/spec-svg.json")).unwrap())
            .unwrap();
    assert_eq!(spec.count.get(), 3);
    assert_eq!(spec.start_index.as_str(), "0");
    let mut setup = BTreeMap::new();
    let bytes = timed(&mut setup, "read_template_ms", || {
        fs::read(root.join("out/test-prepared.svg")).unwrap()
    });
    let template = timed(&mut setup, "import_normalize_ms", || {
        import_svg(&bytes).unwrap()
    });
    assert_eq!(template.hash, spec.template_hash.as_str());
    let mut rows = Vec::new();
    for index in 0..3 {
        let mut times = BTreeMap::new();
        let variant = timed(&mut times, "generate_inclusive_ms", || {
            generate_variant(&template, &spec, index).unwrap()
        });
        let candidate = variant.template.as_ref().expect("accepted fixture");
        let attempt = variant.attempts - 1;
        let mut p = timed(&mut times, "sample_rules_ms", || {
            sample_transform(&spec, index, attempt).unwrap()
        });
        assert!(timed(&mut times, "fit_frame_ms", || {
            fit_transform_to_frame(&template, &spec, &mut p).unwrap()
        }));
        assert_eq!(Some(&p), variant.parameters.as_ref());
        let distorted = timed(&mut times, "distortion_ms", || {
            distort_template(&template, &spec, index, attempt).unwrap()
        });
        let geometric = timed(&mut times, "geometry_ms", || {
            apply_geometric_transform(&distorted, &p).unwrap()
        });
        let replayed = timed(&mut times, "appearance_ms", || {
            apply_appearance(&geometric, &spec, index, attempt).unwrap()
        });
        assert_eq!(replayed.normalized_svg, candidate.normalized_svg);
        let coverage = timed(&mut times, "coverage_validation_ms", || {
            measure_viewport_coverage(
                &replayed,
                spec.output.width.get() as f64,
                spec.output.height.get() as f64,
            )
            .unwrap()
        });
        assert_eq!(Some(coverage), variant.coverage);
        let svg = timed(&mut times, "svg_serialize_validate_ms", || {
            export_svg(replayed.normalized_svg.as_bytes(), &spec.output).unwrap()
        });
        let mut png_output = spec.output.clone();
        png_output.format = "png".parse().unwrap();
        let (tree, plan) = timed(&mut times, "png_prepare_ms", || {
            prepare_png(
                replayed.normalized_svg.as_bytes(),
                &png_output,
                MAX_RENDER_BUFFER_BYTES,
            )
            .unwrap()
        });
        let image = timed(&mut times, "raster_allocate_fill_ms", || {
            let mut image =
                resvg::tiny_skia::Pixmap::new(plan.base.width, plan.base.height).unwrap();
            if png_output.background.as_str() != "transparent" {
                let rgb = u32::from_str_radix(&png_output.background.as_str()[1..], 16).unwrap();
                image.fill(resvg::tiny_skia::Color::from_rgba8(
                    (rgb >> 16) as u8,
                    (rgb >> 8) as u8,
                    rgb as u8,
                    255,
                ));
            }
            image
        });
        let mut image = image;
        timed(&mut times, "raster_render_ms", || {
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::identity(),
                &mut image.as_mut(),
            )
        });
        let png = timed(&mut times, "png_encode_ms", || image.encode_png().unwrap());
        let (svg_hash, png_hash) = timed(&mut times, "hash_svg_and_png_ms", || {
            (
                format!("{:x}", Sha256::digest(&svg)),
                format!("{:x}", Sha256::digest(&png)),
            )
        });
        for (format, data) in [("svg", &svg), ("png", &png)] {
            assert_eq!(
                data,
                &fs::read(root.join(format!("out/cli-6.3-{format}/variant-{index}.{format}")))
                    .unwrap()
            );
        }
        timed(&mut times, "write_sync_svg_and_png_ms", || {
            save(&directory.join(format!("variant-{index}.svg")), &svg);
            save(&directory.join(format!("variant-{index}.png")), &png);
        });
        rows.push(json!({"index":index,"attempts":variant.attempts,"milliseconds":times,"svgBytes":svg.len(),"pngBytes":png.len(),"svgHash":svg_hash,"pngHash":png_hash,"pngReservedBytes":plan.reserved_bytes}));
    }
    let result = json!({"build":"release","backend":"cpu","templateBytes":bytes.len(),"templateHash":template.hash,"setupMilliseconds":setup,"variants":rows,"scope":"Accepted attempt replay; generate_inclusive includes retries and nested imports. Replayed times are not additive with it or CLI totals. File writes include sync_all but exclude CLI hard_link/manifests."});
    save(
        &directory.join("profile.json"),
        &serde_json::to_vec_pretty(&result).unwrap(),
    );
}
