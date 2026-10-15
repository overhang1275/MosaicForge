use crate::{Failure, io_failure, read_template};
use mosaicforge_core::{
    EngineError,
    contract::{GenerationSpec, generation::OutputSpecFormat},
    template::Template,
    validate_spec,
};
use mosaicforge_cpu::{export::generate_and_export, render::MAX_RENDER_BUFFER_BYTES};
use serde_json::json;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

fn engine_failure(error: EngineError) -> Failure {
    Failure {
        exit: if matches!(error, EngineError::RenderFailed(_)) {
            3
        } else {
            2
        },
        code: error.code(),
        message: error.to_string(),
    }
}
fn invalid_spec(message: &'static str) -> Failure {
    engine_failure(EngineError::InvalidSpec(message))
}
pub fn run(args: &clap::ArgMatches, out: &mut impl Write) -> Result<(), Failure> {
    let template = read_template(
        args.get_one::<PathBuf>("template")
            .expect("required by clap"),
    )?;
    let spec_path = args.get_one::<PathBuf>("spec").expect("required by clap");
    let spec = read_spec(spec_path)?;
    validate_spec(&spec).map_err(engine_failure)?;
    if template.hash.as_str() != spec.template_hash.as_str() {
        return Err(invalid_spec(
            "templateHash does not match the normalized SVG",
        ));
    }
    let directory = args.get_one::<PathBuf>("out").expect("required by clap");
    match fs::create_dir(directory) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && directory.is_dir() => (),
        Err(_) => return Err(io_failure("cannot create output directory")),
    }
    generate(&template, &spec, directory, out)
}
fn read_spec(path: &Path) -> Result<GenerationSpec, Failure> {
    if !fs::metadata(path)
        .map_err(|_| io_failure("cannot inspect spec file"))?
        .is_file()
    {
        return Err(io_failure("spec must be a regular file"));
    }
    let file = File::open(path).map_err(|_| io_failure("cannot open spec file"))?;
    if !file
        .metadata()
        .map_err(|_| io_failure("cannot inspect open spec file"))?
        .is_file()
    {
        return Err(io_failure("spec must be a regular file"));
    }
    let mut bytes = Vec::new();
    const LIMIT: u64 = 1024 * 1024;
    file.take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| io_failure("cannot read spec file"))?;
    if bytes.len() as u64 > LIMIT {
        return Err(invalid_spec("spec byte limit exceeded"));
    }
    serde_json::from_slice(&bytes).map_err(|_| invalid_spec("invalid GenerationSpec JSON"))
}
fn emit(out: &mut impl Write, value: &serde_json::Value) -> Result<(), Failure> {
    serde_json::to_writer(&mut *out, value)
        .map_err(|_| io_failure("cannot write command output"))?;
    out.write_all(b"\n")
        .and_then(|_| out.flush())
        .map_err(|_| io_failure("cannot write command output"))
}
fn generate(
    template: &Template,
    spec: &GenerationSpec,
    directory: &Path,
    out: &mut impl Write,
) -> Result<(), Failure> {
    let start = spec
        .start_index
        .parse::<u64>()
        .map_err(|_| invalid_spec("startIndex exceeds u64"))?;
    let extension = match spec.output.format {
        OutputSpecFormat::Svg => "svg",
        OutputSpecFormat::Png => "png",
    };
    let mut accepted = 0u64;
    for offset in 0..spec.count.get() {
        let index = start + offset;
        let exported = generate_and_export(template, spec, index, MAX_RENDER_BUFFER_BYTES)
            .map_err(engine_failure)?;
        if let Some(bytes) = &exported.artifact {
            crate::publish::publish_with(
                &directory.join(format!("variant-{index}.{extension}")),
                |file| file.write_all(bytes),
            )
            .map_err(|_| io_failure("cannot publish artifact without replacement"))?;
            accepted += 1;
        }
        emit(
            out,
            &json!({"kind":"variant","result":exported.variant.result}),
        )?;
    }
    emit(
        out,
        &json!({"kind":"summary","accepted":accepted,"rejected":spec.count.get()-accepted}),
    )
}
