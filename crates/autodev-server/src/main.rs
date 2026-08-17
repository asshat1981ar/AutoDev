use autodev_server::{router, AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("AUTODEV_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let github_secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok();
    let mcp_bearer_token = std::env::var("AUTODEV_MCP_BEARER_TOKEN").ok();

    let mut state = AppState::new(github_secret);
    if let Some(token) = mcp_bearer_token {
        state = state.with_mcp_bearer_token(token);
    }

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, router(state)).await?;
    Ok(())
}
