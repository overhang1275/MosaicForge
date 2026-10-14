use mosaicforge_core::template::import_svg;

fn svg(body: &str) -> String {
    format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"80\">{body}</svg>")
}

fn reject(bytes: &[u8]) {
    let error = import_svg(bytes).unwrap_err();
    assert_eq!(error.code(), "invalid_svg");
}

#[test]
fn fixtures_expose_geometry_and_layers() {
    let basic = import_svg(include_bytes!("../../../tests/fixtures/basic.svg")).unwrap();
    assert_eq!((basic.width, basic.height), (100.0, 80.0));
    assert_eq!(basic.view_box, [0.0, 0.0, 100.0, 80.0]);
    assert_eq!(basic.hash.len(), 64);
    let layers = import_svg(include_bytes!("../../../tests/fixtures/layers.svg")).unwrap();
    assert_eq!(layers.layer_ids, ["back", "front"]);
    assert!(import_svg(include_bytes!("../../../tests/fixtures/clip.svg")).is_ok());
}

#[test]
fn svg_rejects_active_external_and_excessive_content() {
    for body in [
        "<script>alert(1)</script>",
        "<foreignObject/>",
        "<style>rect{fill:red}</style>",
        "<rect width=\"10\" height=\"10\" onclick=\"alert(1)\"/>",
        "<rect width=\"10\" height=\"10\" style=\"fill:red\"/>",
        "<image href=\"file:///secret\"/>",
        "<image href=\"data:image/png;base64,AAAA\"/>",
        "<use href=\"#x\"/>",
        "<text>font</text>",
        "<animate/>",
        "<filter/>",
        "<rect width=\"10\" height=\"10\" fill=\"url(http://example.com/a)\"/>",
        "<rect width=\"10\" height=\"10\" clip-path=\"url(&#102;ile:///a)\"/>",
        "<rect width=\"10\" height=\"10\" clip-path=\"url(#missing)\"/>",
        "<x:rect width=\"10\" height=\"10\"/>",
        "<g xmlns=\"http://example.com/evil\"><rect width=\"10\" height=\"10\"/></g>",
        "<rect width=\"10\" height=\"10\" bogus=\"1\"/>",
        "<rect width=\"10\" height=\"10\" id=\"same\"/><circle r=\"1\" id=\"same\"/>",
        "<defs><clipPath id=\"a\" clip-path=\"url(#b)\"><rect width=\"10\" height=\"10\"/></clipPath><clipPath id=\"b\" clip-path=\"url(#a)\"><rect width=\"10\" height=\"10\"/></clipPath></defs><rect width=\"10\" height=\"10\" clip-path=\"url(#a)\"/>",
    ] {
        reject(svg(body).as_bytes());
    }
    reject(b"<!DOCTYPE svg [<!ENTITY x SYSTEM 'file:///secret'>]><svg>&x;</svg>");
    reject(b"<?xml-stylesheet href='file:///secret'?><svg/>");
    reject(b"<svg><g></svg>");
    reject(b"<svg/><svg/>");
    reject(b"<svg xmlns='http://www.w3.org/2000/svg' width='1' width='2' height='2'/>");
    reject(&[0xff, 0xfe]);
    reject(svg("&unknown;").as_bytes());
}

#[test]
fn geometric_inputs_are_validated_before_usvg() {
    for body in [
        "<rect width=\"-1\" height=\"1\"/>",
        "<circle r=\"0\"/>",
        "<path d=\"M0 0 LNaN 1\"/>",
        "<path d=\"M0 0 L1e100 1\"/>",
        "<path d=\"not a path\"/>",
        "<g transform=\"rotate(bad)\"><rect width=\"1\" height=\"1\"/></g>",
        "<polygon points=\"0,0 bad\"/>",
        "<rect width=\"1\" height=\"1\" opacity=\"2\"/>",
    ] {
        reject(svg(body).as_bytes());
    }
    reject(b"<svg xmlns='http://www.w3.org/2000/svg' width='0' height='10'/>");
    reject(b"<svg xmlns='http://www.w3.org/2000/svg' width='1' height='10' viewBox='0 0 -1 10'/>");
}

