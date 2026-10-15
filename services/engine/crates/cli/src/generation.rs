use crate::{Failure, io_failure, read_template};
use mosaicforge_core::{
    EngineError, configuration_hash,
    contract::{GenerationSpec, VariantResult, generation::OutputSpecFormat},
    generate::generate_variant,
    template::Template,
    validate_spec,
};
use mosaicforge_cpu::{
    export::export_generated_variant,
    render::{MAX_RENDER_BUFFER_BYTES, plan_png_render},
};
use rayon::prelude::*;
use serde_json::json;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub(super) fn cancelled() -> Failure {
    Failure {
        exit: 130,
        code: "cancelled",
        message: "generation cancelled".into(),
    }
}
pub(super) fn check_cancelled(token: &AtomicBool) -> Result<(), Failure> {
    if token.load(Ordering::Relaxed) {
        Err(cancelled())
    } else {
        Ok(())
    }
}
struct Execution<'a> {
    template: &'a Template,
    spec: &'a GenerationSpec,
    directory: &'a Path,
    batch_size: u64,
    threads: usize,
    token: &'a AtomicBool,
}

pub(super) fn engine_failure(error: EngineError) -> Failure {
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
pub(super) fn invalid_spec(message: &'static str) -> Failure {
    engine_failure(EngineError::InvalidSpec(message))
}
pub(super) fn install_cancellation() -> Result<Arc<AtomicBool>, Failure> {
    let token = Arc::new(AtomicBool::new(false));
    let signal_token = Arc::clone(&token);
    ctrlc::set_handler(move || signal_token.store(true, Ordering::Relaxed))
        .map_err(|_| io_failure("cannot register Ctrl+C handler"))?;
    Ok(token)
}
pub fn run(args: &clap::ArgMatches, out: &mut impl Write) -> Result<(), Failure> {
    let token = install_cancellation()?;
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
    let batch_size = *args
        .get_one::<u64>("batch-size")
        .expect("defaulted by clap");
    let threads = *args.get_one::<u64>("threads").expect("defaulted by clap") as usize;
    let mut manifest = create_manifest(directory)?;
    generate(
        &Execution {
            template: &template,
            spec: &spec,
            directory,
            batch_size,
            threads,
            token: &token,
        },
        out,
        &mut manifest,
    )
}
fn read_spec(path: &Path) -> Result<GenerationSpec, Failure> {
    serde_json::from_slice(&read_spec_bytes(path)?)
        .map_err(|_| invalid_spec("invalid GenerationSpec JSON"))
}
pub(super) fn create_manifest(directory: &Path) -> Result<File, Failure> {
    match fs::create_dir(directory) {
        Ok(()) => (),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && directory.is_dir() => (),
        Err(_) => return Err(io_failure("cannot create output directory")),
    }
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join("manifest.jsonl"))
        .map_err(|_| io_failure("cannot create manifest without replacement"))
}
pub(super) fn read_spec_bytes(path: &Path) -> Result<Vec<u8>, Failure> {
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
    Ok(bytes)
}
pub(super) fn write_record(
    writer: &mut impl Write,
    value: &serde_json::Value,
) -> std::io::Result<()> {
    let mut line = serde_json::to_vec(value)?;
    line.push(b'\n');
    writer.write_all(&line).and_then(|_| writer.flush())
}
fn generate(
    job: &Execution<'_>,
    out: &mut impl Write,
    manifest: &mut impl Write,
) -> Result<(), Failure> {
    let start = job
        .spec
        .start_index
        .parse::<u64>()
        .map_err(|_| invalid_spec("startIndex exceeds u64"))?;
    write_record(manifest, &json!({"kind":"job","spec":job.spec,"configurationHash":configuration_hash(job.spec).map_err(engine_failure)?,"backend":"cpu"}))
        .map_err(|_| io_failure("cannot write manifest"))?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(job.threads)
        .build()
        .map_err(|_| io_failure("cannot create worker pool"))?;
    let budget = crate::memory::BufferBudget::new(MAX_RENDER_BUFFER_BYTES);
    let mut accepted = 0u64;
    let mut rejected = 0u64;
    let mut failure = None;
    'groups: for group_start in (0..job.spec.count.get()).step_by(job.batch_size as usize) {
        let group_end = (group_start + job.batch_size).min(job.spec.count.get());
        for window_start in (group_start..group_end).step_by(job.threads) {
            if let Err(e) = check_cancelled(job.token) {
                failure = Some(e);
                break 'groups;
            }
            // Only indices/results for this window survive; artifact bytes are
            // published and dropped by the worker, never queued for the batch.
            let indices: Vec<_> = (window_start
                ..(window_start + job.threads as u64).min(group_end))
                .map(|offset| start + offset)
                .collect();
            let results: Vec<_> = pool.install(|| {
                indices
                    .par_iter()
                    .map(|&index| {
                        generate_one(
                            job.template,
                            job.spec,
                            index,
                            job.directory,
                            &budget,
                            job.token,
                        )
                    })
                    .collect()
            });
            for result in results {
                match result {
                    Ok(result) => {
                        // Persist every completed publication, even if a sibling
                        // failed or stdout is broken. Manifest errors are fatal.
                        write_record(manifest, &json!({"kind":"variant","result":result}))
                            .map_err(|_| io_failure("cannot write manifest"))?;
                        if result.artifact_hash.is_some() {
                            accepted += 1;
                        } else {
                            rejected += 1;
                        }
                        if write_record(out, &json!({"kind":"variant","result":result})).is_err() {
                            failure = Some(io_failure("cannot write command output"));
                        }
                    }
                    Err(e) => {
                        if failure.as_ref().is_none_or(|old: &Failure| old.exit == 130) {
                            failure = Some(e);
                        }
                    }
                }
            }
            if failure.is_some() {
                break 'groups;
            }
        }
    }
    let status = match failure.as_ref() {
        None => "completed",
        Some(e) if e.exit == 130 => "cancelled",
        Some(_) => "failed",
    };
    let summary = json!({"kind":"summary","status":status,"accepted":accepted,"rejected":rejected});
    write_record(manifest, &summary).map_err(|_| io_failure("cannot write manifest"))?;
    write_record(out, &summary).map_err(|_| io_failure("cannot write command output"))?;
    match failure {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn generate_one(
    template: &Template,
    spec: &GenerationSpec,
    index: u64,
    directory: &Path,
    budget: &crate::memory::BufferBudget,
    token: &AtomicBool,
) -> Result<VariantResult, Failure> {
    check_cancelled(token)?;
    let variant = generate_variant(template, spec, index).map_err(engine_failure)?;
    check_cancelled(token)?;
    let bytes = if let Some(candidate) = &variant.template {
        if spec.output.format == OutputSpecFormat::Png {
            plan_png_render(
                candidate.normalized_svg.as_bytes(),
                &spec.output,
                MAX_RENDER_BUFFER_BYTES,
            )
            .map_err(engine_failure)?
            .reserved_bytes
        } else {
            0
        }
    } else {
        0
    };
    let reservation = budget
        .reserve(bytes, token)
        .map_err(engine_failure)?
        .ok_or_else(cancelled)?;
    let exported = export_generated_variant(variant, &spec.output, MAX_RENDER_BUFFER_BYTES)
        .map_err(engine_failure)?;
    check_cancelled(token)?;
    if let Some(bytes) = exported.artifact {
        let extension = match spec.output.format {
            OutputSpecFormat::Svg => "svg",
            OutputSpecFormat::Png => "png",
        };
        crate::publish::publish_with(
            &directory.join(format!("variant-{index}.{extension}")),
            |file| file.write_all(&bytes),
        )
        .map_err(|_| io_failure("cannot publish artifact without replacement"))?;
        drop(bytes);
    }
    drop(reservation);
    Ok(exported.variant.result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io, sync::atomic::AtomicU64};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    const SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100" fill="#123456"/></g></svg>"##;
    struct Fixture {
        directory: PathBuf,
        template: Template,
        spec: GenerationSpec,
    }
    impl Fixture {
        fn new() -> Self {
            let directory = std::env::temp_dir().join(format!(
                "mosaicforge-cancel-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            let template = mosaicforge_core::template::import_svg(SVG).unwrap();
            let mut spec: serde_json::Value =
                serde_json::from_str(include_str!("../../../tests/fixtures/identity.json"))
                    .unwrap();
            spec["templateHash"] = json!(template.hash);
            spec["startIndex"] = json!("9");
            spec["count"] = json!(5);
            spec["output"] =
                json!({"format":"png","width":24,"height":24,"background":"transparent"});
            Self {
                directory,
                template,
                spec: serde_json::from_value(spec).unwrap(),
            }
        }
        fn execution<'a>(&'a self, token: &'a AtomicBool, threads: usize) -> Execution<'a> {
            Execution {
                template: &self.template,
                spec: &self.spec,
                directory: &self.directory,
                token,
                threads,
                batch_size: 3,
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            for entry in fs::read_dir(&self.directory).unwrap() {
                fs::remove_file(entry.unwrap().path()).unwrap();
            }
            fs::remove_dir(&self.directory).unwrap();
        }
    }
    fn lines(bytes: &[u8]) -> Vec<serde_json::Value> {
        std::str::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
    struct CancelAfterRecord<'a> {
        bytes: Vec<u8>,
        token: &'a AtomicBool,
    }
    impl Write for CancelAfterRecord<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            if lines(&self.bytes)
                .iter()
                .any(|line| line["kind"] == "variant")
            {
                self.token.store(true, Ordering::Relaxed);
            }
            Ok(())
        }
    }
    #[test]
    fn preset_cancellation_returns_130_without_artifacts_or_false_rejections() {
        let fixture = Fixture::new();
        let token = AtomicBool::new(true);
        let (mut stdout, mut manifest) = (Vec::new(), Vec::new());
        let error = generate(&fixture.execution(&token, 2), &mut stdout, &mut manifest)
            .err()
            .unwrap();
        assert_eq!(error.exit, 130);
        assert_eq!(fs::read_dir(&fixture.directory).unwrap().count(), 0);
        assert_eq!(
            lines(&manifest)[1],
            json!({"kind":"summary","status":"cancelled","accepted":0,"rejected":0})
        );
        assert_eq!(lines(&manifest)[1..], lines(&stdout));
    }
    #[test]
    fn cancellation_drains_published_siblings_then_stops_before_next_window() {
        for (threads, batch_size) in [(1, 1), (1, 3), (2, 2), (2, 3)] {
            let fixture = Fixture::new();
            let token = AtomicBool::new(false);
            let mut manifest = CancelAfterRecord {
                bytes: Vec::new(),
                token: &token,
            };
            let mut stdout = Vec::new();
            let mut execution = fixture.execution(&token, threads);
            execution.batch_size = batch_size;
            let error = generate(&execution, &mut stdout, &mut manifest)
                .err()
                .unwrap();
            assert_eq!(error.exit, 130);
            let records = lines(&manifest.bytes);
            assert_eq!(records.len(), threads + 2);
            assert_eq!(
                records.last().unwrap(),
                &json!({"kind":"summary","status":"cancelled","accepted":threads,"rejected":0})
            );
            assert_eq!(records[1..], lines(&stdout));
            assert_eq!(fs::read_dir(&fixture.directory).unwrap().count(), threads);
            for index in 9..9 + threads {
                assert!(
                    fixture
                        .directory
                        .join(format!("variant-{index}.png"))
                        .is_file()
                );
            }
        }
    }
    struct FailAfterLines {
        bytes: Vec<u8>,
        complete: usize,
        partial: bool,
        flush_error: bool,
    }
    impl Write for FailAfterLines {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.bytes.iter().filter(|&&b| b == b'\n').count() >= self.complete {
                if self.partial {
                    self.partial = false;
                    self.bytes.extend_from_slice(&bytes[..7.min(bytes.len())]);
                    return Ok(7.min(bytes.len()));
                }
                return Err(io::Error::other("injected disk full"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            if self.flush_error {
                Err(io::Error::other("injected flush error"))
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn manifest_short_write_or_flush_failure_is_io_error_without_success_summary() {
        for (complete, partial, flush_error) in
            [(0, false, false), (2, true, false), (0, false, true)]
        {
            let fixture = Fixture::new();
            let token = AtomicBool::new(false);
            let mut manifest = FailAfterLines {
                bytes: Vec::new(),
                complete,
                partial,
                flush_error,
            };
            let mut stdout = Vec::new();
            let error = generate(&fixture.execution(&token, 2), &mut stdout, &mut manifest)
                .err()
                .unwrap();
            assert_eq!(error.exit, 3);
            assert!(!String::from_utf8_lossy(&manifest.bytes).contains("completed"));
            assert!(!String::from_utf8_lossy(&stdout).contains("completed"));
            // Only complete PNG files may survive a manifest storage failure.
            for entry in fs::read_dir(&fixture.directory).unwrap() {
                let path = entry.unwrap().path();
                assert_eq!(path.extension().unwrap(), "png");
                assert!(fs::read(path).unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
            }
        }
    }
    #[test]
    fn stdout_failure_still_records_all_published_siblings_and_failed_summary() {
        let fixture = Fixture::new();
        let token = AtomicBool::new(false);
        let mut stdout = FailAfterLines {
            bytes: Vec::new(),
            complete: 0,
            partial: false,
            flush_error: false,
        };
        let mut manifest = Vec::new();
        let error = generate(&fixture.execution(&token, 2), &mut stdout, &mut manifest)
            .err()
            .unwrap();
        assert_eq!(error.exit, 3);
        let records = lines(&manifest);
        assert_eq!(records.len(), 4);
        assert_eq!(
            records[3],
            json!({"kind":"summary","status":"failed","accepted":2,"rejected":0})
        );
        assert_eq!(fs::read_dir(&fixture.directory).unwrap().count(), 2);
    }
    #[test]
    fn publication_error_does_not_lose_the_successful_siblings_record() {
        let fixture = Fixture::new();
        let token = AtomicBool::new(false);
        fs::write(fixture.directory.join("variant-9.png"), b"existing").unwrap();
        let (mut stdout, mut manifest) = (Vec::new(), Vec::new());
        let error = generate(&fixture.execution(&token, 2), &mut stdout, &mut manifest)
            .err()
            .unwrap();
        assert_eq!(error.exit, 3);
        let records = lines(&manifest);
        assert_eq!(records.len(), 3);
        assert_eq!(records[1]["result"]["index"], "10");
        assert_eq!(
            records[2],
            json!({"kind":"summary","status":"failed","accepted":1,"rejected":0})
        );
        assert_eq!(records[1..], lines(&stdout));
        assert_eq!(
            fs::read(fixture.directory.join("variant-9.png")).unwrap(),
            b"existing"
        );
    }
}
