//! Bounded editor-export compatibility; the strict importer remains the safety boundary.
use crate::{
    EngineError,
    svg_values::{invalid, validate},
    template::MAX_SVG_BYTES,
};
use quick_xml::{
    Reader, Writer,
    events::{BytesEnd, BytesStart, Event},
};
use std::collections::BTreeMap;

enum Output {
    Start(String, BTreeMap<String, String>),
    End(String),
    Declaration(Event<'static>),
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.bytes().enumerate().all(|(i, c)| {
            c.is_ascii_alphabetic() || c == b'_' || (i > 0 && (c.is_ascii_digit() || c == b'-'))
        })
}

fn stylesheet(
    css: &str,
    rules: &mut Vec<(String, BTreeMap<String, String>)>,
) -> Result<(), EngineError> {
    let mut rest = css.trim();
    while !rest.is_empty() {
        if rules.len() >= 1024 {
            return Err(invalid("SVG CSS rule limit exceeded"));
        }
        let (selector, body) = rest
            .split_once('{')
            .ok_or(invalid("invalid SVG CSS rule"))?;
        let class = selector
            .trim()
            .strip_prefix('.')
            .ok_or(invalid("only simple SVG class selectors are supported"))?;
        if !identifier(class) {
            return Err(invalid("invalid SVG CSS class"));
        }
        let (body, tail) = body
            .split_once('}')
            .ok_or(invalid("invalid SVG CSS rule"))?;
        let mut attrs = BTreeMap::new();
        for declaration in body.split(';') {
            if declaration.trim().is_empty() {
                continue;
            }
            let (key, value) = declaration
                .split_once(':')
                .ok_or(invalid("invalid SVG CSS declaration"))?;
            let key = key.trim();
            let value = value.trim();
            if !matches!(
                key,
                "fill"
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
            ) {
                return Err(invalid("unsupported SVG CSS property"));
            }
            // Validate every declaration, including one overridden by a later declaration.
            let declaration = BTreeMap::from([(key.to_owned(), value.to_owned())]);
            validate("g", &declaration)?;
            attrs.extend(declaration);
        }
        if attrs.is_empty() {
            return Err(invalid("empty SVG CSS rule"));
        }
        rules.push((class.to_owned(), attrs));
        rest = tail.trim();
    }
    Ok(())
}

pub(super) fn normalize(bytes: &[u8]) -> Result<Vec<u8>, EngineError> {
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
    let mut stack: Vec<String> = Vec::new();
    let mut output = Vec::new();
    let mut rules = Vec::new();
    let mut css = String::new();
    let mut css_bytes = 0;
    let mut elements = 0;
    let mut events = 0;
    loop {
        let event = reader
            .read_event()
            .map_err(|_| invalid("malformed SVG XML"))?;
        events += 1;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(element) | Event::Empty(element) => {
                if matches!(stack.last().map(String::as_str), Some("title" | "style")) {
                    return Err(invalid("SVG metadata must contain text only"));
                }
                elements += 1;
                if elements > 20_000 || stack.len() >= 64 {
                    return Err(invalid("SVG depth or element limit exceeded"));
                }
                let name = element.name().as_ref().to_owned();
                let metadata = matches!(name.as_str(), "title" | "style");
                if name == "title" && !(stack.len() == 1 && stack[0] == "svg")
                    || name == "style"
                        && !(stack.len() == 1 && stack[0] == "svg"
                            || stack.len() == 2 && stack[0] == "svg" && stack[1] == "defs")
                {
                    return Err(invalid("unsupported SVG metadata nesting"));
                }
                let mut attrs = BTreeMap::new();
                for attribute in element.attributes() {
                    let attribute =
                        attribute.map_err(|_| invalid("invalid or duplicate XML attribute"))?;
                    let key = attribute.key.as_ref();
                    let value = attribute
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|_| invalid("invalid XML attribute value"))?;
                    let value = value.trim();
                    if key == "xmlns:xlink"
                        && stack.is_empty()
                        && name == "svg"
                        && value == "http://www.w3.org/1999/xlink"
                    {
                        continue;
                    }
                    if key == "data-name" {
                        continue;
                    }
                    if metadata {
                        if name == "style" && key == "type" && value == "text/css" {
                            continue;
                        }
                        return Err(invalid("unsupported SVG metadata attribute"));
                    }
                    attrs.insert(key.to_owned(), value.to_owned());
                }
                if !metadata {
                    output.push(Output::Start(name.clone(), attrs));
                }
                if empty {
                    if !metadata {
                        output.push(Output::End(name));
                    }
                } else {
                    stack.push(name);
                }
            }
            Event::End(element) => {
                let name = stack.pop().ok_or(invalid("unexpected XML closing tag"))?;
                if element.name().as_ref() != name {
                    return Err(invalid("mismatched XML closing tag"));
                }
                match name.as_str() {
                    "style" => {
                        stylesheet(&css, &mut rules)?;
                        css.clear();
                    }
                    "title" => (),
                    _ => output.push(Output::End(name)),
                }
            }
            Event::Text(value) => match stack.last().map(String::as_str) {
                Some("style") => {
                    css_bytes += value.as_ref().len();
                    if css_bytes > 64 * 1024 {
                        return Err(invalid("SVG CSS byte limit exceeded"));
                    }
                    css.push_str(value.as_ref());
                }
                Some("title") => (),
                _ if value
                    .as_ref()
                    .chars()
                    .all(|c| matches!(c, ' ' | '\t' | '\r' | '\n')) => {}
                _ => return Err(invalid("SVG text content is not supported")),
            },
            Event::Decl(declaration) if events == 1 => {
                output.push(Output::Declaration(Event::Decl(declaration.into_owned())))
            }
            Event::Comment(_) => (),
            Event::Eof => break,
            _ => {
                return Err(invalid(
                    "DTD, entities, processing instructions or CDATA are not supported",
                ));
            }
        }
    }
    if !stack.is_empty() {
        return Err(invalid("incomplete SVG XML"));
    }
    let mut writer = Writer::new(Vec::new());
    for event in output {
        let event = match event {
            Output::Start(name, mut attrs) => {
                if let Some(classes) = attrs.remove("class") {
                    if classes.len() > 4096 {
                        return Err(invalid("SVG class list limit exceeded"));
                    }
                    let classes: Vec<_> = classes.split_ascii_whitespace().collect();
                    if classes.is_empty()
                        || classes.len() > 64
                        || classes.iter().any(|class| {
                            !identifier(class) || !rules.iter().any(|(known, _)| known == class)
                        })
                    {
                        return Err(invalid("unknown or invalid SVG class"));
                    }
                    for (class, declarations) in &rules {
                        if classes.contains(&class.as_str()) {
                            attrs.extend(declarations.clone());
                        }
                    }
                }
                let mut element = BytesStart::new(name);
                for (key, value) in &attrs {
                    element.push_attribute((key.as_str(), value.as_str()));
                }
                Event::Start(element)
            }
            Output::End(name) => Event::End(BytesEnd::new(name)),
            Output::Declaration(event) => event,
        };
        writer
            .write_event(event)
            .map_err(|_| invalid("SVG normalization failed"))?;
        if writer.get_ref().len() > MAX_SVG_BYTES {
            return Err(invalid("expanded SVG byte limit exceeded"));
        }
    }
    Ok(writer.into_inner())
}
