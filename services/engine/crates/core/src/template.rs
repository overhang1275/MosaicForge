use crate::EngineError;
use crate::svg_values::{bounded, compose, invalid, numbers, validate};
use quick_xml::{
    Reader,
    events::{BytesEnd, BytesStart, Event},
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashMap, VecDeque};

pub const MAX_SVG_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug)]
pub struct Template {
    pub width: f64,
    pub height: f64,
    pub view_box: [f64; 4],
    pub layer_ids: Vec<String>,
    pub normalized_svg: String,
    pub hash: String,
    pub segment_count: usize,
}

/// Prepare a safe pixel viewport with centered cover framing, without adding a background.
pub fn prepare_viewport(
    bytes: &[u8],
    output: &crate::contract::OutputSpec,
) -> Result<Template, EngineError> {
    crate::validate_output(output)?;
    let source = import_svg(bytes)?;
    let mut root = crate::appearance::Element::read(&source.normalized_svg)?;
    root.attrs.entry("viewBox".into()).or_insert_with(|| {
        format!(
            "{} {} {} {}",
            source.view_box[0], source.view_box[1], source.view_box[2], source.view_box[3]
        )
    });
    root.attrs
        .insert("width".into(), output.width.get().to_string());
    root.attrs
        .insert("height".into(), output.height.get().to_string());
    root.attrs
        .insert("preserveAspectRatio".into(), "xMidYMid slice".into());
    let mut writer = quick_xml::Writer::new(Vec::new());
    root.write(&mut writer)?;
    import_svg(&writer.into_inner())
}

struct Node {
    children: Vec<usize>,
    segments: usize,
}

struct Frame {
    name: String,
    index: usize,
    transform: [f64; 6],
}

pub fn import_svg(bytes: &[u8]) -> Result<Template, EngineError> {
    match import_svg_strict(bytes) {
        Ok(template) => Ok(template),
        Err(original) => {
            let normalized = crate::svg_compat::normalize(bytes).map_err(|_| original)?;
            import_svg_strict(&normalized)
        }
    }
}

