use std::path::Path;

use forge_core::{ActionType, AgentAction, Capability, RiskLevel};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::AppState;

pub(crate) const MCP_MAX_BODY_BYTES: usize = 512 * 1024;
const DEFAULT_MCP_HOSTS: [&str; 4] = ["localhost", "127.0.0.1", "::1", "autodev-server"];
const DEFAULT_MCP_ORIGINS: [&str; 3] = [
    "http://localhost",
    "http://127.0.0.1",
    "http://localhost:8080",
];

#[derive(Clone)]
pub(crate) struct AutoDevMcp {
    state: AppState,
}

impl AutoDevMcp {
    fn new(state: AppState) -> Self {
        Self { state }
    }
}

#[derive(Debug, Deserialize, rmcp::schemars::JsonSchema)]
struct WriteProposalInput {
    /// Durable task identifier that the proposal belongs to.
    task_id: String,
    /// Logical agent identity that generated the proposal.
    agent_id: String,
    /// Human-readable reason for the proposed candidate write.
    reason: String,
    /// Workspace-relative target path. Absolute and traversal paths are rejected.
    path: String,
    /// Proposed UTF-8 file content. This tool never persists it directly.
    content: String,
}

#[tool_router]
impl AutoDevMcp {
    #[tool(
        name = "autodev.objectives.list",
        description = "List the current AutoDev objective projection. This is read-only and does not authorize execution.",
        annotations(
            title = "List AutoDev objectives",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn objectives_list(&self) -> String {
        let objectives = self.state.objectives.read().await;
        serde_json::to_string(&objectives.values().cloned().collect::<Vec<_>>())
            .expect("objective projection is JSON serializable")
    }

    #[tool(
        name = "autodev.gaps.scan",
        description = "Create a capability-gap scan request from current objective state. The result is untrusted intent only; it does not create or activate a skill, MCP server, capability, or policy change.",
        annotations(
            title = "Propose capability gap scan",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn gaps_scan(&self) -> String {
        let objectives = self.state.objectives.read().await;
        let objective_ids: Vec<String> = objectives.keys().cloned().collect();
        json!({
            "status": "proposal_only",
            "proposal_type": "capability_gap_scan",
            "objective_ids": objective_ids,
            "evidence_required": true,
            "authority": "none",
            "next_boundary": "forge_core_capability_gap_evaluation"
        })
        .to_string()
    }

    #[tool(
        name = "autodev.action.propose",
        description = "Create a typed write-file AgentAction proposal without executing it. ForgeCore authorization, policy, workspace confinement, and verification are still required before mutation.",
        annotations(
            title = "Propose AutoDev candidate write",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = false,
            open_world_hint = false
        )
    )]
    async fn action_propose(
        &self,
        Parameters(input): Parameters<WriteProposalInput>,
    ) -> Result<String, McpError> {
        validate_write_proposal(&input)?;

        let proposal = AgentAction {
            id: format!("mcp-proposal-{}", Uuid::new_v4()),
            task_id: input.task_id,
            agent_id: input.agent_id,
            action_type: ActionType::WriteFile,
            reason: input.reason,
            risk: RiskLevel::Medium,
            capabilities: vec![Capability::WriteFile],
            payload: json!({
                "operation": "write_file",
                "path": input.path,
                "content": input.content,
            }),
            expected: json!({
                "status": "candidate_only",
                "execution_authorized": false,
            }),
        };

        Ok(serde_json::to_string(&proposal).expect("AgentAction is JSON serializable"))
    }
}

#[tool_handler]
impl ServerHandler for AutoDevMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(
                Implementation::new("autodev", env!("CARGO_PKG_VERSION")).with_description(
                    "Stateless MCP adapter over AutoDev's proposal and observation boundaries",
                ),
            )
            .with_instructions(
                "Tools expose read projections and untrusted proposals only. ForgeCore remains the sole trusted execution authority.",
            )
    }
}

pub(crate) fn service(state: AppState) -> StreamableHttpService<AutoDevMcp, LocalSessionManager> {
    let hosts = configured_hosts();
    let origins = configured_origins();
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        // Defense in depth alongside the bearer middleware: an HTTP
        // request carrying a valid bearer but a foreign `Host` header
        // is rejected by the transport. The allowlist must remain
        // non-empty; do not introduce a `disable_allowed_hosts()` knob
        // without a security review.
        .with_allowed_hosts(hosts)
        // Bind the MCP transport to localhost origins only by default.
        // A stolen bearer token on a developer laptop cannot be used
        // cross-origin from a malicious co-located browser page.
        .with_allowed_origins(origins);
    StreamableHttpService::new(
        move || Ok(AutoDevMcp::new(state.clone())),
        LocalSessionManager::default().into(),
        config,
    )
}

fn configured_hosts() -> Vec<String> {
    let configured = std::env::var("AUTODEV_MCP_ALLOWED_HOSTS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|host| !host.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|hosts| !hosts.is_empty());

    configured.unwrap_or_else(|| {
        DEFAULT_MCP_HOSTS
            .iter()
            .map(|host| host.to_string())
            .collect()
    })
}

fn configured_origins() -> Vec<String> {
    let configured = std::env::var("AUTODEV_MCP_ALLOWED_ORIGINS")
        .ok()
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|origins| !origins.is_empty());

    configured.unwrap_or_else(|| {
        DEFAULT_MCP_ORIGINS
            .iter()
            .map(|origin| origin.to_string())
            .collect()
    })
}

fn validate_write_proposal(input: &WriteProposalInput) -> Result<(), McpError> {
    if input.task_id.trim().is_empty()
        || input.agent_id.trim().is_empty()
        || input.reason.trim().is_empty()
        || input.path.trim().is_empty()
    {
        return Err(McpError::invalid_params(
            "task_id, agent_id, reason, and path are required",
            None,
        ));
    }

    let path = Path::new(&input.path);
    if path.is_absolute()
        || input.path.contains('\\')
        || input.path.split('/').any(|segment| segment == "..")
    {
        return Err(McpError::invalid_params(
            "path must be workspace-relative and must not contain traversal",
            None,
        ));
    }

    Ok(())
}
