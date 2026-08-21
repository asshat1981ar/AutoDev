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
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_allowed_hosts(hosts);
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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::ObjectiveRequest;

    use super::*;

    async fn state_with_plan() -> (AppState, String) {
        let state = AppState::new(None);
        let record = state
            .enqueue(ObjectiveRequest {
                repository: "owner/repo".to_string(),
                description: "Develop through Vibe".to_string(),
                branch: None,
            })
            .await
            .expect("objective");
        (state, record.id)
    }

    #[tokio::test]
    async fn project_status_reports_counts_and_no_authority() {
        let (state, _) = state_with_plan().await;
        let mcp = AutoDevMcp::new(state);

        let status: Value = serde_json::from_str(&mcp.project_status().await).expect("status JSON");
        assert_eq!(status["objective_count"], 1);
        assert_eq!(status["exec_plan_count"], 1);
        assert_eq!(status["authority"], "none");
        assert_eq!(status["trusted_execution_boundary"], "forge_core");
    }

    #[tokio::test]
    async fn execplan_get_returns_typed_plan_projection() {
        let (state, plan_id) = state_with_plan().await;
        let mcp = AutoDevMcp::new(state);

        let result = mcp
            .execplan_get(Parameters(PlanLookupInput {
                plan_id: plan_id.clone(),
            }))
            .await
            .expect("plan projection");
        let plan: Value = serde_json::from_str(&result).expect("plan JSON");

        assert_eq!(plan["id"], plan_id);
        assert_eq!(plan["budget"]["replans_used"], 0);
    }

    #[tokio::test]
    async fn verification_status_never_self_verifies() {
        let (state, plan_id) = state_with_plan().await;
        let mcp = AutoDevMcp::new(state);

        let result = mcp
            .verification_status(Parameters(VerificationStatusInput {
                plan_id: Some(plan_id),
            }))
            .await
            .expect("verification projection");
        let status: Value = serde_json::from_str(&result).expect("verification JSON");

        assert_eq!(status["verified"], false);
        assert_eq!(status["authority"], "none");
        assert_eq!(status["verification_boundary"], "verification_fabric");
    }

    #[tokio::test]
    async fn test_proposal_is_non_executing_intent() {
        let (state, _) = state_with_plan().await;
        let mcp = AutoDevMcp::new(state);

        let result = mcp
            .test_propose(Parameters(TestProposalInput {
                task_id: "task-1".to_string(),
                agent_id: "vibe".to_string(),
                reason: "verify server".to_string(),
                command: "cargo test -p autodev-server".to_string(),
            }))
            .await
            .expect("test proposal");
        let proposal: Value = serde_json::from_str(&result).expect("proposal JSON");

        assert_eq!(proposal["status"], "proposal_only");
        assert_eq!(proposal["proposal_type"], "test_run");
        assert_eq!(proposal["execution_authorized"], false);
        assert_eq!(proposal["command"], "cargo test -p autodev-server");
    }

    #[tokio::test]
    async fn replan_proposal_does_not_consume_budget_or_mutate_plan() {
        let (state, plan_id) = state_with_plan().await;
        let before = state.exec_plan(&plan_id).await.expect("plan before");
        let mcp = AutoDevMcp::new(state.clone());

        let result = mcp
            .replan_propose(Parameters(ReplanProposalInput {
                plan_id: plan_id.clone(),
                agent_id: "vibe".to_string(),
                reason: "new evidence".to_string(),
                proposed_goal: "Refine Vibe integration".to_string(),
            }))
            .await
            .expect("replan proposal");
        let proposal: Value = serde_json::from_str(&result).expect("proposal JSON");
        let after = state.exec_plan(&plan_id).await.expect("plan after");

        assert_eq!(proposal["status"], "proposal_only");
        assert_eq!(proposal["execution_authorized"], false);
        assert_eq!(before, after);
        assert_eq!(after.budget().replans_used(), 0);
    }
}