fn import_svg_strict(bytes: &[u8]) -> Result<Template, EngineError> {
    if bytes.len() > MAX_SVG_BYTES {
        return Err(invalid("SVG byte limit exceeded"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("SVG must be UTF-8"))?;
    if text.chars().any(|c| {
        (c < ' ' && !matches!(c, '\t' | '\n' | '\r')) || matches!(c, '\u{fffe}' | '\u{ffff}')
    }) {
        return Err(invalid("invalid XML character"));
    }
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_end_names = true;
    reader.config_mut().check_comments = true;
    let mut writer = quick_xml::Writer::new(Vec::new());
    let mut stack: Vec<Frame> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut ids = HashMap::new();
    let mut references = Vec::new();
    let mut layers = Vec::new();
    let mut root_attrs = BTreeMap::new();
    let mut segments = 0;
    let mut event_number = 0;
    loop {
        let event = reader
            .read_event()
            .map_err(|_| invalid("malformed SVG XML"))?;
        event_number += 1;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                let name = element.name().as_ref().to_owned();
                if !matches!(
                    name.as_str(),
                    "svg"
                        | "g"
                        | "defs"
                        | "clipPath"
                        | "path"
                        | "rect"
                        | "circle"
                        | "ellipse"
                        | "line"
                        | "polyline"
                        | "polygon"
                ) {
                    return Err(invalid("unsupported SVG element or namespace prefix"));
                }
                if stack.is_empty() {
                    if !nodes.is_empty() || name != "svg" {
                        return Err(invalid("SVG must have exactly one root"));
                    }
                } else {
                    if name == "svg" {
                        return Err(invalid("nested SVG roots are not supported"));
                    }
                    let parent = stack.last().unwrap().name.as_str();
                    if !matches!(parent, "svg" | "g" | "defs" | "clipPath")
                        || (name == "clipPath" && parent != "defs")
                        || (name == "defs" && parent != "svg")
                        || (parent == "clipPath"
                            && !matches!(
                                name.as_str(),
                                "path"
                                    | "rect"
                                    | "circle"
                                    | "ellipse"
                                    | "line"
                                    | "polyline"
                                    | "polygon"
                            ))
                    {
                        return Err(invalid("unsupported SVG element nesting"));
                    }
                }
                if stack.len() + 1 > 64 || nodes.len() >= 20_000 {
                    return Err(invalid("SVG depth or element limit exceeded"));
                }
                let mut attrs = BTreeMap::new();
                for attribute in element.attributes() {
                    let attribute =
                        attribute.map_err(|_| invalid("invalid or duplicate XML attribute"))?;
                    let key = attribute.key.as_ref().to_owned();
                    let value = attribute
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|_| invalid("invalid XML attribute value"))?
                        .trim()
                        .to_owned();
                    if key.contains(':')
                        || key == "xmlns"
                            && (name != "svg" || value != "http://www.w3.org/2000/svg")
                    {
                        return Err(invalid("unsupported XML namespace"));
                    }
                    if value.chars().any(|c| c.is_control() && !c.is_whitespace()) {
                        return Err(invalid("invalid XML control character"));
                    }
                    attrs.insert(key, value);
                }
                if name == "svg" {
                    if attrs.get("xmlns").map(String::as_str) != Some("http://www.w3.org/2000/svg")
                    {
                        return Err(invalid("SVG default namespace is required"));
                    }
                    if !attrs.contains_key("viewBox")
                        && !(attrs.contains_key("width") && attrs.contains_key("height"))
                    {
                        return Err(invalid("SVG dimensions or viewBox are required"));
                    }
                    root_attrs = attrs.clone();
                }
                let own_segments = validate(&name, &attrs)?;
                segments += own_segments;
                if segments > 100_000 {
                    return Err(invalid("SVG segment limit exceeded"));
                }
                let transform = compose(
                    stack
                        .last()
                        .map_or([1.0, 0.0, 0.0, 1.0, 0.0, 0.0], |f| f.transform),
                    attrs.get("transform"),
                )?;
                let index = nodes.len();
                nodes.push(Node {
                    children: Vec::new(),
                    segments: own_segments,
                });
                if let Some(parent) = stack.last() {
                    nodes[parent.index].children.push(index);
                }
                if let Some(id) = attrs.get("id") {
                    if id.is_empty()
                        || id.len() > 256
                        || !id.bytes().enumerate().all(|(i, c)| {
                            c.is_ascii_alphabetic()
                                || c == b'_'
                                || (i > 0 && (c.is_ascii_digit() || matches!(c, b'-' | b'.')))
                        })
                    {
                        return Err(invalid("invalid SVG identifier"));
                    }
                    if ids
                        .insert(id.clone(), (index, name == "clipPath"))
                        .is_some()
                    {
                        return Err(invalid("duplicate SVG identifier"));
                    }
                }
                if stack.len() == 1 && name == "g" {
                    layers.push(
                        attrs
                            .get("id")
                            .cloned()
                            .unwrap_or_else(|| format!("layer:{}", layers.len())),
                    );
                }
                if let Some(reference) = attrs.get("clip-path")
                    && reference != "none"
                {
                    let target = reference
                        .strip_prefix("url(#")
                        .and_then(|s| s.strip_suffix(')'))
                        .ok_or(invalid("clip path must be an internal reference"))?;
                    references.push((index, target.to_owned()));
                }
                let mut normalized = BytesStart::new(name.as_str());
                for (key, value) in &attrs {
                    normalized.push_attribute((key.as_str(), value.as_str()));
                }
                writer
                    .write_event(Event::Start(normalized))
                    .map_err(|_| invalid("SVG normalization failed"))?;
                if empty {
                    writer
                        .write_event(Event::End(BytesEnd::new(name)))
                        .map_err(|_| invalid("SVG normalization failed"))?;
                } else {
                    stack.push(Frame {
                        name,
                        index,
                        transform,
                    });
                }
            }
            Event::End(element) => {
                let frame = stack.pop().ok_or(invalid("unexpected XML closing tag"))?;
                if element.name().as_ref() != frame.name {
                    return Err(invalid("mismatched XML closing tag"));
                }
                writer
                    .write_event(Event::End(BytesEnd::new(frame.name)))
                    .map_err(|_| invalid("SVG normalization failed"))?;
            }
            Event::Text(value) => {
                let content = value.as_ref();
                if !content
                    .chars()
                    .all(|c| matches!(c, ' ' | '\t' | '\r' | '\n'))
                {
                    return Err(invalid("SVG text content is not supported"));
                }
            }
            Event::Comment(_) => (),
            Event::Decl(declaration) => {
                if event_number != 1
                    || declaration
                        .version()
                        .map_err(|_| invalid("invalid XML declaration"))?
                        .as_ref()
                        != "1.0"
                {
                    return Err(invalid("unsupported XML declaration"));
                }
                let content = BytesStart::from_content(declaration.as_ref(), 3);
                let mut previous_rank = 0;
                for attribute in content.attributes() {
                    let attribute =
                        attribute.map_err(|_| invalid("invalid XML declaration attributes"))?;
                    let rank = match attribute.key.as_ref() {
                        "version" => 1,
                        "encoding" => 2,
                        "standalone" => 3,
                        _ => return Err(invalid("unsupported XML declaration attribute")),
                    };
                    if rank <= previous_rank {
                        return Err(invalid("invalid XML declaration attribute order"));
                    }
                    previous_rank = rank;
                }
                if let Some(encoding) = declaration.encoding()
                    && !encoding
                        .map_err(|_| invalid("invalid XML encoding"))?
                        .eq_ignore_ascii_case("utf-8")
                {
                    return Err(invalid("SVG must declare UTF-8"));
                }
                if let Some(standalone) = declaration.standalone()
                    && !matches!(
                        standalone
                            .map_err(|_| invalid("invalid XML standalone"))?
                            .as_ref(),
                        "yes" | "no"
                    )
                {
                    return Err(invalid("invalid XML standalone"));
                }
            }
            Event::Eof => break,
            _ => {
                return Err(invalid(
                    "DTD, entities, processing instructions or CDATA are not supported",
                ));
            }
        }
    }
    if !stack.is_empty() || nodes.is_empty() {
        return Err(invalid("incomplete SVG XML"));
    }
    for (source, target) in references {
        let (target, is_clip) = ids
            .get(&target)
            .ok_or(invalid("unresolved SVG clip reference"))?;
        if !is_clip {
            return Err(invalid("clip reference must target clipPath"));
        }
        nodes[source].children.push(*target);
    }
    validate_graph(&nodes)?;
    let normalized_svg =
        String::from_utf8(writer.into_inner()).map_err(|_| invalid("SVG normalization failed"))?;
    let options = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(&normalized_svg, &options)
        .map_err(|_| invalid("SVG normalization could not build geometry"))?;
    let width = f64::from(tree.size().width());
    let height = f64::from(tree.size().height());
    bounded(&[width, height])?;
    if width <= 0.0 || height <= 0.0 {
        return Err(invalid("SVG dimensions must be positive"));
    }
    let view_box = match root_attrs.get("viewBox") {
        Some(value) => numbers(value)?
            .try_into()
            .map_err(|_| invalid("invalid SVG viewBox"))?,
        None => [0.0, 0.0, width, height],
    };
    let hash = format!("{:x}", Sha256::digest(normalized_svg.as_bytes()));
    Ok(Template {
        width,
        height,
        view_box,
        layer_ids: layers,
        normalized_svg,
        hash,
        segment_count: segments,
    })
}

