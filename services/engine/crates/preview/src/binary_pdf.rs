use crate::{contract::*, http::UNKNOWN_REQUEST, validation::*};
use axum::{
    body::{Body, Bytes},
    http::{StatusCode, header},
    response::Response,
};
use mosaicforge_core::{EngineError, template::import_svg};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, Write},
    path::PathBuf,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll},
};
use tokio::sync::{OwnedSemaphorePermit, mpsc};

const MAX_DISK_BYTES: u64 = 1024 * 1024 * 1024;
const CHUNK_BYTES: usize = 64 * 1024;
static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);

struct TemporaryPdf {
    file: Option<File>,
    directory: PathBuf,
    written: u64,
    limit: u64,
    exceeded: bool,
}
impl TemporaryPdf {
    fn new(limit: u64) -> io::Result<Self> {
        for _ in 0..100 {
            let directory = std::env::temp_dir().join(format!(
                "mosaicforge-pdf-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(io::Error::other)?
                    .as_nanos(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            let mut builder = std::fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&directory) {
                Ok(()) => {
                    let mut temporary = Self {
                        file: None,
                        directory,
                        written: 0,
                        limit,
                        exceeded: false,
                    };
                    let mut options = OpenOptions::new();
                    options.create_new(true).read(true).write(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        options.mode(0o600);
                    }
                    temporary.file = Some(options.open(temporary.directory.join("document.pdf"))?);
                    return Ok(temporary);
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::other("cannot allocate private PDF destination"))
    }
}
impl Drop for TemporaryPdf {
    fn drop(&mut self) {
        drop(self.file.take());
        let _ = std::fs::remove_file(self.directory.join("document.pdf"));
        let _ = std::fs::remove_dir(&self.directory);
    }
}
impl Write for TemporaryPdf {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() as u64 > self.limit.saturating_sub(self.written) {
            self.exceeded = true;
            return Err(io::Error::other("PDF exceeds temporary disk quota"));
        }
        let written = self.file.as_mut().unwrap().write(bytes)?;
        self.written += written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file.as_mut().unwrap().flush()
    }
}

type Failure = (StatusCode, String, &'static str);
struct PreparedPdf {
    temporary: TemporaryPdf,
    metadata: String,
}

fn prepare(bytes: &[u8], limit: u64) -> Result<PreparedPdf, Failure> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            UNKNOWN_REQUEST.into(),
            "invalid_request",
        )
    })?;
    let id = value["requestId"]
        .as_str()
        .filter(|s| {
            s.len() == 36
                && s.bytes().enumerate().all(|(i, b)| {
                    if [8, 13, 18, 23].contains(&i) {
                        b == b'-'
                    } else {
                        b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
                    }
                })
        })
        .unwrap_or(UNKNOWN_REQUEST)
        .to_owned();
    let invalid = || (StatusCode::BAD_REQUEST, id.clone(), "invalid_request");
    let internal = || {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            id.clone(),
            "internal_error",
        )
    };
    let engine_error = |e| match e {
        EngineError::InvalidSvg(_) => (StatusCode::BAD_REQUEST, id.clone(), "invalid_template"),
        EngineError::InvalidSpec(_) => invalid(),
        _ => internal(),
    };
    let request: SelectedPdfExportRequest = serde_json::from_value(value).map_err(|_| invalid())?;
    let indices = validate_selected_pdf_export_request(&request).map_err(|_| invalid())?;
    let template = import_svg(request.template_svg.as_bytes()).map_err(engine_error)?;
    let mut temporary = TemporaryPdf::new(limit).map_err(|_| internal())?;
    let document = mosaicforge_cpu::pdf::generate_selected_pdf(
        &template,
        &request.job,
        &indices,
        &mut temporary,
        |_| Ok(()),
    );
    if temporary.exceeded {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, id, "payload_too_large"));
    }
    let document = document.map_err(engine_error)?;
    let metadata: PdfDownloadResponse = serde_json::from_value(json!({"schemaVersion":"3", "requestId":id, "document":document, "variantIndices":request.variant_indices}))
        .map_err(|_| internal())?;
    validate_pdf_download_response(&metadata).map_err(|_| internal())?;
    temporary.flush().map_err(|_| internal())?;
    temporary
        .file
        .as_mut()
        .unwrap()
        .rewind()
        .map_err(|_| internal())?;
    let metadata = serde_json::to_string(&metadata).map_err(|_| internal())?;
    if !metadata.is_ascii() {
        return Err(internal());
    }
    Ok(PreparedPdf {
        temporary,
        metadata,
    })
}

struct PdfStream {
    receiver: mpsc::Receiver<io::Result<Bytes>>,
    _permit: Arc<OwnedSemaphorePermit>,
}
impl futures_core::Stream for PdfStream {
    type Item = io::Result<Bytes>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

fn stream(prepared: PreparedPdf, permit: OwnedSemaphorePermit) -> Response {
    let PreparedPdf {
        mut temporary,
        metadata,
    } = prepared;
    let length = temporary.written;
    let permit = Arc::new(permit);
    let reader_permit = permit.clone();
    let (sender, receiver) = mpsc::channel(1);
    tokio::task::spawn_blocking(move || {
        let _permit = reader_permit;
        loop {
            let mut bytes = vec![0; CHUNK_BYTES];
            match temporary.file.as_mut().unwrap().read(&mut bytes) {
                Ok(0) => break,
                Ok(count) => {
                    bytes.truncate(count);
                    if sender.blocking_send(Ok(Bytes::from(bytes))).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    let _ = sender.blocking_send(Err(e));
                    break;
                }
            }
        }
        // Delete the private file before the reader's permit is released.
        drop(temporary);
    });
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_LENGTH, length)
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .header("x-mosaicforge-result", metadata)
        .body(Body::from_stream(PdfStream {
            receiver,
            _permit: permit,
        }))
        .unwrap()
}

