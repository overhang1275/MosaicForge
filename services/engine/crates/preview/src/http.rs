use axum::{
    Router,
    body::to_bytes,
    extract::{Request, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub const MAX_BODY_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

pub const UNKNOWN_REQUEST: &str = "00000000-0000-0000-0000-000000000000";
#[derive(Clone)]
struct ServerState {
    token: String,
    host: String,
    gate: Arc<Semaphore>,
    cancel_gate: Arc<Semaphore>,
    download_gate: Arc<Semaphore>,
    journal: Option<Arc<crate::durable::Journal>>,
}

pub fn valid_token(token: &str) -> bool {
    (32..=256).contains(&token.len()) && token.bytes().all(|b| b.is_ascii_graphic())
}
pub fn router(token: String, host: String) -> Router {
    configured_router(token, host, None)
}
fn configured_router(
    token: String,
    host: String,
    journal: Option<Arc<crate::durable::Journal>>,
) -> Router {
    let mut routes = Router::new()
        .route("/v1/inspect", any(handle))
        .route("/v1/preview", any(handle))
        .route("/v1/export", any(handle))
        .route("/v1/export-pdf", any(handle))
        .route("/v2/export-pdf-selection", any(handle))
        .route("/v3/export-pdf-selection", any(handle));
    if journal.is_some() {
        routes = routes
            .route("/internal/v1/blocks", any(handle))
            .route("/internal/v1/finalize", any(handle))
            .route("/internal/v1/cancel", any(handle))
            .route("/internal/v1/artifacts/{artifactId}/{kind}", any(handle));
    }
    routes.with_state(ServerState {
        token,
        host,
        gate: Arc::new(Semaphore::new(1)),
        cancel_gate: Arc::new(Semaphore::new(4)),
        download_gate: Arc::new(Semaphore::new(4)),
        journal,
    })
}
pub(crate) fn error(status: StatusCode, id: &str, code: &str) -> Response {
    json_response(
        status,
        json!({"requestId":id,"code":code,"message":"Preview request could not be completed"})
            .to_string()
            .into_bytes(),
    )
}
fn json_response(status: StatusCode, bytes: Vec<u8>) -> Response {
    (
        status,
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        ],
        bytes,
    )
        .into_response()
}
async fn blocking<F>(
    permit: OwnedSemaphorePermit,
    work: F,
) -> Result<(u16, Vec<u8>), tokio::task::JoinError>
where
    F: FnOnce() -> (u16, Vec<u8>) + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        work()
    })
    .await
}
async fn handle(State(state): State<ServerState>, request: Request) -> Response {
    let trace = request
        .headers()
        .get("traceparent")
        .filter(|value| {
            let bytes = value.as_bytes();
            bytes.len() == 55
                && bytes.starts_with(b"00-")
                && bytes[35] == b'-'
                && bytes[52] == b'-'
                && bytes.iter().enumerate().all(|(i, b)| {
                    [2, 35, 52].contains(&i) || b.is_ascii_digit() || (b'a'..=b'f').contains(b)
                })
                && bytes[3..35].iter().any(|b| *b != b'0')
                && bytes[36..52].iter().any(|b| *b != b'0')
        })
        .cloned();
    let mut response = handle_inner(state, request).await;
    if let Some(trace) = trace {
        response.headers_mut().insert("traceparent", trace);
    }
    response
}
async fn handle_inner(state: ServerState, request: Request) -> Response {
    let headers = request.headers();
    if headers.get(header::HOST).and_then(|h| h.to_str().ok()) != Some(state.host.as_str())
        || headers.contains_key(header::ORIGIN)
    {
        return error(StatusCode::FORBIDDEN, UNKNOWN_REQUEST, "forbidden");
    }
    if headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        != Some(format!("Bearer {}", state.token).as_str())
    {
        return error(StatusCode::UNAUTHORIZED, UNKNOWN_REQUEST, "unauthorized");
    }
    if request.uri().path().starts_with("/internal/v1/artifacts/") {
        let permit = match state.download_gate.try_acquire_owned() {
            Ok(p) => p,
            Err(_) => return error(StatusCode::TOO_MANY_REQUESTS, UNKNOWN_REQUEST, "busy"),
        };
        return crate::artifact_http::respond(
            state.journal.expect("configured journal"),
            request,
            permit,
        )
        .await;
    }
    if request.method() != axum::http::Method::POST {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    if !headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|v| {
            v.split(';')
                .next()
                .is_some_and(|m| m.trim().eq_ignore_ascii_case("application/json"))
        })
    {
        return error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            UNKNOWN_REQUEST,
            "unsupported_media_type",
        );
    }
    if headers
        .get(header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .is_some_and(|n| n > MAX_BODY_BYTES as u64)
    {
        return error(
            StatusCode::PAYLOAD_TOO_LARGE,
            UNKNOWN_REQUEST,
            "payload_too_large",
        );
    }
    let path = request.uri().path().to_owned();
    let cancellation = path == "/internal/v1/cancel";
    let gate = if cancellation {
        state.cancel_gate.clone()
    } else {
        state.gate.clone()
    };
    let permit = match gate.try_acquire_owned() {
        Ok(p) => p,
        Err(_) => return error(StatusCode::TOO_MANY_REQUESTS, UNKNOWN_REQUEST, "busy"),
    };
    let bytes = match tokio::time::timeout(
        Duration::from_secs(30),
        to_bytes(request.into_body(), MAX_BODY_BYTES),
    )
    .await
    {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(e)) => {
            use std::error::Error;
            let size = e
                .source()
                .is_some_and(|s| s.is::<http_body_util::LengthLimitError>());
            return error(
                if size {
                    StatusCode::PAYLOAD_TOO_LARGE
                } else {
                    StatusCode::BAD_REQUEST
                },
                UNKNOWN_REQUEST,
                if size {
                    "payload_too_large"
                } else {
                    "invalid_request"
                },
            );
        }
        Err(_) => return error(StatusCode::BAD_REQUEST, UNKNOWN_REQUEST, "invalid_request"),
    };
    // The blocking closure owns the permit even if its HTTP future is dropped.
    if path == "/v3/export-pdf-selection" {
        return crate::binary_pdf::respond(bytes, permit).await;
    }
    match blocking(permit, move || {
        if path.starts_with("/internal/v1/") {
            let journal=state.journal.expect("configured journal route");
            let result=match path.as_str(){"/internal/v1/blocks"=>journal.execute(&bytes),"/internal/v1/finalize"=>journal.finalize(&bytes),"/internal/v1/cancel"=>journal.cancel(&bytes),_=>unreachable!()};
            return match result{
                Ok(bytes)=>(200,bytes),
                Err(e)=>(e.status,json!({"requestId":UNKNOWN_REQUEST,"code":e.code,"message":"Block operation could not be completed"}).to_string().into_bytes()),
            };
        }
        crate::compute::execute(&path, &bytes)
    }).await {
        Ok((status, bytes)) => json_response(
            StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            bytes,
        ),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            UNKNOWN_REQUEST,
            "internal_error",
        ),
    }
}
pub fn router_with_journal(
    token: String,
    host: String,
    root: &std::path::Path,
) -> std::io::Result<Router> {
    let journal = crate::durable::Journal::open(root, crate::durable::JournalLimits::default())?;
    Ok(configured_router(token, host, Some(Arc::new(journal))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn cancellation_bypasses_occupied_compute_gate() {
        use tower::ServiceExt;
        let dir =
            std::env::temp_dir().join(format!("mosaic-cancel-admission-{}", uuid::Uuid::new_v4()));
        let gate = Arc::new(Semaphore::new(1));
        let _permit = gate.clone().try_acquire_owned().unwrap();
        let journal = Arc::new(crate::durable::Journal::open(&dir, Default::default()).unwrap());
        let state = ServerState {
            token: "test-token".into(),
            host: "worker".into(),
            gate,
            cancel_gate: Arc::new(Semaphore::new(4)),
            download_gate: Arc::new(Semaphore::new(4)),
            journal: Some(journal.clone()),
        };
        let app = Router::new()
            .route("/internal/v1/cancel", any(handle))
            .with_state(state);
        let request = axum::http::Request::post("/internal/v1/cancel")
            .header("host", "worker")
            .header("authorization", "Bearer test-token")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                r#"{"schemaVersion":"1","jobId":"12345678-1234-1234-1234-123456789abc"}"#,
            ))
            .unwrap();
        assert_eq!(app.oneshot(request).await.unwrap().status(), 200);
        assert!(journal.is_cancelled("12345678-1234-1234-1234-123456789abc"));
        drop(journal);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[tokio::test]
    async fn successful_request_preserves_valid_trace_context() {
        use tower::ServiceExt;
        let trace = "00-1234567890abcdef1234567890abcdef-1234567890abcdef-01";
        let app = router("test-token".into(), "127.0.0.1:4771".into());
        let request = axum::http::Request::post("/v1/inspect")
            .header("host", "127.0.0.1:4771")
            .header("authorization", "Bearer test-token")
            .header("content-type", "application/json")
            .header("traceparent", trace)
            .body(axum::body::Body::from(r#"{"requestId":"12345678-1234-1234-1234-123456789abc","templateSvg":"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"100\"><rect width=\"100\" height=\"100\"/></svg>"}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get("traceparent").unwrap(), trace);
    }
    #[tokio::test]
    async fn occupied_gate_returns_busy_without_reading_body() {
        use tower::ServiceExt;
        let gate = Arc::new(Semaphore::new(1));
        let _permit = gate.clone().try_acquire_owned().unwrap();
        let state = ServerState {
            token: "test-token".into(),
            host: "127.0.0.1:4771".into(),
            gate,
            cancel_gate: Arc::new(Semaphore::new(4)),
            download_gate: Arc::new(Semaphore::new(4)),
            journal: None,
        };
        let app = Router::new()
            .route("/v1/inspect", any(handle))
            .with_state(state);
        let request = axum::http::Request::post("/v1/inspect")
            .header("host", "127.0.0.1:4771")
            .header("authorization", "Bearer test-token")
            .header("content-type", "application/json")
            .body(axum::body::Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap()["code"],
            "busy"
        );
    }
    #[tokio::test]
    async fn disconnect_retains_permit_and_panic_releases_it() {
        let gate = Arc::new(Semaphore::new(1));
        let permit = gate.clone().try_acquire_owned().unwrap();
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let task = tokio::spawn(blocking(permit, move || {
            started.send(()).unwrap();
            wait.recv().unwrap();
            (200, Vec::new())
        }));
        ready.await.unwrap();
        task.abort();
        let _ = task.await;
        assert!(gate.clone().try_acquire_owned().is_err());
        release.send(()).unwrap();
        let permit = tokio::time::timeout(Duration::from_secs(2), gate.clone().acquire_owned())
            .await
            .unwrap()
            .unwrap();
        assert!(
            blocking(permit, || panic!("simulated compute failure"))
                .await
                .is_err()
        );
        assert!(gate.try_acquire_owned().is_ok());
    }
}
