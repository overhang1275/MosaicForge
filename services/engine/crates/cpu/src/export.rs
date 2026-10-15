use mosaicforge_core::{
    EngineError,
    contract::{GenerationSpec, generation::OutputSpecFormat},
    export::export_svg,
    generate::{Variant, generate_variant},
    template::Template,
    validate_result,
};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub struct ExportedVariant {
    pub variant: Variant,
    pub artifact: Option<Vec<u8>>,
}

/// Generates one candidate and hashes its final SVG/PNG bytes, without publishing files.
/// The PNG buffer budget applies only to accepted PNG output, not total process memory.
pub fn generate_and_export(
    template: &Template,
    spec: &GenerationSpec,
    index: u64,
    available_png_bytes: u64,
) -> Result<ExportedVariant, EngineError> {
    let mut variant = generate_variant(template, spec, index)?;
    let artifact = if let Some(candidate) = &variant.template {
        let svg = candidate.normalized_svg.as_bytes();
        let bytes = match spec.output.format {
            OutputSpecFormat::Svg => export_svg(svg, &spec.output)?,
            OutputSpecFormat::Png => {
                crate::render::render_png_with_budget(svg, &spec.output, available_png_bytes)?
            }
        };
        variant.result.artifact_hash = Some(
            format!("{:x}", Sha256::digest(&bytes))
                .parse()
                .map_err(|_| EngineError::InvalidSpec("cannot encode artifact hash"))?,
        );
        Some(bytes)
    } else {
        None
    };
    validate_result(&variant.result)?;
    Ok(ExportedVariant { variant, artifact })
}
