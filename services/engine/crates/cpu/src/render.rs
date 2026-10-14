use mosaicforge_core::{
    EngineError,
    contract::{OutputSpec, generation::OutputSpecFormat},
    validate_output,
};

pub const MAX_RENDER_BUFFER_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, PartialEq, Eq)]
pub struct PngBufferPlan {
    pub width: u32,
    pub height: u32,
    pub rgba_bytes: u64,
    pub reserved_bytes: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PngRenderPlan {
    pub base: PngBufferPlan,
    pub temporary_bytes: u64,
    pub reserved_bytes: u64,
}

pub fn plan_png_render(
    svg: &[u8],
    output: &OutputSpec,
    available_bytes: u64,
) -> Result<PngRenderPlan, EngineError> {
    prepare_png(svg, output, available_bytes).map(|(_, plan)| plan)
}

pub fn render_png(svg: &[u8], output: &OutputSpec) -> Result<Vec<u8>, EngineError> {
    render_png_with_budget(svg, output, MAX_RENDER_BUFFER_BYTES)
}

pub fn render_png_with_budget(
    svg: &[u8],
    output: &OutputSpec,
    available_bytes: u64,
) -> Result<Vec<u8>, EngineError> {
    let (tree, plan) = prepare_png(svg, output, available_bytes)?;
    let mut image = resvg::tiny_skia::Pixmap::new(plan.base.width, plan.base.height)
        .ok_or(EngineError::RenderFailed("cannot allocate PNG canvas"))?;
    if output.background.as_str() != "transparent" {
        let rgb = u32::from_str_radix(&output.background.as_str()[1..], 16)
            .map_err(|_| EngineError::InvalidSpec("invalid RGB background"))?;
        image.fill(resvg::tiny_skia::Color::from_rgba8(
            (rgb >> 16) as u8,
            (rgb >> 8) as u8,
            rgb as u8,
            255,
        ));
    }
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut image.as_mut(),
    );
    image
        .encode_png()
        .map_err(|_| EngineError::RenderFailed("PNG encoding failed"))
}

fn prepare_png(
    svg: &[u8],
    output: &OutputSpec,
    available_bytes: u64,
) -> Result<(resvg::usvg::Tree, PngRenderPlan), EngineError> {
    let base = plan_png_buffers(output, available_bytes)?;
    let source = mosaicforge_core::template::prepare_viewport(svg, output)?;
    let options = resvg::usvg::Options {
        image_href_resolver: resvg::usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        ..Default::default()
    };
    let tree = resvg::usvg::Tree::from_str(&source.normalized_svg, &options)
        .map_err(|_| EngineError::InvalidSvg("cannot build PNG geometry"))?;
    let mut estimator = TemporaryBuffers {
        width: base.width,
        height: base.height,
        clips: std::collections::HashMap::new(),
    };
    let temporary_bytes =
        estimator.children(tree.root(), resvg::tiny_skia::Transform::identity(), 0)?;
    let reserved_bytes =
        base.reserved_bytes
            .checked_add(temporary_bytes)
            .ok_or(EngineError::InvalidSpec(
                "render buffer reservation overflow",
            ))?;
    if reserved_bytes > available_bytes {
        return Err(EngineError::InvalidSpec(
            "insufficient budget for PNG layers and clips",
        ));
    }
    Ok((
        tree,
        PngRenderPlan {
            base,
            temporary_bytes,
            reserved_bytes,
        },
    ))
}

struct TemporaryBuffers {
    width: u32,
    height: u32,
    clips: std::collections::HashMap<*const resvg::usvg::ClipPath, u64>,
}

impl TemporaryBuffers {
    fn children(
        &mut self,
        group: &resvg::usvg::Group,
        transform: resvg::tiny_skia::Transform,
        depth: u16,
    ) -> Result<u64, EngineError> {
        if depth > 128 {
            return Err(EngineError::InvalidSvg("render group depth exceeds budget"));
        }
        let mut peak = 0;
        for node in group.children() {
            match node {
                resvg::usvg::Node::Group(child) => {
                    peak = peak.max(self.group(child, transform, depth + 1)?)
                }
                resvg::usvg::Node::Path(_) => (),
                _ => return Err(EngineError::InvalidSvg("unsupported PNG rendering node")),
            }
        }
        Ok(peak)
    }