#[test]
fn canonical_hash_ignores_attribute_order_comments_and_empty_tag_form() {
    let first = import_svg(
        svg("<rect x=\"1\" y=\"2\" width=\"10\" height=\"20\" fill=\"#123456\"/>").as_bytes(),
    )
    .unwrap();
    let second = import_svg(svg("<!--original--><rect fill=\"#123456\" height=\"20\" width=\"10\" y=\"2\" x=\"1\"></rect>").as_bytes()).unwrap();
    assert_eq!(first.normalized_svg, second.normalized_svg);
    assert_eq!(first.hash, second.hash);
    let roundtrip = import_svg(first.normalized_svg.as_bytes()).unwrap();
    assert_eq!(first.hash, roundtrip.hash);
    let changed = import_svg(
        svg("<rect x=\"1\" y=\"2\" width=\"11\" height=\"20\" fill=\"#123456\"/>").as_bytes(),
    )
    .unwrap();
    assert_ne!(first.hash, changed.hash);
}

#[test]
fn byte_limit_is_checked_at_and_above_five_mib() {
    let head = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"80\"><!--";
    let tail = "--><rect width=\"1\" height=\"1\"/></svg>";
    let at = format!(
        "{head}{}{tail}",
        "x".repeat(5 * 1024 * 1024 - head.len() - tail.len())
    );
    assert!(import_svg(at.as_bytes()).is_ok());
    reject(format!(" {at}").as_bytes());
}

#[test]
fn element_limit_is_checked_at_and_above_twenty_thousand() {
    let body = format!("<rect width=\"1\" height=\"1\"/>{}", "<g/>".repeat(19_998));
    assert!(import_svg(svg(&body).as_bytes()).is_ok());
    reject(svg(&format!("{body}<g/>")).as_bytes());
}

#[test]
fn depth_limit_is_checked_at_and_above_sixty_four() {
    let nested = |depth| {
        svg(&format!(
            "{}<rect width=\"1\" height=\"1\"/>{}",
            "<g>".repeat(depth),
            "</g>".repeat(depth)
        ))
    };
    assert!(import_svg(nested(62).as_bytes()).is_ok());
    reject(nested(63).as_bytes());
}

#[test]
fn segment_limit_is_checked_at_and_above_one_hundred_thousand() {
    let path = |lines| {
        svg(&format!(
            "<path d=\"M0 0 {}\" fill=\"none\" stroke=\"#123456\"/>",
            "L1 1 ".repeat(lines)
        ))
    };
    let at = import_svg(path(99_999).as_bytes()).unwrap();
    assert_eq!(at.segment_count, 100_000);
    reject(path(100_000).as_bytes());
}

#[test]
fn malformed_declarations_and_xml_characters_are_not_normalized_away() {
    let valid = svg("<rect width=\"1\" height=\"1\"/>");
    for prefix in [
        "<?xml version='1.0'?><?xml version='1.0'?>",
        " <!--before--><?xml version='1.0'?>",
        "<?xml version='1.0' standalone='maybe'?>",
        "<?xml version='1.0' unknown='x'?>",
        "<?xml version='1.1'?>",
        "<?xml version='1.0' encoding='UTF-16'?>",
    ] {
        reject(format!("{prefix}{valid}").as_bytes());
    }
    reject(svg("\u{000b}<rect width=\"1\" height=\"1\"/>").as_bytes());
    reject(svg("<!--\u{ffff}--><rect width=\"1\" height=\"1\"/>").as_bytes());
    assert!(
        import_svg(
            format!("<?xml version='1.0' encoding='UTF-8' standalone='yes'?>{valid}").as_bytes()
        )
        .is_ok()
    );
}

