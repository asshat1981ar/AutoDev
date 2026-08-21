use autodev_server::{router, AppState};
use tokio::net::TcpListener;

fn resolve_bind_addr(configured: Option<&str>) -> String {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("127.0.0.1")
        .to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("AUTODEV_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let bind_addr = resolve_bind_addr(std::env::var("AUTODEV_BIND_ADDR").ok().as_deref());
    let github_secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok();
    let mcp_bearer_token = std::env::var("AUTODEV_MCP_BEARER_TOKEN").ok();

    let mut state = AppState::new(github_secret);
    if let Some(token) = mcp_bearer_token {
        state = state.with_mcp_bearer_token(token);
    }

    let listener = TcpListener::bind((bind_addr.as_str(), port)).await?;
    axum::serve(listener, router(state)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bind_address_defaults_to_localhost_and_allows_explicit_override() {
        assert_eq!(resolve_bind_addr(None), "127.0.0.1");
        assert_eq!(resolve_bind_addr(Some("")), "127.0.0.1");
        assert_eq!(resolve_bind_addr(Some(" 0.0.0.0 ")), "0.0.0.0");
    }
}
