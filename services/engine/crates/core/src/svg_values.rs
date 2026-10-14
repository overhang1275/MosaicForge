use crate::EngineError;
use std::{collections::BTreeMap, str::FromStr};

pub(super) fn invalid(message: &'static str) -> EngineError {
    EngineError::InvalidSvg(message)
}

pub(super) fn bounded(values: &[f64]) -> Result<(), EngineError> {
    if values
        .iter()
        .any(|n| !n.is_finite() || n.abs() > 1_000_000.0 || (*n != 0.0 && *n as f32 == 0.0))
    {
        return Err(invalid("SVG coordinates exceed finite supported limits"));
    }
    Ok(())
}

pub(super) fn length(value: &str) -> Result<f64, EngineError> {
    use svgtypes::LengthUnit::*;
    let parsed = svgtypes::Length::from_str(value).map_err(|_| invalid("invalid SVG length"))?;
    bounded(&[parsed.number])?;
    let factor = match parsed.unit {
        None | Px => 1.0,
        In => 96.0,
        Cm => 96.0 / 2.54,
        Mm => 96.0 / 25.4,
        Pt => 96.0 / 72.0,
        Pc => 16.0,
        _ => return Err(invalid("relative SVG length units are not supported")),
    };
    let number = parsed.number * factor;
    bounded(&[number])?;
    Ok(number)
}

pub(super) fn numbers(value: &str) -> Result<Vec<f64>, EngineError> {
    let mut output = Vec::new();
    for number in svgtypes::NumberListParser::from(value) {
        let number = number.map_err(|_| invalid("invalid SVG number list"))?;
        bounded(&[number])?;
        if output.len() >= 200_000 {
            return Err(invalid("SVG number list exceeds segment limits"));
        }
        output.push(number);
    }
    Ok(output)
}

pub(super) fn allowed(tag: &str, attribute: &str) -> bool {
    if matches!(
        attribute,
        "id" | "transform"
            | "fill"
            | "stroke"
            | "stroke-width"
            | "stroke-linecap"
            | "stroke-linejoin"
            | "stroke-miterlimit"
            | "fill-rule"
            | "clip-rule"
            | "fill-opacity"
            | "stroke-opacity"
            | "opacity"
            | "clip-path"
    ) {
        return true;
    }
    match tag {
        "svg" => matches!(
            attribute,
            "xmlns" | "width" | "height" | "viewBox" | "preserveAspectRatio"
        ),
        "rect" => matches!(attribute, "x" | "y" | "width" | "height" | "rx" | "ry"),
        "circle" => matches!(attribute, "cx" | "cy" | "r"),
        "ellipse" => matches!(attribute, "cx" | "cy" | "rx" | "ry"),
        "line" => matches!(attribute, "x1" | "y1" | "x2" | "y2"),
        "path" => attribute == "d",
        "polyline" | "polygon" => attribute == "points",
        "clipPath" => attribute == "clipPathUnits",
        _ => false,
    }
}