async fn generate<F>(
    permit: OwnedSemaphorePermit,
    work: F,
) -> Result<(Result<PreparedPdf, Failure>, OwnedSemaphorePermit), tokio::task::JoinError>
where
    F: FnOnce() -> Result<PreparedPdf, Failure> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        // Cancellation drops completed files; a running generation retains its permit.
        let result = work();
        (result, permit)
    })
    .await
}

pub(crate) async fn respond(bytes: Bytes, permit: OwnedSemaphorePermit) -> Response {
    let result = generate(permit, move || prepare(&bytes, MAX_DISK_BYTES)).await;
    match result {
        Ok((Ok(prepared), permit)) => stream(prepared, permit),
        Ok((Err((status, id, code)), _)) => crate::http::error(status, &id, code),
        Err(_) => crate::http::error(
            StatusCode::INTERNAL_SERVER_ERROR,
            UNKNOWN_REQUEST,
            "internal_error",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use tokio::sync::Semaphore;

    fn input() -> Vec<u8> {
        let mut request: Value = serde_json::from_str(include_str!(
            "../../../../../contracts/preview/v1/fixtures/export-pdf.json"
        ))
        .unwrap();
        request["schemaVersion"] = json!("2");
        request["variantIndices"] = json!(["0", "2", "18446744073709551615"]);
        serde_json::to_vec(&request).unwrap()
    }

    #[test]
    fn quota_failure_does_not_publish_a_document_and_deletes_private_file() {
        let failure = prepare(&input(), 16).err().unwrap();
        assert_eq!(failure.0, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(failure.2, "payload_too_large");
        let mut temporary = TemporaryPdf::new(16).unwrap();
        let directory = temporary.directory.clone();
        temporary.write_all(b"partial").unwrap();
        assert!(temporary.write_all(&[0; 17]).is_err());
        assert_eq!(
            temporary.file.as_ref().unwrap().metadata().unwrap().len(),
            7
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        drop(temporary);
        assert!(!directory.exists());
    }

    #[tokio::test]
    async fn stream_holds_gate_and_cleans_up_on_success_and_disconnect() {
        for disconnect in [false, true] {
            let gate = Arc::new(Semaphore::new(1));
            let permit = gate.clone().try_acquire_owned().unwrap();
            let mut temporary = TemporaryPdf::new(MAX_DISK_BYTES).unwrap();
            let directory = temporary.directory.clone();
            temporary.write_all(&vec![42; CHUNK_BYTES * 4]).unwrap();
            temporary.file.as_mut().unwrap().rewind().unwrap();
            let mut body = stream(
                PreparedPdf {
                    temporary,
                    metadata: "{}".into(),
                },
                permit,
            )
            .into_body();
            assert!(gate.clone().try_acquire_owned().is_err());
            if !disconnect {
                let mut length = 0;
                while let Some(frame) = body.frame().await {
                    let bytes = frame.unwrap().into_data().unwrap();
                    assert!(bytes.len() <= CHUNK_BYTES);
                    length += bytes.len();
                }
                assert_eq!(length, CHUNK_BYTES * 4);
                assert!(gate.clone().try_acquire_owned().is_err());
            }
            drop(body);
            let _released = tokio::time::timeout(std::time::Duration::from_secs(2), gate.acquire())
                .await
                .unwrap()
                .unwrap();
            assert!(!directory.exists());
        }
    }

    #[test]
    fn invalid_selection_and_rejected_pages_never_prepare_a_pdf() {
        let mut request: Value = serde_json::from_slice(&input()).unwrap();
        for indices in [json!(["2", "0", "3"]), json!(["0", "0", "2"])] {
            request["variantIndices"] = indices;
            assert_eq!(
                prepare(&serde_json::to_vec(&request).unwrap(), MAX_DISK_BYTES)
                    .err()
                    .unwrap()
                    .0,
                StatusCode::BAD_REQUEST
            );
        }
        request["variantIndices"] = json!(["0", "2", "18446744073709551615"]);
        request["job"]["generation"]["rules"]["translateX"] = json!({"min":10000,"max":10000});
        assert!(prepare(&serde_json::to_vec(&request).unwrap(), MAX_DISK_BYTES).is_err());
    }

    #[tokio::test]
    async fn disconnect_during_generation_retains_gate_until_file_is_deleted() {
        let gate = Arc::new(Semaphore::new(1));
        let permit = gate.clone().try_acquire_owned().unwrap();
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let (path_sender, path_receiver) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(generate(permit, move || {
            let prepared = prepare(&input(), MAX_DISK_BYTES)?;
            path_sender
                .send(prepared.temporary.directory.clone())
                .unwrap();
            started.send(()).unwrap();
            wait.recv().unwrap();
            Ok(prepared)
        }));
        ready.await.unwrap();
        let directory = path_receiver.await.unwrap();
        assert!(directory.exists());
        task.abort();
        let _ = task.await;
        assert!(gate.clone().try_acquire_owned().is_err());
        release.send(()).unwrap();
        let _released = tokio::time::timeout(std::time::Duration::from_secs(2), gate.acquire())
            .await
            .unwrap()
            .unwrap();
        assert!(!directory.exists());
    }
}
