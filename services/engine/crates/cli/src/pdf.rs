use crate::{
    Failure,
    generation::{
        cancelled, check_cancelled, create_manifest, engine_failure, install_cancellation,
        invalid_spec, read_spec_bytes, write_record,
    },
    io_failure,
    publish::PendingFile,
    read_template,
};
use mosaicforge_core::{
    EngineError,
    contract::PdfJobSpec,
    pdf::{pdf_configuration_hash, validate_pdf_job},
    template::Template,
};
use mosaicforge_cpu::pdf::generate_pdf_cancellable;
use serde_json::json;
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

pub fn run(args: &clap::ArgMatches, out: &mut impl Write) -> Result<(), Failure> {
    let token = install_cancellation()?;
    let template = read_template(
        args.get_one::<PathBuf>("template")
            .expect("required by clap"),
    )?;
    let job: PdfJobSpec = serde_json::from_slice(&read_spec_bytes(
        args.get_one::<PathBuf>("spec").expect("required by clap"),
    )?)
    .map_err(|_| invalid_spec("invalid PdfJobSpec JSON"))?;
    validate_pdf_job(&job).map_err(engine_failure)?;
    if template.hash.as_str() != job.generation.template_hash.as_str() {
        return Err(invalid_spec(
            "templateHash does not match the normalized SVG",
        ));
    }
    let directory = args.get_one::<PathBuf>("out").expect("required by clap");
    let batch_size = *args
        .get_one::<u64>("batch-size")
        .expect("defaulted by clap");
    let mut manifest = create_manifest(directory)?;
    generate(
        &template,
        &job,
        batch_size,
        &token,
        out,
        &mut manifest,
        directory,
    )
}

