use autodev_server::{router, AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("AUTODEV_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok();
    let state = AppState::new(secret);
    let listener = TcpListener::bind(("0.0.0.0", port)).await?;

    axum::serve(listener, router(state)).await?;
    Ok(())
}
