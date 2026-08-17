use std::sync::Arc;

use autodev_server::{router, AppState, CodexProcessService};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("AUTODEV_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok();
    let codex_binary = std::env::var("AUTODEV_CODEX_BIN").unwrap_or_else(|_| "codex".into());
    let state = match CodexProcessService::spawn(&codex_binary, env!("CARGO_PKG_VERSION")) {
        Ok(service) => AppState::with_codex_service(secret, Arc::new(service)),
        Err(_) => {
            eprintln!(
                "Codex subscription provider unavailable; auth endpoints will return 503"
            );
            AppState::new(secret)
        }
    };
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;

    axum::serve(listener, router(state)).await?;
    Ok(())
}