    fn group(
        &mut self,
        group: &resvg::usvg::Group,
        parent: resvg::tiny_skia::Transform,
        depth: u16,
    ) -> Result<u64, EngineError> {
        if group.mask().is_some() || !group.filters().is_empty() {
            return Err(EngineError::InvalidSvg("unsupported PNG rendering effect"));
        }
        let transform = parent.pre_concat(group.transform());
        let children = self.children(group, transform, depth)?;
        if !group.should_isolate() {
            return Ok(children);
        }
        let Some(bbox) = group.layer_bounding_box().transform(transform) else {
            return Ok(0);
        };
        if f64::from(bbox.x()).floor() < f64::from(i32::MIN) + 2.0
            || f64::from(bbox.y()).floor() < f64::from(i32::MIN) + 2.0
            || f64::from(bbox.width()).ceil() > f64::from(u32::MAX) - 4.0
            || f64::from(bbox.height()).ceil() > f64::from(u32::MAX) - 4.0
        {
            return Err(EngineError::InvalidSvg(
                "layer bounds exceed renderer integer limits",
            ));
        }
        // Pinned resvg caps layer extents at five viewport widths/heights; allow AA padding.
        let width = (f64::from(bbox.width()).ceil() + 6.0).min(f64::from(self.width) * 5.0) as u64;
        let height =
            (f64::from(bbox.height()).ceil() + 6.0).min(f64::from(self.height) * 5.0) as u64;
        let area = width
            .checked_mul(height)
            .ok_or(EngineError::InvalidSpec("layer area overflow"))?;
        let clip = if let Some(clip) = group.clip_path() {
            area.checked_mul(self.clip_cost(clip, 0)?)
                .ok_or(EngineError::InvalidSpec("clip buffer overflow"))?
        } else {
            0
        };
        area.checked_mul(4)
            .and_then(|own| own.checked_add(children.max(clip)))
            .ok_or(EngineError::InvalidSpec("layer buffer overflow"))
    }

    // Clip costs are bytes per pixel: RGBA temporaries plus the alpha mask, excluding target.
    fn clip_cost(&mut self, clip: &resvg::usvg::ClipPath, depth: u16) -> Result<u64, EngineError> {
        if depth > 128 {
            return Err(EngineError::InvalidSvg("render clip depth exceeds budget"));
        }
        let key = std::ptr::from_ref(clip);
        if let Some(&cost) = self.clips.get(&key) {
            return Ok(cost);
        }
        let children = self.clip_children(clip.root(), depth + 1)?;
        let nested = if let Some(next) = clip.clip_path() {
            self.clip_cost(next, depth + 1)?
        } else {
            0
        };
        let cost = 4u64
            .checked_add(children.max(nested).max(1))
            .ok_or(EngineError::InvalidSpec("clip cost overflow"))?;
        self.clips.insert(key, cost);
        Ok(cost)
    }

    fn clip_children(
        &mut self,
        group: &resvg::usvg::Group,
        depth: u16,
    ) -> Result<u64, EngineError> {
        if depth > 128 {
            return Err(EngineError::InvalidSvg(
                "render clip group depth exceeds budget",
            ));
        }
        let mut peak = 0;
        for node in group.children() {
            match node {
                resvg::usvg::Node::Group(child) => {
                    let children = self.clip_children(child, depth + 1)?;
                    let cost = if let Some(clip) = child.clip_path() {
                        4u64.checked_add(children.max(self.clip_cost(clip, depth + 1)?))
                            .ok_or(EngineError::InvalidSpec("clip group cost overflow"))?
                    } else {
                        children
                    };
                    peak = peak.max(cost);
                }
                resvg::usvg::Node::Path(_) => (),
                _ => return Err(EngineError::InvalidSvg("unsupported clip rendering node")),
            }
        }
        Ok(peak)
    }
}

/// Base reservation only; renderer layer/clip temporaries must also fit before rendering.
pub fn plan_png_buffers(
    output: &OutputSpec,
    available_bytes: u64,
) -> Result<PngBufferPlan, EngineError> {
    validate_output(output)?;
    let invalid = EngineError::InvalidSpec;
    if output.format != OutputSpecFormat::Png {
        return Err(invalid("PNG buffer planning requires PNG output"));
    }
    if available_bytes > MAX_RENDER_BUFFER_BYTES {
        return Err(invalid("available render budget exceeds worker limit"));
    }
    let width = output.width.get();
    let height = output.height.get();
    let rgba_bytes = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(4))
        .ok_or(invalid("RGBA buffer size overflow"))?;
    // Policy reserve for canvas, encoding copies/output and fixed working space; not process RSS.
    let reserved_bytes = rgba_bytes
        .checked_mul(4)
        .and_then(|n| n.checked_add(16 * 1024 * 1024))
        .ok_or(invalid("render reservation overflow"))?;
    if reserved_bytes > available_bytes {
        return Err(invalid("insufficient render buffer budget"));
    }
    Ok(PngBufferPlan {
        width: u32::try_from(width).map_err(|_| invalid("render width exceeds u32"))?,
        height: u32::try_from(height).map_err(|_| invalid("render height exceeds u32"))?,
        rgba_bytes,
        reserved_bytes,
    })
}