pub(super) fn validate(tag: &str, attrs: &BTreeMap<String, String>) -> Result<usize, EngineError> {
    for (name, value) in attrs {
        if !allowed(tag, name) {
            return Err(invalid("unsupported SVG attribute"));
        }
        match name.as_str() {
            "id" | "xmlns" | "clip-path" => (),
            "fill" | "stroke" => {
                if value != "none" {
                    svgtypes::Color::from_str(value)
                        .map_err(|_| invalid("only solid SVG colors are supported"))?;
                }
            }
            "fill-rule" | "clip-rule" => {
                if !matches!(value.as_str(), "nonzero" | "evenodd") {
                    return Err(invalid("invalid SVG fill rule"));
                }
            }
            "stroke-linecap" => {
                if !matches!(value.as_str(), "butt" | "round" | "square") {
                    return Err(invalid("invalid SVG line cap"));
                }
            }
            "stroke-linejoin" => {
                if !matches!(value.as_str(), "miter" | "round" | "bevel") {
                    return Err(invalid("invalid SVG line join"));
                }
            }
            "clipPathUnits" => {
                if !matches!(value.as_str(), "userSpaceOnUse" | "objectBoundingBox") {
                    return Err(invalid("invalid clip units"));
                }
            }
            "preserveAspectRatio" => {
                svgtypes::AspectRatio::from_str(value)
                    .map_err(|_| invalid("invalid SVG aspect ratio"))?;
            }
            "viewBox" => {
                let values = numbers(value)?;
                if values.len() != 4 || values[2] <= 0.0 || values[3] <= 0.0 {
                    return Err(invalid("invalid SVG viewBox"));
                }
            }
            "transform" => {
                let t = svgtypes::Transform::from_str(value)
                    .map_err(|_| invalid("invalid SVG transform"))?;
                bounded(&[t.a, t.b, t.c, t.d, t.e, t.f])?;
            }
            "d" | "points" => (),
            "opacity" | "fill-opacity" | "stroke-opacity" | "stroke-miterlimit" => {
                let n = value
                    .parse::<f64>()
                    .map_err(|_| invalid("invalid SVG presentation number"))?;
                bounded(&[n])?;
                if (name == "stroke-miterlimit" && n < 1.0)
                    || (name != "stroke-miterlimit" && !(0.0..=1.0).contains(&n))
                {
                    return Err(invalid("SVG presentation number out of range"));
                }
            }
            _ => {
                let n = length(value)?;
                if matches!(
                    name.as_str(),
                    "width" | "height" | "r" | "rx" | "ry" | "stroke-width"
                ) && n < 0.0
                {
                    return Err(invalid("negative SVG size"));
                }
            }
        }
    }
    for required in match tag {
        "rect" => &["width", "height"][..],
        "circle" => &["r"][..],
        "ellipse" => &["rx", "ry"][..],
        _ => &[],
    } {
        if length(
            attrs
                .get(*required)
                .ok_or(invalid("missing SVG shape size"))?,
        )? <= 0.0
        {
            return Err(invalid("SVG shape size must be positive"));
        }
    }
    let segments = match tag {
        "path" => {
            let d = attrs.get("d").ok_or(invalid("missing SVG path data"))?;
            let mut count = 0;
            let mut drawable = false;
            for segment in svgtypes::SimplifyingPathParser::from(d.as_str()) {
                use svgtypes::SimplePathSegment::*;
                let segment = segment.map_err(|_| invalid("invalid SVG path"))?;
                drawable |= matches!(segment, LineTo { .. } | CurveTo { .. } | Quadratic { .. });
                match segment {
                    MoveTo { x, y } | LineTo { x, y } => bounded(&[x, y])?,
                    CurveTo {
                        x1,
                        y1,
                        x2,
                        y2,
                        x,
                        y,
                    } => bounded(&[x1, y1, x2, y2, x, y])?,
                    Quadratic { x1, y1, x, y } => bounded(&[x1, y1, x, y])?,
                    ClosePath => (),
                }
                count += 1;
                if count > 100_000 {
                    return Err(invalid("SVG segment limit exceeded"));
                }
            }
            if !drawable {
                return Err(invalid("SVG path has no drawable segments"));
            }
            count
        }
        "polyline" | "polygon" => {
            let points = numbers(attrs.get("points").ok_or(invalid("missing SVG points"))?)?;
            if points.len() % 2 != 0 || points.len() < if tag == "polygon" { 6 } else { 4 } {
                return Err(invalid("invalid SVG points"));
            }
            points.len() / 2 + usize::from(tag == "polygon")
        }
        "rect" => {
            if attrs.contains_key("rx") || attrs.contains_key("ry") {
                10
            } else {
                5
            }
        }
        "circle" | "ellipse" => 6,
        "line" => 2,
        _ => 0,
    };
    Ok(segments)
}

pub(super) fn compose(parent: [f64; 6], value: Option<&String>) -> Result<[f64; 6], EngineError> {
    let t = match value {
        Some(value) => {
            svgtypes::Transform::from_str(value).map_err(|_| invalid("invalid SVG transform"))?
        }
        None => svgtypes::Transform::default(),
    };
    validate_transform([t.a, t.b, t.c, t.d, t.e, t.f])?;
    let [a, b, c, d, e, f] = parent;
    let result = [
        a * t.a + c * t.b,
        b * t.a + d * t.b,
        a * t.c + c * t.d,
        b * t.c + d * t.d,
        a * t.e + c * t.f + e,
        b * t.e + d * t.f + f,
    ];
    validate_transform(result)?;
    Ok(result)
}

fn validate_transform(result: [f64; 6]) -> Result<(), EngineError> {
    bounded(&result)?;
    let backend = usvg::Transform::from_row(
        result[0] as f32,
        result[1] as f32,
        result[2] as f32,
        result[3] as f32,
        result[4] as f32,
        result[5] as f32,
    );
    let (x_scale, y_scale) = backend.get_scale();
    if (x_scale <= f32::EPSILON && (result[0] != 0.0 || result[2] != 0.0))
        || (y_scale <= f32::EPSILON && (result[1] != 0.0 || result[3] != 0.0))
    {
        return Err(invalid(
            "nonzero SVG transform scale is below backend tolerance",
        ));
    }
    Ok(())
}
