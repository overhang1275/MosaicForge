use crate::{
    EngineError,
    appearance::Element,
    contract::{OutputSpec, generation::OutputSpecFormat},
    template::{import_svg, prepare_viewport},
    validate_output,
};
use std::collections::BTreeMap;

pub fn export_svg(svg: &[u8], output: &OutputSpec) -> Result<Vec<u8>, EngineError> {
    validate_output(output)?;
    if output.format != OutputSpecFormat::Svg {
        return Err(EngineError::InvalidSpec("SVG export requires SVG output"));
    }
    let source = prepare_viewport(svg, output)?;
    if output.background.as_str() == "transparent" {
        return Ok(source.normalized_svg.into_bytes());
    }
    let mut root = Element::read(&source.normalized_svg)?;
    let [x, y, width, height] = source.view_box;
    let background = Element {
        name: "rect".into(),
        attrs: BTreeMap::from([
            ("x".into(), x.to_string()),
            ("y".into(), y.to_string()),
            ("width".into(), width.to_string()),
            ("height".into(), height.to_string()),
            (
                "fill".into(),
                output.background.as_str().to_ascii_lowercase(),
            ),
            ("fill-opacity".into(), "1".into()),
            ("stroke".into(), "none".into()),
        ]),
        children: Vec::new(),
    };
    // Canvas background must not inherit the artwork's root transform, opacity or clip.
    let mut effects = BTreeMap::new();
    for key in ["transform", "opacity", "clip-path"] {
        if let Some(value) = root.attrs.remove(key) {
            effects.insert(key.into(), value);
        }
    }
    if effects.is_empty() {
        root.children.insert(0, background);
    } else {
        let mut definitions = Vec::new();
        let mut artwork = Vec::new();
        for child in root.children {
            if child.name == "defs" {
                definitions.push(child);
            } else {
                artwork.push(child);
            }
        }
        definitions.push(background);
        definitions.push(Element {
            name: "g".into(),
            attrs: effects,
            children: artwork,
        });
        root.children = definitions;
    }
    let mut writer = quick_xml::Writer::new(Vec::new());
    root.write(&mut writer)?;
    let exported = import_svg(&writer.into_inner())?;
    Ok(exported.normalized_svg.into_bytes())
}
