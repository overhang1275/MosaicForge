use mosaicforge_core::{
    EngineError,
    contract::{
        GenerationSpec, OutputSpec, generation::OutputSpecFormat, result::VariantResultStatus,
    },
    export::export_svg,
    generate::{Variant, generate_variant},
    template::Template,
    validate_output, validate_result,
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
    export_generated_variant(
        generate_variant(template, spec, index)?,
        &spec.output,
        available_png_bytes,
    )
}

/// Exports a candidate returned by generate_variant, allowing callers to reserve
/// shared buffer capacity after planning that candidate and before rendering it.
pub fn export_generated_variant(
    mut variant: Variant,
    output: &OutputSpec,
    available_png_bytes: u64,
) -> Result<ExportedVariant, EngineError> {
    validate_output(output)?;
    validate_result(&variant.result)?;
    if variant.template.is_some() != (variant.result.status == VariantResultStatus::Accepted) {
        return Err(EngineError::InvalidSpec(
            "candidate does not match variant status",
        ));
    }
    let artifact = if let Some(candidate) = &variant.template {
        let svg = candidate.normalized_svg.as_bytes();
        let bytes = match output.format {
            OutputSpecFormat::Svg => export_svg(svg, output)?,
            OutputSpecFormat::Png => {
                crate::render::render_png_with_budget(svg, output, available_png_bytes)?
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
