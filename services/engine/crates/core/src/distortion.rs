use crate::{
    EngineError,
    appearance::Element,
    contract::GenerationSpec,
    rng::{map_u32_to_range, sample_u32},
    svg_values::{bounded, invalid},
    template::{Template, import_svg},
    validate_spec,
};
use quick_xml::Writer;
use svgtypes::SimplePathSegment;

struct Warp {
    amplitude: f64,
    frequency: f64,
    phase_x: f64,
    phase_y: f64,
}

impl Warp {
    fn point(&self, x: f64, y: f64) -> Result<(f64, f64), EngineError> {
        let dx = self.amplitude
            * (x * self.frequency + self.phase_x).sin()
            * (y * self.frequency + self.phase_y).cos();
        let dy = self.amplitude
            * (y * self.frequency + self.phase_y).sin()
            * (x * self.frequency + self.phase_x).cos();
        let point = (x + dx, y + dy);
        bounded(&[point.0, point.1])?;
        Ok(point)
    }

    fn apply(&self, node: &mut Element) -> Result<(), EngineError> {
        if matches!(node.name.as_str(), "defs" | "clipPath") {
            return Ok(());
        }
        if node.name == "path" {
            let original = node
                .attrs
                .get("d")
                .ok_or(invalid("missing distortion path"))?;
            let mut result = String::new();
            for segment in svgtypes::SimplifyingPathParser::from(original.as_str()) {
                match segment.map_err(|_| invalid("invalid distortion path"))? {
                    SimplePathSegment::MoveTo { x, y } => {
                        let (x, y) = self.point(x, y)?;
                        result.push_str(&format!("M{x} {y}"));
                    }
                    SimplePathSegment::LineTo { x, y } => {
                        let (x, y) = self.point(x, y)?;
                        result.push_str(&format!("L{x} {y}"));
                    }
                    SimplePathSegment::Quadratic { x1, y1, x, y } => {
                        let (x1, y1) = self.point(x1, y1)?;
                        let (x, y) = self.point(x, y)?;
                        result.push_str(&format!("Q{x1} {y1} {x} {y}"));
                    }
                    SimplePathSegment::CurveTo {
                        x1,
                        y1,
                        x2,
                        y2,
                        x,
                        y,
                    } => {
                        let (x1, y1) = self.point(x1, y1)?;
                        let (x2, y2) = self.point(x2, y2)?;
                        let (x, y) = self.point(x, y)?;
                        result.push_str(&format!("C{x1} {y1} {x2} {y2} {x} {y}"));
                    }
                    SimplePathSegment::ClosePath => result.push('Z'),
                }
            }
            node.attrs.insert("d".into(), result);
        }
        for child in &mut node.children {
            self.apply(child)?;
        }
        Ok(())
    }
}

pub fn distort_template(
    template: &Template,
    spec: &GenerationSpec,
    index: u64,
    attempt: u8,
) -> Result<Template, EngineError> {
    validate_spec(spec)?;
    let seed = spec
        .seed
        .parse::<u64>()
        .map_err(|_| EngineError::InvalidSpec("seed exceeds u64"))?;
    let phase = |rule| {
        map_u32_to_range(
            sample_u32(seed, index, rule, attempt, 0)?,
            0.0,
            std::f64::consts::TAU,
        )
    };
    let (phase_x, phase_y) = (phase(9)?, phase(10)?);
    let source = import_svg(template.normalized_svg.as_bytes())?;
    if spec.rules.distortion == 0.0 {
        return Ok(source);
    }
    bounded(&[spec.rules.distortion])?;
    let warp = Warp {
        amplitude: spec.rules.distortion / std::f64::consts::SQRT_2,
        frequency: std::f64::consts::TAU / source.view_box[2].max(source.view_box[3]),
        phase_x,
        phase_y,
    };
    let mut root = Element::read(&source.normalized_svg)?;
    warp.apply(&mut root)?;
    let mut writer = Writer::new(Vec::new());
    root.write(&mut writer)?;
    import_svg(&writer.into_inner())
}
