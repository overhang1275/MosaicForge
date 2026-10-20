use crate::{
    durable::Journal,
    http::{UNKNOWN_REQUEST, error},
};
use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use std::{
    io::{self, Read, Seek, SeekFrom},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
struct Stream {
    receiver: tokio::sync::mpsc::Receiver<io::Result<Bytes>>,
    _permit: Arc<tokio::sync::OwnedSemaphorePermit>,
}
impl futures_core::Stream for Stream {
    type Item = io::Result<Bytes>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
fn range(value: &str, total: u64) -> Option<(u64, u64)> {
    let s = value.strip_prefix("bytes=")?;
    if s.contains(',') || s.bytes().any(|b| !b.is_ascii_digit() && b != b'-') {
        return None;
    }
    let (a, b) = s.split_once('-')?;
    if total == 0 {
        return None;
    }
    if a.is_empty() {
        let n = b.parse::<u64>().ok()?;
        if n == 0 {
            return None;
        }
        return Some((total.saturating_sub(n), total - 1));
    }
    let start = a.parse::<u64>().ok()?;
    let end = if b.is_empty() {
        total - 1
    } else {
        b.parse::<u64>().ok()?.min(total - 1)
    };
    if start > end || start >= total {
        return None;
    }
    Some((start, end))
}
pub async fn respond(
    journal: Arc<Journal>,
    request: Request,
    permit: tokio::sync::OwnedSemaphorePermit,
) -> Response {
    let permit = Arc::new(permit);
    let hashing_permit = permit.clone();
    if request.method() != Method::GET && request.method() != Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let parts: Vec<_> = request.uri().path().split('/').collect();
    if parts.len() != 6 {
        return StatusCode::NOT_FOUND.into_response();
    }
    let id = parts[4].to_owned();
    let kind = parts[5].to_owned();
    let j = journal.clone();
    let artifact = match tokio::task::spawn_blocking(move || {
        let _permit = hashing_permit;
        j.artifact(&id, &kind)
    })
    .await
    {
        Ok(Ok(a)) => a,
        Ok(Err(e)) => {
            return error(
                StatusCode::from_u16(e.status).unwrap(),
                UNKNOWN_REQUEST,
                e.code,
            );
        }
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                UNKNOWN_REQUEST,
                "artifact_error",
            );
        }
    };
    let etag = format!("\"{}\"", artifact.hash);
    let total = artifact.length;
    let mut start = 0;
    let mut end = total.saturating_sub(1);
    let mut status = StatusCode::OK;
    if let Some(r) = request.headers().get(header::RANGE) {
        let use_range = request
            .headers()
            .get(header::IF_RANGE)
            .is_none_or(|h| h.to_str().ok() == Some(etag.as_str()));
        if use_range {
            match r.to_str().ok().and_then(|s| range(s, total)) {
                Some((a, b)) => {
                    start = a;
                    end = b;
                    status = StatusCode::PARTIAL_CONTENT;
                }
                None => {
                    return Response::builder()
                        .status(416)
                        .header(
                            header::CONTENT_TYPE,
                            if artifact.kind == "pdf" {
                                "application/pdf"
                            } else {
                                "application/x-ndjson"
                            },
                        )
                        .header(header::CONTENT_LENGTH, 0)
                        .header(header::CONTENT_RANGE, format!("bytes */{total}"))
                        .header(header::ETAG, etag)
                        .header(header::ACCEPT_RANGES, "bytes")
                        .body(Body::empty())
                        .unwrap();
                }
            }
        }
    }
    let length = if total == 0 { 0 } else { end - start + 1 };
    let mut builder = Response::builder()
        .status(status)
        .header(
            header::CONTENT_TYPE,
            if artifact.kind == "pdf" {
                "application/pdf"
            } else {
                "application/x-ndjson"
            },
        )
        .header(header::CONTENT_LENGTH, length)
        .header(header::ETAG, etag)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    if status == StatusCode::PARTIAL_CONTENT {
        builder = builder.header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{total}"),
        );
    }
    if request.method() == Method::HEAD {
        return builder.body(Body::empty()).unwrap();
    }
    let (sender, receiver) = tokio::sync::mpsc::channel(1);
    let reader_permit = permit.clone();
    tokio::task::spawn_blocking(move || {
        let _permit = reader_permit;
        let mut file = artifact.file;
        let mut remaining = length;
        if let Err(e) = file.seek(SeekFrom::Start(start)) {
            let _ = sender.blocking_send(Err(e));
            return;
        }
        while remaining > 0 {
            if journal.is_cancelled(&artifact.job_id) {
                let _ = sender.blocking_send(Err(io::Error::other("cancelled")));
                return;
            }
            let mut bytes = vec![0; remaining.min(65536) as usize];
            match file.read(&mut bytes) {
                Ok(0) => {
                    let _ = sender.blocking_send(Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "truncated artifact",
                    )));
                    return;
                }
                Ok(n) => {
                    bytes.truncate(n);
                    remaining -= n as u64;
                    if sender.blocking_send(Ok(Bytes::from(bytes))).is_err() {
                        return;
                    }
                }
                Err(e) => {
                    let _ = sender.blocking_send(Err(e));
                    return;
                }
            }
        }
    });
    builder
        .body(Body::from_stream(Stream {
            receiver,
            _permit: permit,
        }))
        .unwrap()
}
