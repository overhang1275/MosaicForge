use crate::{
    EngineError,
    svg_values::invalid,
    template::{Template, import_svg},
};
use crate::{
    appearance::apply_appearance,
    contract::GenerationSpec,
    distortion::distort_template,
    generate::{apply_geometric_transform, geometric_matrix},
    rules::{TransformParameters, sample_transform},
    validate_spec,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Coverage {
    pub painted_pixels: u32,
    pub occupied_tiles: u16,
    pub empty_corner_tiles: u16,
}

fn output_crop(source: &Template, spec: &GenerationSpec) -> Result<[f64; 4], EngineError> {
    let [x, y, width, height] = source.view_box;
    let crop = spec
        .rules
        .crop
        .as_ref()
        .map_or(source.view_box, |c| [c.x, c.y, c.width, c.height]);
    if crop[0] < x || crop[1] < y || crop[0] + crop[2] > x + width || crop[1] + crop[3] > y + height
    {
        return Err(EngineError::InvalidSpec(
            "crop must stay inside the template viewBox",
        ));
    }
    Ok(crop)
}

pub fn fit_transform_to_frame(
    template: &Template,
    spec: &GenerationSpec,
    p: &mut TransformParameters,
) -> Result<bool, EngineError> {
    validate_spec(spec)?;
    for (value, range) in [
        (p.translate_x, &spec.rules.translate_x),
        (p.translate_y, &spec.rules.translate_y),
        (p.rotation_degrees, &spec.rules.rotation_degrees),
        (p.scale_x, &spec.rules.scale_x),
        (p.scale_y, &spec.rules.scale_y),
    ] {
        if !value.is_finite() || value < range.min || value > range.max {
            return Err(EngineError::InvalidSpec(
                "framing parameters exceed configured ranges",
            ));
        }
    }
    if p.mirror_x != spec.rules.mirror_x || p.mirror_y != spec.rules.mirror_y {
        return Err(EngineError::InvalidSpec(
            "framing mirror flags differ from configured rules",
        ));
    }
    let source = import_svg(template.normalized_svg.as_bytes())?;
    let [x, y, width, height] = source.view_box;
    let crop = output_crop(&source, spec)?;
    let [a, b, c, d, e, f] = geometric_matrix(p, source.view_box)?;
    let det = a * d - b * c;
    let (cx, cy) = (x + width / 2.0, y + height / 2.0);
    let mut factor = 1.0f64;
    for px in [crop[0], crop[0] + crop[2]] {
        for py in [crop[1], crop[1] + crop[3]] {
            let ix = (d * (px - e) - c * (py - f)) / det;
            let iy = (-b * (px - e) + a * (py - f)) / det;
            factor = factor
                .max((ix - cx).abs() / (width / 2.0))
                .max((iy - cy).abs() / (height / 2.0));
        }
    }
    // Inversion/trigonometry can put an exact quarter turn a few ulps above its bound.
    let capped_scale = |value: f64, maximum: f64| {
        if !value.is_finite() || value > maximum && value - maximum > maximum * 8.0 * f64::EPSILON {
            None
        } else {
            Some(value.min(maximum))
        }
    };
    let Some((sx, sy)) = capped_scale(p.scale_x * factor, spec.rules.scale_x.max)
        .zip(capped_scale(p.scale_y * factor, spec.rules.scale_y.max))
    else {
        return Ok(false);
    };
    p.scale_x = sx;
    p.scale_y = sy;
    Ok(true)
}

#[derive(Debug)]
pub struct CandidateDecision {
    pub template: Option<Template>,
    pub parameters: Option<TransformParameters>,
    pub attempts: u8,
    pub reason: Option<&'static str>,
    pub coverage: Option<Coverage>,
}

pub fn search_candidate(
    template: &Template,
    spec: &GenerationSpec,
    index: u64,
) -> Result<CandidateDecision, EngineError> {
    validate_spec(spec)?;
    let source = import_svg(template.normalized_svg.as_bytes())?;
    // Configuration errors must not be turned into eight candidate rejections.
    output_crop(&source, spec)?;
    let mut reason = "invalid_geometry";
    let mut last_coverage = None;
    for attempt in 0..8 {
        last_coverage = None;
        let mut p = sample_transform(spec, index, attempt)?;
        match fit_transform_to_frame(&source, spec, &mut p) {
            Ok(true) => (),
            Ok(false) => {
                reason = "frame_exceeds_scale_limits";
                continue;
            }
            Err(EngineError::InvalidSvg(_)) => {
                reason = "invalid_geometry";
                continue;
            }
            Err(error) => return Err(error),
        }
        let candidate = distort_template(&source, spec, index, attempt)
            .and_then(|t| apply_geometric_transform(&t, &p))
            .and_then(|t| apply_appearance(&t, spec, index, attempt));
        let candidate = match candidate {
            Ok(candidate) => candidate,
            Err(EngineError::InvalidSvg(_)) => {
                reason = "invalid_geometry";
                continue;
            }
            Err(error) => return Err(error),
        };
        let coverage = measure_coverage(&candidate)?;
        reason = if coverage.painted_pixels < 512 {
            "empty_viewport"
        } else if coverage.empty_corner_tiles != 0 {
            "empty_corners"
        } else if coverage.occupied_tiles < 250 {
            "sparse_viewport"
        } else {
            return Ok(CandidateDecision {
                template: Some(candidate),
                parameters: Some(p),
                attempts: attempt + 1,
                reason: None,
                coverage: Some(coverage),
            });
        };
        last_coverage = Some(coverage);
    }
    Ok(CandidateDecision {
        template: None,
        parameters: None,
        attempts: 8,
        reason: Some(reason),
        coverage: last_coverage,
    })
}

pub fn measure_coverage(template: &Template) -> Result<Coverage, EngineError> {
    let source = import_svg(template.normalized_svg.as_bytes())?;
    // Measure the crop itself; original width/height can letterbox a differently shaped crop.
    let mut root = crate::appearance::Element::read(&source.normalized_svg)?;
    root.attrs
        .insert("width".into(), source.view_box[2].to_string());
    root.attrs
        .insert("height".into(), source.view_box[3].to_string());
    let mut writer = quick_xml::Writer::new(Vec::new());
    root.write(&mut writer)?;
    let mask_svg = String::from_utf8(writer.into_inner())
        .map_err(|_| invalid("invalid coverage SVG encoding"))?;
    let tree = usvg::Tree::from_str(&mask_svg, &usvg::Options::default())
        .map_err(|_| invalid("cannot measure SVG coverage"))?;
    let mut mask =
        resvg::tiny_skia::Pixmap::new(256, 256).ok_or(invalid("cannot allocate coverage mask"))?;
    let transform = resvg::tiny_skia::Transform::from_scale(
        256.0 / tree.size().width(),
        256.0 / tree.size().height(),
    );
    resvg::render(&tree, transform, &mut mask.as_mut());
    let mut tiles = [0u16; 256];
    let mut painted_pixels = 0;
    for (i, pixel) in mask.pixels().iter().enumerate() {
        if pixel.alpha() >= 8 {
            painted_pixels += 1;
            tiles[(i / 256 / 16) * 16 + (i % 256 / 16)] += 1;
        }
    }
    let occupied_tiles = tiles.iter().filter(|&&n| n >= 2).count() as u16;
    let empty_corner_tiles = tiles
        .iter()
        .enumerate()
        .filter(|&(i, &n)| n < 2 && (i / 16 < 2 || i / 16 >= 14) && (i % 16 < 2 || i % 16 >= 14))
        .count() as u16;
    Ok(Coverage {
        painted_pixels,
        occupied_tiles,
        empty_corner_tiles,
    })
}
