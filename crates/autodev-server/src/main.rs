use std::{path::PathBuf, sync::Arc, time::Duration};

use autodev_server::{
    default_state_dir, router, run_objective_loop, validate_control_plane_paths, AppState,
    FileObjectiveStore, ModelActionProposer, ObjectiveRunner, RunnerExecution,
};
use forge_core::{
    default_fabric, default_profiles, AgentRole, OllamaProvider, VerificationFabric, Workspace,
};
use tokio::net::TcpListener;

const DEFAULT_MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("AUTODEV_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").ok();
    let mcp_bearer_token = std::env::var("AUTODEV_MCP_BEARER_TOKEN").ok();
    let model_base_url = std::env::var("AUTODEV_MODEL_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let workspace_root = std::env::var("AUTODEV_WORKSPACE").unwrap_or_else(|_| ".".to_string());
    let max_file_bytes = std::env::var("AUTODEV_MAX_FILE_BYTES")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_FILE_BYTES);

    let workspace = Workspace::new(workspace_root, max_file_bytes)?;
    let requested_state_dir = std::env::var("AUTODEV_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_state_dir(&workspace));
    let state_dir = validate_control_plane_paths(&workspace, requested_state_dir)?;
    let store = Arc::new(FileObjectiveStore::open(state_dir)?);
    let mut state = AppState::with_store(secret, store.clone());
    if let Some(token) = mcp_bearer_token {
        state = state.with_mcp_bearer_token(token);
    }

    let developer = default_profiles()
        .into_iter()
        .find(|profile| profile.role == AgentRole::Developer)
        .ok_or("developer agent profile is not configured")?;
    let provider = Arc::new(OllamaProvider::new(model_base_url));
    let proposer = Arc::new(ModelActionProposer::new(
        "control-plane-developer",
        developer,
        provider,
    ));
    let verification = Arc::new(|| -> VerificationFabric { default_fabric() });
    let runner = ObjectiveRunner::new(store, proposer, state.event_sender()).with_execution(
        RunnerExecution::new(workspace, AgentRole::Developer, verification),
    );

    std::thread::Builder::new()
        .name("autodev-objective-worker".to_string())
        .spawn(move || run_objective_loop(runner, Duration::from_millis(250)))?;

    let listener = TcpListener::bind(("0.0.0.0", port)).await?;
    axum::serve(listener, router(state)).await?;
    Ok(())
}