fn generate(
    template: &Template,
    job: &PdfJobSpec,
    batch_size: u64,
    token: &AtomicBool,
    out: &mut impl Write,
    manifest: &mut impl Write,
    directory: &Path,
) -> Result<(), Failure> {
    write_record(manifest,&json!({"kind":"job","format":"pdf","spec":job,"configurationHash":pdf_configuration_hash(job).map_err(engine_failure)?,"backend":"cpu"}))
        .map_err(|_|io_failure("cannot write manifest"))?;
    let (mut accepted, mut rejected) = (0u64, 0u64);
    let mut manifest_broken = false;
    let outcome = (|| -> Result<(), Failure> {
        check_cancelled(token)?;
        let mut pending = PendingFile::create(directory)
            .map_err(|_| io_failure("cannot create PDF temporary"))?;
        let mut callback_failure = None;
        let result = generate_pdf_cancellable(
            template,
            job,
            batch_size,
            pending.writer(),
            |record| {
                let value = json!({"kind":"pdf_page","record":record});
                if write_record(manifest, &value).is_err() {
                    manifest_broken = true;
                    callback_failure = Some(io_failure("cannot write manifest"));
                    return Err(EngineError::RenderFailed("PDF progress write failed"));
                }
                if record.page_number.is_some() {
                    accepted += 1;
                } else {
                    rejected += 1;
                }
                if write_record(out, &value).is_err() {
                    callback_failure = Some(io_failure("cannot write command output"));
                    return Err(EngineError::RenderFailed("PDF progress write failed"));
                }
                Ok(())
            },
            token,
        );
        if let Some(failure) = callback_failure {
            return Err(failure);
        }
        let result = result.map_err(engine_failure)?.ok_or_else(cancelled)?;
        check_cancelled(token)?;
        let artifact = if result.document_hash.is_some() {
            pending
                .publish(&directory.join("mosaic.pdf"))
                .map_err(|_| io_failure("cannot publish PDF without replacement"))?;
            Some("mosaic.pdf")
        } else {
            drop(pending);
            None
        };
        let value = json!({"kind":"pdf_document","result":result,"artifact":artifact});
        write_record(manifest, &value).map_err(|_| {
            manifest_broken = true;
            io_failure("cannot write manifest")
        })?;
        write_record(out, &value).map_err(|_| io_failure("cannot write command output"))?;
        Ok(())
    })();
    if manifest_broken {
        return outcome;
    }
    let status = match outcome.as_ref() {
        Ok(()) => "completed",
        Err(e) if e.exit == 130 => "cancelled",
        Err(_) => "failed",
    };
    let summary = json!({"kind":"summary","status":status,"accepted":accepted,"rejected":rejected});
    write_record(manifest, &summary).map_err(|_| io_failure("cannot write manifest"))?;
    write_record(out, &summary).map_err(|_| io_failure("cannot write command output"))?;
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs, io,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture {
        directory: PathBuf,
        template: Template,
        job: PdfJobSpec,
    }
    impl Fixture {
        fn new(count: u64) -> Self {
            let directory = std::env::temp_dir().join(format!(
                "mosaicforge-pdf-cli-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&directory).unwrap();
            let template=mosaicforge_core::template::import_svg(br##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><g><rect width="100" height="100" fill="#123456"/></g></svg>"##).unwrap();
            let mut generation: serde_json::Value =
                serde_json::from_str(include_str!("../../../tests/fixtures/identity.json"))
                    .unwrap();
            generation["templateHash"] = json!(template.hash);
            generation["count"] = json!(count);
            let job=serde_json::from_value(json!({"schemaVersion":"1","generation":generation,"pdf":{"exporterVersion":"1","trimWidthMm":100,"trimHeightMm":100,"bleedMm":0}})).unwrap();
            Self {
                directory,
                template,
                job,
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
    fn records(bytes: &[u8]) -> Vec<serde_json::Value> {
        std::str::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
    struct CancelOnPage<'a> {
        bytes: Vec<u8>,
        token: &'a AtomicBool,
    }
    impl Write for CancelOnPage<'_> {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            if records(&self.bytes).iter().any(|r| r["kind"] == "pdf_page") {
                self.token.store(true, Ordering::Relaxed);
            }
            Ok(())
        }
    }
    #[test]
    fn cancellation_before_during_and_after_last_page_never_publishes_pdf() {
        for (count, preset) in [(3, true), (3, false), (1, false)] {
            let fixture = Fixture::new(count);
            let token = AtomicBool::new(preset);
            let mut manifest = CancelOnPage {
                bytes: Vec::new(),
                token: &token,
            };
            let mut stdout = Vec::new();
            let error = generate(
                &fixture.template,
                &fixture.job,
                7,
                &token,
                &mut stdout,
                &mut manifest,
                &fixture.directory,
            )
            .err()
            .unwrap();
            assert_eq!(error.exit, 130);
            assert_eq!(
                fs::read_dir(&fixture.directory).unwrap().count(),
                0,
                "no PDF or temporary survives"
            );
            let values = records(&manifest.bytes);
            assert_eq!(
                values.last().unwrap(),
                &json!({"kind":"summary","status":"cancelled","accepted":if preset {0}else{1},"rejected":0})
            );
            assert!(!values.iter().any(|r| r["kind"] == "pdf_document"));
            assert_eq!(values[1..], records(&stdout));
        }
    }
    struct FailAtRecord {
        bytes: Vec<u8>,
        allowed: usize,
        partial: bool,
        flush_error: bool,
    }
    impl Write for FailAtRecord {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.bytes.iter().filter(|&&b| b == b'\n').count() >= self.allowed {
                if self.partial {
                    self.partial = false;
                    let n = bytes.len().min(7);
                    self.bytes.extend_from_slice(&bytes[..n]);
                    return Ok(n);
                }
                return Err(io::Error::other("injected full disk"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            if self.flush_error {
                Err(io::Error::other("injected flush failure"))
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn manifest_or_stdout_failure_before_commit_discards_pdf_temporary() {
        for (fail_stdout, partial, flush_error) in [
            (false, false, false),
            (false, true, false),
            (false, false, true),
            (true, false, false),
        ] {
            let fixture = Fixture::new(3);
            let token = AtomicBool::new(false);
            let mut broken = FailAtRecord {
                bytes: Vec::new(),
                allowed: 1,
                partial,
                flush_error,
            };
            let mut healthy = Vec::new();
            let outcome = if fail_stdout {
                generate(
                    &fixture.template,
                    &fixture.job,
                    7,
                    &token,
                    &mut broken,
                    &mut healthy,
                    &fixture.directory,
                )
            } else {
                generate(
                    &fixture.template,
                    &fixture.job,
                    7,
                    &token,
                    &mut healthy,
                    &mut broken,
                    &fixture.directory,
                )
            };
            assert_eq!(outcome.err().unwrap().exit, 3);
            assert_eq!(fs::read_dir(&fixture.directory).unwrap().count(), 0);
            for bytes in [&broken.bytes, &healthy] {
                assert!(!String::from_utf8_lossy(bytes).contains("pdf_document"));
                assert!(!String::from_utf8_lossy(bytes).contains("completed"));
            }
        }
    }
    #[test]
    fn manifest_failure_after_commit_can_only_leave_a_complete_pdf() {
        let fixture = Fixture::new(1);
        let token = AtomicBool::new(false);
        let mut manifest = FailAtRecord {
            bytes: Vec::new(),
            allowed: 2,
            partial: false,
            flush_error: false,
        };
        let mut stdout = Vec::new();
        let error = generate(
            &fixture.template,
            &fixture.job,
            7,
            &token,
            &mut stdout,
            &mut manifest,
            &fixture.directory,
        )
        .err()
        .unwrap();
        assert_eq!(error.exit, 3);
        assert_eq!(fs::read_dir(&fixture.directory).unwrap().count(), 1);
        let bytes = fs::read(fixture.directory.join("mosaic.pdf")).unwrap();
        assert!(bytes.starts_with(b"%PDF-1.7"));
        assert!(bytes.ends_with(b"%%EOF\n"));
        assert!(!String::from_utf8_lossy(&stdout).contains("pdf_document"));
        assert!(!String::from_utf8_lossy(&manifest.bytes).contains("completed"));
    }
}
