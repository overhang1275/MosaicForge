use crate::{
    EngineError,
    rules::TransformParameters,
    svg_values::{bounded, compose, invalid},
    template::{Template, import_svg},
};
use quick_xml::{
    Reader, Writer, XmlVersion,
    events::{BytesStart, Event},
};
use std::collections::BTreeMap;

const IDENTITY: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

pub fn geometric_matrix(
    p: &TransformParameters,
    view_box: [f64; 4],
) -> Result<[f64; 6], EngineError> {
    if ![
        p.translate_x,
        p.translate_y,
        p.rotation_degrees,
        p.scale_x,
        p.scale_y,
    ]
    .iter()
    .all(|n| n.is_finite())
        || p.scale_x <= 0.0
        || p.scale_y <= 0.0
    {
        return Err(EngineError::InvalidSpec(
            "geometric parameters must be finite with positive scales",
        ));
    }
    bounded(&view_box)?;
    let [x, y, width, height] = view_box;
    if width <= 0.0 || height <= 0.0 {
        return Err(EngineError::InvalidSpec("viewBox size must be positive"));
    }
    let (cx, cy) = (x + width / 2.0, y + height / 2.0);
    let sx = if p.mirror_x { -p.scale_x } else { p.scale_x };
    let sy = if p.mirror_y { -p.scale_y } else { p.scale_y };
    let (sin, cos) = p.rotation_degrees.rem_euclid(360.0).to_radians().sin_cos();
    let (a, b, c, d) = (cos * sx, sin * sx, -sin * sy, cos * sy);
    let matrix = [
        a,
        b,
        c,
        d,
        cx + p.translate_x - a * cx - c * cy,
        cy + p.translate_y - b * cx - d * cy,
    ];
    compose(matrix, None)
}

pub fn apply_geometric_transform(
    template: &Template,
    parameters: &TransformParameters,
) -> Result<Template, EngineError> {
    // Template has public fields; re-import before using metadata or taking the identity shortcut.
    let source = import_svg(template.normalized_svg.as_bytes())?;
    let matrix = geometric_matrix(parameters, source.view_box)?;
    if matrix == IDENTITY {
        return Ok(source);
    }
    let mut reader = Reader::from_str(&source.normalized_svg);
    let Event::Start(root) = reader
        .read_event()
        .map_err(|_| invalid("cannot read normalized SVG root"))?
    else {
        return Err(invalid("normalized SVG has no root"));
    };
    let mut attrs = BTreeMap::new();
    for attr in root.attributes() {
        let attr = attr.map_err(|_| invalid("invalid normalized root attributes"))?;
        attrs.insert(
            attr.key.as_ref().to_owned(),
            attr.normalized_value(XmlVersion::Implicit1_0)
                .map_err(|_| invalid("invalid normalized attribute value"))?
                .into_owned(),
        );
    }
    let combined = compose(matrix, attrs.get("transform"))?;
    attrs.insert(
        "transform".to_owned(),
        format!(
            "matrix({} {} {} {} {} {})",
            combined[0], combined[1], combined[2], combined[3], combined[4], combined[5]
        ),
    );
    let mut rewritten = BytesStart::new("svg");
    for (name, value) in &attrs {
        rewritten.push_attribute((name.as_str(), value.as_str()));
    }
    let mut writer = Writer::new(Vec::new());
    writer
        .write_event(Event::Start(rewritten))
        .map_err(|_| invalid("cannot write SVG transform"))?;
    loop {
        let event = reader
            .read_event()
            .map_err(|_| invalid("cannot read normalized SVG"))?;
        if matches!(event, Event::Eof) {
            break;
        }
        writer
            .write_event(event)
            .map_err(|_| invalid("cannot write transformed SVG"))?;
    }
    import_svg(&writer.into_inner())
}
