use super::geometry::{Node, OptimizedPage};
use mosaicforge_core::EngineError;
use pdf_writer::{Chunk, Content, Filter, Finish, Name, Rect, Ref};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, io::Write};
fn error() -> EngineError {
    EngineError::RenderFailed("PDF writer failed or exceeded resource limits")
}
#[derive(Default)]
struct Resources {
    forms: BTreeMap<String, Ref>,
    alpha: BTreeMap<String, Ref>,
}
pub(super) struct PdfWriter<'a, W: Write> {
    out: &'a mut W,
    offset: u64,
    hash: Sha256,
    offsets: Vec<Option<u64>>,
    pages: Vec<Ref>,
}
impl<'a, W: Write> PdfWriter<'a, W> {
    pub fn new(out: &'a mut W) -> Self {
        Self {
            out,
            offset: 0,
            hash: Sha256::new(),
            offsets: vec![None, None, None],
            pages: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Result<Ref, EngineError> {
        if self.offsets.len() >= 1_000_000 {
            return Err(error());
        }
        let id = Ref::new(self.offsets.len() as i32);
        self.offsets.push(None);
        Ok(id)
    }
    fn write(&mut self, bytes: &[u8]) -> Result<(), EngineError> {
        let next = self
            .offset
            .checked_add(bytes.len() as u64)
            .ok_or_else(error)?;
        if next > 9_999_999_999 {
            return Err(error());
        }
        self.out.write_all(bytes).map_err(|_| error())?;
        self.hash.update(bytes);
        self.offset = next;
        Ok(())
    }
    fn object(&mut self, id: Ref, chunk: Chunk) -> Result<(), EngineError> {
        if chunk.len() > 64 * 1024 * 1024 {
            return Err(error());
        }
        if self.offset == 0 {
            self.write(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n")?;
        }
        let slot = self.offsets.get_mut(id.get() as usize).ok_or_else(error)?;
        if slot.is_some() {
            return Err(error());
        }
        *slot = Some(self.offset);
        self.write(chunk.as_bytes())
    }
    fn alpha(&mut self, opacity: f32, res: &mut Resources) -> Result<String, EngineError> {
        let id = self.alloc()?;
        let name = format!("A{}", id.get());
        let mut chunk = Chunk::new();
        chunk
            .ext_graphics(id)
            .non_stroking_alpha(opacity)
            .stroking_alpha(opacity);
        self.object(id, chunk)?;
        res.alpha.insert(name.clone(), id);
        Ok(name)
    }
    fn node(
        &mut self,
        node: &Node,
        content: &mut Content,
        res: &mut Resources,
        rect: Rect,
        depth: usize,
    ) -> Result<(), EngineError> {
        if depth > 128 || content.len() > 64 * 1024 * 1024 {
            return Err(error());
        }
        match node {
            Node::Paint {
                contours,
                color,
                opacity,
            } => {
                if contours.is_empty() || *opacity == 0.0 {
                    return Ok(());
                }
                content.save_state();
                if *opacity != 1.0 {
                    let name = self.alpha(*opacity, res)?;
                    content.set_parameters(Name(name.as_bytes()));
                }
                content.set_fill_rgb(
                    color[0] as f32 / 255.0,
                    color[1] as f32 / 255.0,
                    color[2] as f32 / 255.0,
                );
                for contour in contours {
                    if let Some((first, rest)) = contour.split_first() {
                        content.move_to(first[0] as f32, first[1] as f32);
                        for point in rest {
                            content.line_to(point[0] as f32, point[1] as f32);
                        }
                        content.close_path();
                    }
                }
                content.fill_nonzero();
                content.restore_state();
            }
            Node::Group { children, opacity } => {
                if *opacity == 0.0 || children.is_empty() {
                    return Ok(());
                }
                if *opacity == 1.0 {
                    for child in children {
                        self.node(child, content, res, rect, depth + 1)?;
                    }
                } else {
                    let mut inner = Content::new();
                    let mut resources = Resources::default();
                    for child in children {
                        self.node(child, &mut inner, &mut resources, rect, depth + 1)?;
                    }
                    let bytes = miniz_oxide::deflate::compress_to_vec_zlib(&inner.finish(), 6);
                    let id = self.alloc()?;
                    let mut chunk = Chunk::new();
                    {
                        let mut form = chunk.form_xobject(id, &bytes);
                        form.bbox(rect).filter(Filter::FlateDecode);
                        form.group()
                            .transparency()
                            .isolated(true)
                            .color_space()
                            .device_rgb();
                        set_resources(form.resources(), &resources);
                    }
                    self.object(id, chunk)?;
                    let name = format!("F{}", id.get());
                    res.forms.insert(name.clone(), id);
                    content.save_state();
                    let alpha = self.alpha(*opacity, res)?;
                    content.set_parameters(Name(alpha.as_bytes()));
                    content.x_object(Name(name.as_bytes()));
                    content.restore_state();
                }
            }
        }
        Ok(())
    }
    pub fn page(&mut self, page: &OptimizedPage) -> Result<(), EngineError> {
        if self.pages.len() >= 10000 {
            return Err(error());
        }
        let b = page.boxes;
        let rect = Rect::new(b.media[0], b.media[1], b.media[2], b.media[3]);
        let mut content = Content::new();
        content.transform([1.0, 0.0, 0.0, -1.0, 0.0, b.media[3]]);
        let mut resources = Resources::default();
        self.node(&page.root, &mut content, &mut resources, rect, 0)?;
        let bytes = miniz_oxide::deflate::compress_to_vec_zlib(&content.finish(), 6);
        let stream = self.alloc()?;
        let mut chunk = Chunk::new();
        chunk.stream(stream, &bytes).filter(Filter::FlateDecode);
        self.object(stream, chunk)?;
        let id = self.alloc()?;
        let mut chunk = Chunk::new();
        {
            let mut p = chunk.page(id);
            p.parent(Ref::new(2))
                .media_box(rect)
                .crop_box(rect)
                .bleed_box(rect)
                .trim_box(Rect::new(b.trim[0], b.trim[1], b.trim[2], b.trim[3]))
                .contents(stream);
            set_resources(p.resources(), &resources);
        }
        self.object(id, chunk)?;
        self.pages.push(id);
        Ok(())
    }
    pub fn finish(mut self) -> Result<Option<String>, EngineError> {
        if self.pages.is_empty() {
            return Ok(None);
        }
        let mut catalog = Chunk::new();
        catalog
            .indirect(Ref::new(1))
            .dict()
            .pair(Name(b"Type"), Name(b"Catalog"))
            .pair(Name(b"Pages"), Ref::new(2));
        self.object(Ref::new(1), catalog)?;
        let mut pages = Chunk::new();
        pages
            .pages(Ref::new(2))
            .kids(self.pages.iter().copied())
            .count(self.pages.len() as i32);
        self.object(Ref::new(2), pages)?;
        let xref = self.offset;
        let mut trailer = format!("xref\n0 {}\n0000000000 65535 f \n", self.offsets.len());
        for offset in self.offsets.iter().skip(1) {
            trailer.push_str(&format!("{:010} 00000 n \n", offset.ok_or_else(error)?));
        }
        trailer.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            self.offsets.len()
        ));
        self.write(trailer.as_bytes())?;
        self.out.flush().map_err(|_| error())?;
        Ok(Some(format!("{:x}", self.hash.finalize())))
    }
}
fn set_resources(mut target: pdf_writer::writers::Resources<'_>, res: &Resources) {
    {
        let mut forms = target.x_objects();
        for (name, id) in &res.forms {
            forms.pair(Name(name.as_bytes()), *id);
        }
    }
    {
        let mut alpha = target.ext_g_states();
        for (name, id) in &res.alpha {
            alpha.pair(Name(name.as_bytes()), *id);
        }
    }
    target.finish();
}
