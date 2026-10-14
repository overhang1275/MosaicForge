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