fn validate_graph(nodes: &[Node]) -> Result<(), EngineError> {
    let mut incoming = vec![0; nodes.len()];
    for node in nodes {
        for &child in &node.children {
            incoming[child] += 1;
        }
    }
    let mut ready: VecDeque<_> = incoming
        .iter()
        .enumerate()
        .filter_map(|(i, &count)| (count == 0).then_some(i))
        .collect();
    let mut order = Vec::new();
    let mut depth = vec![1; nodes.len()];
    while let Some(index) = ready.pop_front() {
        order.push(index);
        for &child in &nodes[index].children {
            depth[child] = depth[child].max(depth[index] + 1);
            if depth[child] > 64 {
                return Err(invalid("expanded SVG reference depth limit exceeded"));
            }
            incoming[child] -= 1;
            if incoming[child] == 0 {
                ready.push_back(child);
            }
        }
    }
    if order.len() != nodes.len() {
        return Err(invalid("cyclic SVG clip references"));
    }
    let mut elements = vec![0usize; nodes.len()];
    let mut segments = vec![0usize; nodes.len()];
    for index in order.into_iter().rev() {
        elements[index] = 1;
        segments[index] = nodes[index].segments;
        for &child in &nodes[index].children {
            elements[index] = elements[index].saturating_add(elements[child]);
            segments[index] = segments[index].saturating_add(segments[child]);
        }
        if elements[index] > 20_000 || segments[index] > 100_000 {
            return Err(invalid("expanded SVG reference budget exceeded"));
        }
    }
    Ok(())
}
