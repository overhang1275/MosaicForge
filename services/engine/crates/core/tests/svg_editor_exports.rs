use mosaicforge_core::template::import_svg;

#[test]
fn declarative_class_export_matches_explicit_presentation_attributes() {
    let exported=br##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100" height="100"><defs><style>.cls-1{fill:none;stroke:#00a5f1;stroke-width:0.09;stroke-miterlimit:10}</style></defs><title>Editor export</title><path class="cls-1" data-name="art" d="M0 0L100 100"/></svg>"##;
    let explicit=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs/><path fill="none" stroke="#00a5f1" stroke-width="0.09" stroke-miterlimit="10" d="M0 0L100 100"/></svg>"##;
    let actual = import_svg(exported).unwrap();
    assert_eq!(
        actual.normalized_svg,
        import_svg(explicit).unwrap().normalized_svg
    );
    assert_eq!(actual.hash, import_svg(explicit).unwrap().hash);
    assert!(!actual.normalized_svg.contains("class="));
}

#[test]
fn stylesheet_source_order_overrides_presentation_and_class_attribute_order() {
    let source=br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><style>.a{fill:#ff0000}.b{fill:#00ff00}.a{stroke:#0000ff}</style><rect class="b a" fill="#ffffff" width="10" height="10"/></svg>"##;
    let result = import_svg(source).unwrap();
    assert!(result.normalized_svg.contains("fill=\"#00ff00\""));
    assert!(result.normalized_svg.contains("stroke=\"#0000ff\""));
}

#[test]
fn compatibility_does_not_admit_active_css_external_references_or_unknown_classes() {
    for body in [
        "<style>.a{fill:url(https://example.com/a)}</style><rect class=\"a\" width=\"10\" height=\"10\"/>",
        "<style>@import 'https://example.com/a';</style>",
        "<style>.a{onclick:evil}</style>",
        "<style>rect{fill:red}</style><rect width=\"10\" height=\"10\"/>",
        "<style>.a{fill:red!important}</style>",
        "<style>.a{fill:red}</style><rect class=\"missing\" width=\"10\" height=\"10\"/>",
        "<style>.a{fill:red}</style><script>evil</script>",
        "<title><script>evil</script></title>",
        "<title onclick=\"evil\">title</title>",
        "<title><?evil instruction?></title>",
        "<title><![CDATA[hidden]]></title>",
        "<title>&unknown;</title>",
        "<style>.a{fill:red}<script/></style>",
        "<style><![CDATA[.a{fill:red}]]></style>",
        "<style>.a:hover{fill:red}</style>",
        "<style>.a,.b{fill:red}</style>",
        "<style>.a{fill:url(#paint);fill:red}</style>",
        "<style>.a{fill:red}</style><rect class=\"a\" onclick=\"evil\" width=\"10\" height=\"10\"/>",
        "<g><title>nested</title></g>",
        "<g><style>.a{fill:red}</style></g>",
        "<rect style=\"fill:red\" width=\"10\" height=\"10\"/>",
        "<rect xlink:href=\"file:///secret\" width=\"10\" height=\"10\"/>",
    ] {
        let source = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"10\" height=\"10\">{body}</svg>"
        );
        assert!(import_svg(source.as_bytes()).is_err(), "{body}");
    }
}

#[test]
fn compatibility_counts_removed_metadata_towards_resource_limits() {
    for body in [
        format!("<style>{}</style>", " ".repeat(64 * 1024 + 1)),
        format!("<style>{}</style>", ".a{fill:red}".repeat(1025)),
        "<title/>".repeat(20_000),
        format!("<style>.{}{}</style>", "a".repeat(257), "{fill:red}"),
        format!("{}{}", "<g>".repeat(64), "</g>".repeat(64)),
    ] {
        let source = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"10\" height=\"10\">{body}</svg>"
        );
        assert!(import_svg(source.as_bytes()).is_err());
    }
}

#[test]
fn compatibility_applies_later_styles_and_preserves_safe_clip_references() {
    let source = br##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><defs><clipPath id="clip"><rect width="10" height="10"/></clipPath></defs><g class="a" clip-path="url(#clip)"><rect width="10" height="10"/></g><style type="text/css">.a{fill:#ff0000;fill:#0000ff}</style></svg>"##;
    let result = import_svg(source).unwrap();
    assert!(result.normalized_svg.contains("fill=\"#0000ff\""));
    assert!(result.normalized_svg.contains("clip-path=\"url(#clip)\""));
    assert_eq!(
        result.hash,
        import_svg(result.normalized_svg.as_bytes()).unwrap().hash
    );
}