#[test]
fn reference_expansion_and_accumulated_transforms_are_bounded() {
    let chain = |length: usize, branching: bool| {
        let mut clips = String::from("<defs>");
        for index in 0..length {
            clips.push_str(&format!("<clipPath id=\"c{index}\">"));
            for _ in 0..if branching { 2 } else { 1 } {
                let reference = if index + 1 < length {
                    format!(" clip-path=\"url(#c{})\"", index + 1)
                } else {
                    String::new()
                };
                clips.push_str(&format!("<rect width=\"1\" height=\"1\"{reference}/>"));
            }
            clips.push_str("</clipPath>");
        }
        clips.push_str("</defs><rect width=\"1\" height=\"1\" clip-path=\"url(#c0)\"/>");
        svg(&clips)
    };
    assert!(import_svg(chain(3, false).as_bytes()).is_ok());
    reject(chain(40, false).as_bytes());
    reject(chain(14, true).as_bytes());
    reject(svg("<g transform=\"scale(10000)\"><g transform=\"scale(10000)\"><rect width=\"1\" height=\"1\"/></g></g>").as_bytes());
}

#[test]
fn all_supported_shapes_and_absolute_units_are_accepted() {
    let shapes = "<path d=\"M1 1 Q2 3 4 5 C5 6 7 8 9 10 A2 2 0 0 1 11 12 Z\"/><rect width=\"10\" height=\"20\" rx=\"2\"/><circle r=\"2\"/><ellipse rx=\"2\" ry=\"3\"/><line x1=\"0\" y1=\"0\" x2=\"10\" y2=\"10\"/><polyline points=\"0,0 1,1\"/><polygon points=\"0,0 10,0 10,10\"/>";
    assert!(import_svg(svg(shapes).as_bytes()).is_ok());
    let unit = import_svg(b"<svg xmlns='http://www.w3.org/2000/svg' width='1in' height='1in'><rect width='1cm' height='1mm'/></svg>").unwrap();
    assert_eq!((unit.width, unit.height), (96.0, 96.0));
    let viewbox = import_svg(b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='-10 -20 100 80'><rect width='1' height='1'/></svg>").unwrap();
    assert_eq!(viewbox.view_box, [-10.0, -20.0, 100.0, 80.0]);
}

#[test]
fn backend_unsupported_or_underflowing_geometry_is_rejected() {
    for body in [
        "<defs><clipPath id=\"c\"><g><rect width=\"10\" height=\"10\"/></g></clipPath></defs><rect width=\"50\" height=\"50\" clip-path=\"url(#c)\"/>",
        "<path d=\"M0 0 M1 1\"/>",
        "<rect width=\"1e-50\" height=\"1\"/>",
        "<g transform=\"scale(1e-50)\"><rect width=\"1\" height=\"1\"/></g>",
    ] {
        reject(svg(body).as_bytes());
    }
}

#[test]
fn xml_whitespace_and_declaration_order_are_strict() {
    let valid = svg("<rect width=\"1\" height=\"1\"/>");
    reject(format!("\u{00a0}{valid}").as_bytes());
    reject(format!("{valid}\u{2003}").as_bytes());
    reject(format!("<?xml version='1.0' standalone='yes' encoding='UTF-8'?>{valid}").as_bytes());
    assert!(import_svg(format!(" \t\r\n{valid}\n").as_bytes()).is_ok());
}

#[test]
fn physical_unit_underflow_and_near_zero_transforms_are_rejected() {
    reject(svg("<rect width=\"1e-46in\" height=\"1\"/>").as_bytes());
    reject(svg("<g transform=\"scale(1e-10)\"><rect width=\"1\" height=\"1\"/></g>").as_bytes());
    reject(svg("<g transform=\"scale(1e-30)\"><rect width=\"1\" height=\"1\"/></g>").as_bytes());
    assert!(
        import_svg(
            svg("<g transform=\"scale(0)\"><rect width=\"1\" height=\"1\"/></g>").as_bytes()
        )
        .is_ok()
    );
}

#[test]
fn local_near_zero_transform_cannot_be_hidden_by_parent_scale() {
    reject(svg("<g transform=\"scale(1000000)\"><g transform=\"scale(1e-10)\"><rect width=\"1\" height=\"1\"/></g></g>").as_bytes());
}
