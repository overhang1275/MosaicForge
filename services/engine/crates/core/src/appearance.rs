use crate::{
    EngineError,
    contract::GenerationSpec,
    rng::{map_u32_to_range, sample_u32},
    svg_values::{bounded, invalid},
    template::{Template, import_svg},
    validate_spec,
};
use quick_xml::{
    Reader, Writer, XmlVersion,
    events::{BytesEnd, BytesStart, Event},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

const OPACITY_RULE: u16 = 6;
const PALETTE_RULE: u16 = 7;
const LAYER_ORDER_RULE: u16 = 8;
type ColorKey = (u8, u8, u8, u8);

struct Element {
    name: String,
    attrs: BTreeMap<String, String>,
    children: Vec<Element>,
}

impl Element {
    fn read(svg: &str) -> Result<Self, EngineError> {
        let mut reader = Reader::from_str(svg);
        let mut stack: Vec<Self> = Vec::new();
        loop {
            match reader
                .read_event()
                .map_err(|_| invalid("cannot read normalized SVG"))?
            {
                Event::Start(start) => {
                    let mut attrs = BTreeMap::new();
                    for attr in start.attributes() {
                        let attr = attr.map_err(|_| invalid("invalid normalized attribute"))?;
                        attrs.insert(
                            attr.key.as_ref().to_owned(),
                            attr.normalized_value(XmlVersion::Implicit1_0)
                                .map_err(|_| invalid("invalid normalized attribute value"))?
                                .into_owned(),
                        );
                    }
                    stack.push(Self {
                        name: start.name().as_ref().to_owned(),
                        attrs,
                        children: Vec::new(),
                    });
                }
                Event::End(_) => {
                    let node = stack.pop().ok_or(invalid("invalid normalized nesting"))?;
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        return Ok(node);
                    }
                }
                Event::Eof => return Err(invalid("missing normalized SVG root")),
                _ => return Err(invalid("unexpected normalized SVG event")),
            }
        }
    }

    fn write(&self, writer: &mut Writer<Vec<u8>>) -> Result<(), EngineError> {
        let mut start = BytesStart::new(&self.name);
        for (key, value) in &self.attrs {
            start.push_attribute((key.as_str(), value.as_str()));
        }
        writer
            .write_event(Event::Start(start))
            .map_err(|_| invalid("cannot write SVG appearance"))?;
        for child in &self.children {
            child.write(writer)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new(&self.name)))
            .map_err(|_| invalid("cannot write SVG appearance"))
    }

    fn colors(&self, colors: &mut BTreeSet<ColorKey>) -> Result<(), EngineError> {
        for key in ["fill", "stroke"] {
            if let Some(value) = self.attrs.get(key)
                && value != "none"
            {
                colors.insert(color_key(value)?);
            }
        }
        for child in &self.children {
            child.colors(colors)?;
        }
        Ok(())
    }

    fn recolor(&mut self, colors: &BTreeMap<ColorKey, String>) -> Result<(), EngineError> {
        for key in ["fill", "stroke"] {
            if let Some(value) = self.attrs.get_mut(key)
                && value != "none"
            {
                *value = colors
                    .get(&color_key(value)?)
                    .ok_or(invalid("missing palette mapping"))?
                    .clone();
            }
        }
        for child in &mut self.children {
            child.recolor(colors)?;
        }
        Ok(())
    }
}

fn color_key(value: &str) -> Result<ColorKey, EngineError> {
    let c = svgtypes::Color::from_str(value).map_err(|_| invalid("invalid solid color"))?;
    Ok((c.red, c.green, c.blue, c.alpha))
}

fn choose(value: u32, count: usize) -> usize {
    ((u64::from(value) * count as u64) >> 32) as usize
}

pub fn apply_appearance(
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
    // Validate even an identity operation or a template with no layers.
    sample_u32(seed, index, OPACITY_RULE, attempt, 0)?;
    let source = import_svg(template.normalized_svg.as_bytes())?;
    let mut root = Element::read(&source.normalized_svg)?;
    let rules = &spec.rules;
    if let Some(crop) = &rules.crop {
        let [x, y, width, height] = source.view_box;
        bounded(&[crop.x, crop.y, crop.width, crop.height])?;
        if crop.x < x
            || crop.y < y
            || crop.x + crop.width > x + width
            || crop.y + crop.height > y + height
        {
            return Err(EngineError::InvalidSpec(
                "crop must stay inside the template viewBox",
            ));
        }
        root.attrs.insert(
            "viewBox".into(),
            format!("{} {} {} {}", crop.x, crop.y, crop.width, crop.height),
        );
    }
    let slots: Vec<usize> = root
        .children
        .iter()
        .enumerate()
        .filter_map(|(i, child)| (child.name == "g").then_some(i))
        .collect();
    for (ordinal, &slot) in slots.iter().enumerate() {
        let layer = &mut root.children[slot];
        let opacity = map_u32_to_range(
            sample_u32(seed, index, OPACITY_RULE, attempt, ordinal as u32)?,
            rules.opacity.min,
            rules.opacity.max,
        )?;
        if opacity != 1.0 {
            let existing = layer
                .attrs
                .get("opacity")
                .map_or(Ok(1.0), |n| n.parse::<f64>())
                .map_err(|_| invalid("invalid layer opacity"))?;
            layer
                .attrs
                .insert("opacity".into(), (existing * opacity).to_string());
        }
        if !rules.palette.is_empty() {
            for (key, default) in [("fill", "black"), ("stroke", "none")] {
                layer.attrs.entry(key.into()).or_insert_with(|| {
                    root.attrs
                        .get(key)
                        .cloned()
                        .unwrap_or_else(|| default.into())
                });
            }
        }
    }
    if !rules.palette.is_empty() {
        let mut originals = BTreeSet::new();
        for &slot in &slots {
            root.children[slot].colors(&mut originals)?;
        }
        let mut colors = BTreeMap::new();
        for (draw, original) in originals.into_iter().enumerate() {
            let selected = &rules.palette[choose(
                sample_u32(seed, index, PALETTE_RULE, attempt, draw as u32)?,
                rules.palette.len(),
            )];
            let replacement = if original.3 == 255 {
                selected.to_ascii_lowercase()
            } else {
                let (r, g, b, _) = color_key(selected)?;
                format!("rgba({r},{g},{b},{})", f64::from(original.3) / 255.0)
            };
            colors.insert(original, replacement);
        }
        for &slot in &slots {
            root.children[slot].recolor(&colors)?;
        }
    }
    if rules.shuffle_layers {
        for i in (1..slots.len()).rev() {
            let j = choose(
                sample_u32(
                    seed,
                    index,
                    LAYER_ORDER_RULE,
                    attempt,
                    (slots.len() - 1 - i) as u32,
                )?,
                i + 1,
            );
            root.children.swap(slots[i], slots[j]);
        }
    }
    let mut writer = Writer::new(Vec::new());
    root.write(&mut writer)?;
    import_svg(&writer.into_inner())
}
