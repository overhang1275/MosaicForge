use i_overlay::{
    core::{fill_rule::FillRule, overlay_rule::OverlayRule},
    float::scale::FixedScaleFloatOverlay,
};
use kurbo::{CubicBez, ParamCurve, Point};
use mosaicforge_core::{
    EngineError,
    contract::PdfJobSpec,
    pdf::{PageBoxes, page_boxes},
};
use resvg::{
    tiny_skia::{Path, PathSegment, Transform},
    usvg,
};
pub type Contours = Vec<Vec<[f64; 2]>>;
#[derive(Debug)]
pub enum Node {
    Paint {
        contours: Contours,
        color: [u8; 3],
        opacity: f32,
    },
    Group {
        children: Vec<Node>,
        opacity: f32,
    },
}
#[derive(Debug, Default)]
pub struct GeometryStats {
    pub input_segments: usize,
    pub output_points: usize,
    pub discarded_paints: usize,
    pub flattened_points: usize,
    pub input_paints: usize,
    pub output_paints: usize,
}
#[derive(Debug)]
pub struct OptimizedPage {
    pub root: Node,
    pub boxes: PageBoxes,
    pub stats: GeometryStats,
}
// Precision is in physical PDF points; measured image differences are recorded in docs/.
pub const FLATNESS_PT: f64 = 0.001;
const MAX_POINTS: usize = 2_000_000;
const MAX_BOOLEAN_POINTS: usize = 32_768;
fn failed() -> EngineError {
    EngineError::RenderFailed("PDF geometry exceeds supported precision or complexity")
}
struct Geometry {
    stats: GeometryStats,
    flattened: usize,
}
pub fn optimize_page(svg: &[u8], job: &PdfJobSpec) -> Result<OptimizedPage, EngineError> {
    let boxes = page_boxes(job)?;
    let safe = mosaicforge_core::template::import_svg(svg)?;
    let options = usvg::Options {
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(&safe.normalized_svg, &options)
        .map_err(|_| EngineError::InvalidSvg("cannot parse PDF SVG"))?;
    let (w, h) = (tree.size().width(), tree.size().height());
    let scale = ((boxes.trim[2] - boxes.trim[0]) / w).max((boxes.trim[3] - boxes.trim[1]) / h);
    let tx = (boxes.trim[0] + boxes.trim[2] - w * scale) / 2.0;
    let ty = (boxes.trim[1] + boxes.trim[3] - h * scale) / 2.0;
    let transform = Transform::from_row(scale, 0.0, 0.0, scale, tx, ty);
    let boundary = vec![vec![
        [0.0, 0.0],
        [boxes.media[2] as f64, 0.0],
        [boxes.media[2] as f64, boxes.media[3] as f64],
        [0.0, boxes.media[3] as f64],
    ]];
    let mut geo = Geometry {
        stats: GeometryStats {
            input_segments: safe.segment_count,
            ..Default::default()
        },
        flattened: 0,
    };
    let mut children = Vec::new();
    if job.generation.output.background.as_str() != "transparent" {
        let rgb = u32::from_str_radix(&job.generation.output.background.as_str()[1..], 16)
            .map_err(|_| failed())?;
        children.push(Node::Paint {
            contours: boundary.clone(),
            color: [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8],
            opacity: 1.0,
        });
    }
    children.push(geo.group(tree.root(), transform, &boundary, 0)?);
    geo.stats.flattened_points = geo.flattened;
    Ok(OptimizedPage {
        root: Node::Group {
            children,
            opacity: 1.0,
        },
        boxes,
        stats: geo.stats,
    })
}
fn count(c: &Contours) -> usize {
    c.iter().map(Vec::len).sum()
}
fn overlay(
    a: &Contours,
    b: &Contours,
    op: OverlayRule,
    rule: FillRule,
) -> Result<Contours, EngineError> {
    let na = count(a);
    let nb = count(b);
    if na + nb > MAX_BOOLEAN_POINTS || na.saturating_mul(nb) > 4_000_000 {
        return Err(failed());
    }
    intersection_budget(a, b)?;
    let result: Contours = a
        .overlay_with_fixed_scale_as::<i64>(b, op, rule, 1_000_000.0)
        .map_err(|_| EngineError::RenderFailed("PDF coordinate precision is not representable"))?
        .into_iter()
        .flatten()
        .collect();
    if count(&result) > MAX_BOOLEAN_POINTS {
        return Err(failed());
    }
    Ok(result)
}

// Include self-intersections: limiting only subject × clip misses hostile single paths.
fn intersection_budget(a: &Contours, b: &Contours) -> Result<(), EngineError> {
    let mut edges = Vec::new();
    for contour in a.iter().chain(b) {
        if contour.len() < 2 {
            continue;
        }
        for i in 0..contour.len() {
            let p = contour[i];
            let q = contour[(i + 1) % contour.len()];
            edges.push([
                p[0].min(q[0]) - 0.00001,
                p[0].max(q[0]) + 0.00001,
                p[1].min(q[1]) - 0.00001,
                p[1].max(q[1]) + 0.00001,
            ]);
        }
    }
    edges.sort_by(|a, b| a[0].total_cmp(&b[0]));
    let mut active: Vec<[f64; 4]> = Vec::new();
    let (mut comparisons, mut candidates) = (0usize, 0usize);
    for edge in edges {
        active.retain(|old| old[1] >= edge[0]);
        for old in &active {
            comparisons += 1;
            if comparisons > 2_000_000 {
                return Err(EngineError::RenderFailed(
                    "PDF boolean preflight comparison budget exceeded",
                ));
            }
            if old[2] <= edge[3] && edge[2] <= old[3] {
                candidates += 1;
                if candidates > 50_000 {
                    return Err(EngineError::RenderFailed(
                        "PDF intersection candidate budget exceeded",
                    ));
                }
            }
        }
        active.push(edge);
    }
    Ok(())
}
impl Geometry {
    fn group(
        &mut self,
        g: &usvg::Group,
        parent: Transform,
        clip: &Contours,
        depth: usize,
    ) -> Result<Node, EngineError> {
        if depth > 128 || g.mask().is_some() || !g.filters().is_empty() {
            return Err(failed());
        }
        let ts = parent.pre_concat(g.transform());
        let effective = if let Some(c) = g.clip_path() {
            overlay(
                clip,
                &self.clip(c, ts, depth + 1)?,
                OverlayRule::Intersect,
                FillRule::NonZero,
            )?
        } else {
            clip.clone()
        };
        let mut children = Vec::new();
        for child in g.children() {
            match child {
                usvg::Node::Group(g) => {
                    let node = self.group(g, ts, &effective, depth + 1)?;
                    if !matches!(&node,Node::Group{children,..} if children.is_empty()) {
                        children.push(node);
                    }
                }
                usvg::Node::Path(p) => {
                    if p.is_visible() {
                        let mut paint = |stroke: bool| -> Result<(), EngineError> {
                            let (path, color, opacity, rule) = if stroke {
                                let Some(s) = p.stroke() else { return Ok(()) };
                                let resolution =
                                    (ts.sx.abs() + ts.kx.abs() + ts.ky.abs() + ts.sy.abs())
                                        .clamp(1.0, 10000.0);
                                let outline = p
                                    .data()
                                    .stroke(&s.to_tiny_skia(), resolution)
                                    .ok_or_else(failed)?;
                                (outline, s.paint(), s.opacity().get(), FillRule::NonZero)
                            } else {
                                let Some(f) = p.fill() else { return Ok(()) };
                                (
                                    p.data().clone(),
                                    f.paint(),
                                    f.opacity().get(),
                                    fill_rule(f.rule()),
                                )
                            };
                            let usvg::Paint::Color(color) = color else {
                                return Err(failed());
                            };
                            self.stats.input_paints += 1;
                            let raw = self.flatten(&path, ts)?;
                            let normalized = overlay(&raw, &Vec::new(), OverlayRule::Union, rule)?;
                            let contours = overlay(
                                &normalized,
                                &effective,
                                OverlayRule::Intersect,
                                FillRule::NonZero,
                            )?;
                            if contours.is_empty() {
                                self.stats.discarded_paints += 1;
                                return Ok(());
                            }
                            self.stats.output_paints += 1;
                            self.stats.output_points = self
                                .stats
                                .output_points
                                .checked_add(count(&contours))
                                .ok_or_else(failed)?;
                            if self.stats.output_points > MAX_POINTS {
                                return Err(failed());
                            }
                            children.push(Node::Paint {
                                contours,
                                color: [color.red, color.green, color.blue],
                                opacity,
                            });
                            Ok(())
                        };
                        if p.paint_order() == usvg::PaintOrder::FillAndStroke {
                            paint(false)?;
                            paint(true)?;
                        } else {
                            paint(true)?;
                            paint(false)?;
                        }
                    }
                }
                _ => return Err(failed()),
            }
        }
        Ok(Node::Group {
            children,
            opacity: g.opacity().get(),
        })
    }
    fn clip(
        &mut self,
        c: &usvg::ClipPath,
        parent: Transform,
        depth: usize,
    ) -> Result<Contours, EngineError> {
        if depth > 128 {
            return Err(failed());
        }
        let ts = parent.pre_concat(c.transform());
        let mut contours = self.clip_children(c.root(), ts, depth + 1)?;
        if let Some(next) = c.clip_path() {
            contours = overlay(
                &contours,
                &self.clip(next, parent, depth + 1)?,
                OverlayRule::Intersect,
                FillRule::NonZero,
            )?;
        }
        Ok(contours)
    }
    fn clip_children(
        &mut self,
        g: &usvg::Group,
        parent: Transform,
        depth: usize,
    ) -> Result<Contours, EngineError> {
        if depth > 128 {
            return Err(failed());
        }
        let ts = parent.pre_concat(g.transform());
        let mut result = Vec::new();
        for child in g.children() {
            let geometry = match child {
                usvg::Node::Path(p) => {
                    let Some(fill) = p.fill() else { continue };
                    let points = self.flatten(p.data(), ts)?;
                    overlay(
                        &points,
                        &Vec::new(),
                        OverlayRule::Union,
                        fill_rule(fill.rule()),
                    )?
                }
                usvg::Node::Group(g) => self.clip_children(g, ts, depth + 1)?,
                _ => return Err(failed()),
            };
            result = overlay(&result, &geometry, OverlayRule::Union, FillRule::NonZero)?;
        }
        if let Some(c) = g.clip_path() {
            result = overlay(
                &result,
                &self.clip(c, ts, depth + 1)?,
                OverlayRule::Intersect,
                FillRule::NonZero,
            )?;
        }
        Ok(result)
    }
    fn flatten(&mut self, path: &Path, ts: Transform) -> Result<Contours, EngineError> {
        if path.len() > MAX_BOOLEAN_POINTS {
            return Err(failed());
        }
        let path = path.clone().transform(ts).ok_or_else(failed)?;
        let mut out = Vec::new();
        let mut current = Vec::new();
        let mut last = Point::ZERO;
        for segment in path.segments() {
            match segment {
                PathSegment::MoveTo(p) => {
                    if current.len() >= 3 {
                        out.push(std::mem::take(&mut current));
                    } else {
                        current.clear();
                    }
                    last = Point::new(p.x as f64, p.y as f64);
                    current.push([last.x, last.y]);
                }
                PathSegment::LineTo(p) => {
                    self.push(&mut current, Point::new(p.x as f64, p.y as f64))?;
                    last = Point::new(p.x as f64, p.y as f64);
                }
                PathSegment::QuadTo(a, b) => {
                    let a = Point::new(a.x as f64, a.y as f64);
                    let b = Point::new(b.x as f64, b.y as f64);
                    self.curve(
                        CubicBez::new(
                            last,
                            last + (a - last) * (2.0 / 3.0),
                            b + (a - b) * (2.0 / 3.0),
                            b,
                        ),
                        &mut current,
                        0,
                    )?;
                    last = b;
                }
                PathSegment::CubicTo(a, b, c) => {
                    let c = Point::new(c.x as f64, c.y as f64);
                    self.curve(
                        CubicBez::new(
                            last,
                            Point::new(a.x as f64, a.y as f64),
                            Point::new(b.x as f64, b.y as f64),
                            c,
                        ),
                        &mut current,
                        0,
                    )?;
                    last = c;
                }
                PathSegment::Close => {
                    if current.len() >= 3 {
                        out.push(std::mem::take(&mut current));
                    } else {
                        current.clear();
                    }
                }
            }
        }
        if current.len() >= 3 {
            out.push(current);
        }
        Ok(out)
    }
    fn push(&mut self, out: &mut Vec<[f64; 2]>, p: Point) -> Result<(), EngineError> {
        self.flattened += 1;
        if self.flattened > MAX_POINTS
            || out.len() >= MAX_BOOLEAN_POINTS
            || !p.x.is_finite()
            || !p.y.is_finite()
            || p.x.abs() > 1e25
            || p.y.abs() > 1e25
        {
            return Err(failed());
        }
        out.push([p.x, p.y]);
        Ok(())
    }
    fn curve(
        &mut self,
        c: CubicBez,
        out: &mut Vec<[f64; 2]>,
        depth: u8,
    ) -> Result<(), EngineError> {
        let distance = |p: Point| {
            let v = c.p3 - c.p0;
            let d = v.hypot2();
            let t = if d == 0.0 {
                0.0
            } else {
                ((p - c.p0).dot(v) / d).clamp(0.0, 1.0)
            };
            (p - (c.p0 + v * t)).hypot()
        };
        if distance(c.p1).max(distance(c.p2)) <= FLATNESS_PT {
            return self.push(out, c.p3);
        }
        if depth >= 24 {
            return Err(failed());
        }
        self.curve(c.subsegment(0.0..0.5), out, depth + 1)?;
        self.curve(c.subsegment(0.5..1.0), out, depth + 1)
    }
}
fn fill_rule(r: usvg::FillRule) -> FillRule {
    match r {
        usvg::FillRule::NonZero => FillRule::NonZero,
        usvg::FillRule::EvenOdd => FillRule::EvenOdd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn self_intersections_are_rejected_before_building_the_boolean_graph() {
        let n = 1025;
        let contour = (0..n)
            .map(|i| {
                let a = ((i * 512) % n) as f64 * std::f64::consts::TAU / n as f64;
                [100.0 * a.cos(), 100.0 * a.sin()]
            })
            .collect();
        assert!(
            overlay(
                &vec![contour],
                &Vec::new(),
                OverlayRule::Union,
                FillRule::EvenOdd
            )
            .is_err()
        );
    }

    #[test]
    #[ignore = "requires local PDF source sample; creates a controlled unclipped-geometry baseline"]
    fn write_untrimmed_local_baseline() {
        use std::{fs, io::Write};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
        let job: PdfJobSpec =
            serde_json::from_slice(&fs::read(root.join("out/pdf-5.6/job.json")).unwrap()).unwrap();
        let svg = fs::read(root.join("out/pdf-5.6/source-0.svg")).unwrap();
        let boxes = page_boxes(&job).unwrap();
        let safe = mosaicforge_core::template::import_svg(&svg).unwrap();
        let tree = usvg::Tree::from_str(&safe.normalized_svg, &usvg::Options::default()).unwrap();
        let scale =
            (boxes.media[2] / tree.size().width()).max(boxes.media[3] / tree.size().height());
        let ts = Transform::from_row(
            scale,
            0.0,
            0.0,
            scale,
            (boxes.media[2] - tree.size().width() * scale) / 2.0,
            (boxes.media[3] - tree.size().height() * scale) / 2.0,
        );
        let mut geo = Geometry {
            stats: GeometryStats {
                input_segments: safe.segment_count,
                ..Default::default()
            },
            flattened: 0,
        };
        let broad = vec![vec![[-1e6, -1e6], [1e6, -1e6], [1e6, 1e6], [-1e6, 1e6]]];
        let root_node = geo.group(tree.root(), ts, &broad, 0).unwrap();
        geo.stats.flattened_points = geo.flattened;
        let background = Node::Paint {
            contours: vec![vec![
                [0.0, 0.0],
                [boxes.media[2] as f64, 0.0],
                [boxes.media[2] as f64, boxes.media[3] as f64],
                [0.0, boxes.media[3] as f64],
            ]],
            color: [255, 255, 255],
            opacity: 1.0,
        };
        let baseline = OptimizedPage {
            root: Node::Group {
                children: vec![background, root_node],
                opacity: 1.0,
            },
            boxes,
            stats: geo.stats,
        };
        let optimized = optimize_page(&svg, &job).unwrap();
        for (name, page) in [("baseline", baseline), ("optimized", optimized)] {
            let mut bytes = Vec::new();
            let mut writer = super::super::writer::PdfWriter::new(&mut bytes);
            writer.page(&page).unwrap();
            writer.finish().unwrap();
            let path = root.join(format!("out/pdf-5.6-{name}-validated.pdf"));
            if path.exists() {
                assert_eq!(fs::read(path).unwrap(), bytes);
            } else {
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .unwrap()
                    .write_all(&bytes)
                    .unwrap();
            }
            println!("{name} bytes={} stats={:?}", bytes.len(), page.stats);
        }
    }

    #[test]
    #[ignore = "writes local independent PDF visual QA fixtures"]
    fn write_pdf_visual_fixtures() {
        use serde_json::{Value, json};
        use std::{fs, io::Write};
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
        let dir = root.join("out/pdf-5.6-fixtures");
        fs::create_dir_all(&dir).unwrap();
        let sources = [
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g opacity="0.5"><rect x="5" y="5" width="70" height="70" fill="#ff0000"/><rect x="30" y="30" width="65" height="65" fill="#0000ff"/></g></svg>"##,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g transform="matrix(1.3 0.2 0.3 0.5 -15 30)"><path d="M-10 0Q50 120 110 0 L50 100Z" fill="#ddaa44" fill-opacity="0.6" stroke="#0044cc" stroke-width="8" stroke-linejoin="miter" stroke-linecap="round"/></g></svg>"##,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><defs><clipPath id="a"><path d="M0 0H100V100H0Z M25 25H75V75H25Z" clip-rule="evenodd"/></clipPath><clipPath id="b" clip-path="url(#a)"><circle cx="50" cy="50" r="45"/><rect x="0" y="0" width="20" height="100"/></clipPath></defs><g clip-path="url(#b)" opacity="0.7"><rect x="-20" y="-20" width="140" height="140" fill="#118866"/></g></svg>"##,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100"><rect x="-10" y="-10" width="120" height="120" fill="#eebb22"/><circle cx="0" cy="50" r="20" fill="#aa1188"/></svg>"##,
        ];
        let g: Value =
            serde_json::from_str(include_str!("../../../../tests/fixtures/identity.json")).unwrap();
        let mut job:PdfJobSpec=serde_json::from_value(json!({"schemaVersion":"1","generation":g,"pdf":{"exporterVersion":"1","trimWidthMm":100,"trimHeightMm":100,"bleedMm":0}})).unwrap();
        let mut file = Vec::new();
        let mut writer = super::super::writer::PdfWriter::new(&mut file);
        for (i, svg) in sources.iter().enumerate() {
            job.pdf.bleed_mm = if i == 3 { 3.0 } else { 0.0 };
            let page = optimize_page(svg.as_bytes(), &job).unwrap();
            writer.page(&page).unwrap();
            let tree = usvg::Tree::from_str(svg, &usvg::Options::default()).unwrap();
            let b = page.boxes;
            let s = (b.trim[2] - b.trim[0]) / 100.0;
            let dpi = 300.0 / 72.0;
            let mut image = resvg::tiny_skia::Pixmap::new(
                (b.media[2] * dpi).ceil() as u32,
                (b.media[3] * dpi).ceil() as u32,
            )
            .unwrap();
            image.fill(resvg::tiny_skia::Color::WHITE);
            resvg::render(
                &tree,
                Transform::from_row(s * dpi, 0.0, 0.0, s * dpi, b.trim[0] * dpi, b.trim[1] * dpi),
                &mut image.as_mut(),
            );
            let path = dir.join(format!("reference-{i}.png"));
            let png = image.encode_png().unwrap();
            if path.exists() {
                assert_eq!(fs::read(path).unwrap(), png);
            } else {
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .unwrap()
                    .write_all(&png)
                    .unwrap();
            }
        }
        writer.finish().unwrap();
        let path = dir.join("fixtures.pdf");
        if path.exists() {
            assert_eq!(fs::read(path).unwrap(), file);
        } else {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap()
                .write_all(&file)
                .unwrap();
        }
    }
}
