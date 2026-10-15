use mosaicforge_core::contract::PdfJobSpec;
use mosaicforge_cpu::pdf::geometry::{Node, optimize_page};
use serde_json::{Value, json};
fn job() -> PdfJobSpec {
    let generation: Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/identity.json")).unwrap();
    serde_json::from_value(json!({"schemaVersion":"1","generation":generation,"pdf":{"exporterVersion":"1","trimWidthMm":100,"trimHeightMm":100,"bleedMm":0}})).unwrap()
}
fn points(node: &Node) -> Vec<[f64; 2]> {
    match node {
        Node::Paint { contours, .. } => contours.iter().flatten().copied().collect(),
        Node::Group { children, .. } => children.iter().flat_map(points).collect(),
    }
}
#[test]
fn exterior_geometry_is_removed_and_crossing_strokes_are_really_cut() {
    let svg=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect x="200" y="200" width="20" height="20"/><path d="M-20 50L120 50" stroke="red" stroke-width="10" stroke-linecap="round" fill="none"/></svg>"##;
    let p = optimize_page(svg, &job()).unwrap();
    let pts = points(&p.root);
    assert!(!pts.is_empty());
    assert!(
        pts.iter()
            .all(|p| p[0] >= -0.0001 && p[1] >= -0.0001 && p[0] <= 283.4647 && p[1] <= 283.4647)
    );
    assert!(p.stats.discarded_paints >= 1);
    assert!(pts.iter().any(|p| p[0].abs() < 0.001));
}
#[test]
fn holes_clips_and_group_opacity_survive_optimization() {
    let svg=br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><clipPath id="c"><path d="M0 0H100V100H0Z M25 25H75V75H25Z" clip-rule="evenodd"/></clipPath></defs><g opacity="0.5" clip-path="url(#c)"><rect width="100" height="100"/><circle cx="50" cy="50" r="45" fill="red"/></g></svg>"##;
    let p = optimize_page(svg, &job()).unwrap();
    fn has_group(n: &Node) -> bool {
        match n {
            Node::Group { opacity, children } => *opacity == 0.5 || children.iter().any(has_group),
            _ => false,
        }
    }
    fn has_hole(n: &Node) -> bool {
        match n {
            Node::Paint { contours, .. } => contours.len() >= 2,
            Node::Group { children, .. } => children.iter().any(has_hole),
        }
    }
    assert!(has_group(&p.root));
    assert!(has_hole(&p.root));
}
