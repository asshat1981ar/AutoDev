use autodev_server::{router, AppState};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("AUTODEV_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    // AUTODEV_BIND controls the network interface the server listens on.
    // The default is `0.0.0.0` to keep the GitHub webhook public-surface
    // contract unchanged, but this also exposes the bearer-token-protected
    // MCP route to the LAN. A loud startup warning makes the trade-off
    // visible; production deployments are expected to run behind a TLS-
    // terminating reverse proxy that enforces the same network policy.
    let bind_addr: std::net::IpAddr = std::env::var("AUTODEV_BIND")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| "0.0.0.0".parse().expect("static 0.0.0.0 is valid"));
    let github_secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok();
    let mcp_bearer_token = std::env::var("AUTODEV_MCP_BEARER_TOKEN").ok();

    if bind_addr.is_unspecified() && mcp_bearer_token.is_some() {
        eprintln!(
            "warning: AUTODEV_BIND is the unspecified address and \
             AUTODEV_MCP_BEARER_TOKEN is set; the bearer-token-protected \
             /mcp route is reachable from the LAN. Bind to 127.0.0.1 \
             for local-only use, or run behind a reverse proxy that \
             enforces the same network policy."
        );
    }

    let mut state = AppState::new(github_secret);
    if let Some(token) = mcp_bearer_token {
        state = state.with_mcp_bearer_token(token);
    }

    let listener = TcpListener::bind((bind_addr, port)).await?;
    axum::serve(listener, router(state)).await?;
    Ok(())
}
