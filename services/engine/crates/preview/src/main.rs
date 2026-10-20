use mosaicforge_preview::http::{router, valid_token};
#[tokio::main]
async fn main() -> std::process::ExitCode {
    if run().await.is_err() {
        eprintln!("Preview server failed: check private token, port and loopback availability");
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}
async fn run() -> Result<(), ()> {
    let token = std::env::var("MOSAICFORGE_ENGINE_TOKEN").map_err(|_| ())?;
    if !valid_token(&token) {
        return Err(());
    }
    let port = std::env::var("MOSAICFORGE_ENGINE_PORT")
        .unwrap_or_else(|_| "4771".into())
        .parse::<u16>()
        .map_err(|_| ())?;
    if port == 0 {
        return Err(());
    }
    let host = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&host).await.map_err(|_| ())?;
    eprintln!("MosaicForge preview listening on {host}");
    let app = match std::env::var_os("MOSAICFORGE_JOURNAL_DIR") {
        Some(root) => {
            mosaicforge_preview::http::router_with_journal(token, host, std::path::Path::new(&root))
                .map_err(|_| ())?
        }
        None => router(token, host),
    };
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .map_err(|_| ())
}
