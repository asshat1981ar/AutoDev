//! Deterministic model assignment for ForgeLoop dispatch plans.
//!
//! This layer resolves an agent's provider-neutral [`crate::ModelRequirement`]
//! against observed provider/model availability. It remains side-effect free:
//! discovery happens elsewhere, and this module consumes normalized snapshots.

use serde::{Deserialize, Serialize};

use crate::{
    route, AgentRole, DispatchPlan, Model, ModelCapabilities, ModelHealth, RoutingPolicy,
};

/// A normalized model observed from a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvailableModel {
    /// Stable provider instance name, e.g. `local-ollama`.
    pub provider: String,
    /// Provider family expected by agent model requirements, e.g. `ollama`.
    pub family: String,
    /// Whether this provider executes locally to the AutoDev runtime.
    pub local: bool,
    /// Current provider/model health.
    pub health: ModelHealth,
    /// Declared usable context capacity. Unknown capacity is intentionally not
    /// assumed to satisfy a minimum-context requirement.
    pub context_tokens: Option<u32>,
    /// Provider-neutral model metadata.
    pub model: Model,
}

/// One resolved model for one skill/agent assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelAssignment {
    pub skill_id: String,
    pub agent_role: AgentRole,
    pub provider: String,
    pub model_id: String,
    pub score: u32,
    pub reasons: Vec<String>,
}

/// A dispatch assignment that could not obtain a compatible model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedModel {
    pub skill_id: String,
    pub agent_role: AgentRole,
    pub reason: String,
}

/// Complete model-resolution result for a dispatch plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelPlan {
    pub assignments: Vec<ModelAssignment>,
    pub unresolved: Vec<UnresolvedModel>,
}

/// Resolve each dispatched skill to the best available model.
///
/// Hard eligibility gates are applied before scoring:
/// - provider family must match the agent requirement;
/// - unavailable providers/models are rejected;
/// - declared context capacity must satisfy the minimum requirement;
/// - requested model capabilities must be supported.
///
/// Eligible models then reuse the existing model-fabric routing score. The
/// preferred model, healthy status, and local execution receive deterministic
/// bonuses. Ties are broken by provider then model id.
pub fn resolve_models(
    dispatch: &DispatchPlan,
    available: &[AvailableModel],
    requested: &ModelCapabilities,
    policy: &RoutingPolicy,
) -> ModelPlan {
    let mut assignments = Vec::new();
    let mut unresolved = Vec::new();

    for assignment in &dispatch.assignments {
        let requirement = &assignment.model;
        let mut candidates: Vec<ModelAssignment> = available
            .iter()
            .filter(|candidate| candidate.family == requirement.family)
            .filter(|candidate| candidate.health != ModelHealth::Unavailable)
            .filter(|candidate| {
                candidate.context_tokens.unwrap_or(0) >= requirement.min_context_tokens
            })
            .filter_map(|candidate| {
                let base = route(std::slice::from_ref(&candidate.model), requested, policy)
                    .into_iter()
                    .next()?;

                let mut score = base.score;
                let mut reasons = vec![
                    format!("family:{}", candidate.family),
                    format!("context:{}", candidate.context_tokens.unwrap_or(0)),
                    "capabilities:satisfied".to_string(),
                ];

                if preferred_matches(&candidate.model, &requirement.preferred) {
                    score += 100;
                    reasons.push("preferred_model".to_string());
                }
                if candidate.health == ModelHealth::Healthy {
                    score += 20;
                    reasons.push("health:healthy".to_string());
                } else {
                    reasons.push("health:degraded".to_string());
                }
                if candidate.local {
                    score += 10;
                    reasons.push("local".to_string());
                }

                Some(ModelAssignment {
                    skill_id: assignment.skill_id.clone(),
                    agent_role: assignment.agent_role,
                    provider: candidate.provider.clone(),
                    model_id: candidate.model.id.clone(),
                    score,
                    reasons,
                })
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.provider.cmp(&b.provider))
                .then_with(|| a.model_id.cmp(&b.model_id))
        });

        if let Some(best) = candidates.into_iter().next() {
            assignments.push(best);
        } else {
            unresolved.push(UnresolvedModel {
                skill_id: assignment.skill_id.clone(),
                agent_role: assignment.agent_role,
                reason: format!(
                    "no available model satisfies family '{}', context >= {}, and requested capabilities",
                    requirement.family, requirement.min_context_tokens
                ),
            });
        }
    }

    ModelPlan {
        assignments,
        unresolved,
    }
}

fn preferred_matches(model: &Model, preferred: &str) -> bool {
    model.id == preferred
        || model.name == preferred
        || model
            .id
            .strip_prefix(preferred)
            .is_some_and(|suffix| suffix.starts_with(':'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        default_profiles, default_skills, plan_dispatch, route_skills, AgentRegistry,
        DevelopmentContract,
    };

    fn dispatch() -> DispatchPlan {
        let skills = default_skills();
        let route = route_skills(
            &skills,
            &DevelopmentContract::new("continue development and implement a feature"),
            1,
        );
        let mut agents = AgentRegistry::new();
        for profile in default_profiles() {
            agents.register(profile);
        }
        plan_dispatch(&route, &skills, &agents)
    }

    fn available(id: &str, context_tokens: u32, local: bool) -> AvailableModel {
        AvailableModel {
            provider: "local-ollama".to_string(),
            family: "ollama".to_string(),
            local,
            health: ModelHealth::Healthy,
            context_tokens: Some(context_tokens),
            model: Model {
                id: id.to_string(),
                name: id.to_string(),
                size: Some(4_000_000_000),
                capabilities: ModelCapabilities::default(),
            },
        }
    }

    #[test]
    fn resolves_preferred_model_when_it_meets_requirements() {
        let models = vec![
            available("other:latest", 16_384, true),
            available("qwen2.5-coder:latest", 16_384, true),
        ];
        let plan = resolve_models(
            &dispatch(),
            &models,
            &ModelCapabilities::default(),
            &RoutingPolicy::default(),
        );
        assert!(plan.unresolved.is_empty());
        assert_eq!(plan.assignments[0].model_id, "qwen2.5-coder:latest");
        assert!(plan.assignments[0]
            .reasons
            .iter()
            .any(|reason| reason == "preferred_model"));
    }

    #[test]
    fn rejects_unknown_or_insufficient_context_capacity() {
        let mut unknown = available("qwen2.5-coder:latest", 16_384, true);
        unknown.context_tokens = None;
        let models = vec![unknown, available("other:latest", 4_096, true)];
        let plan = resolve_models(
            &dispatch(),
            &models,
            &ModelCapabilities::default(),
            &RoutingPolicy::default(),
        );
        assert!(plan.assignments.is_empty());
        assert_eq!(plan.unresolved.len(), 1);
    }

    #[test]
    fn unavailable_models_are_not_selected() {
        let mut model = available("qwen2.5-coder:latest", 16_384, true);
        model.health = ModelHealth::Unavailable;
        let plan = resolve_models(
            &dispatch(),
            &[model],
            &ModelCapabilities::default(),
            &RoutingPolicy::default(),
        );
        assert!(plan.assignments.is_empty());
        assert_eq!(plan.unresolved.len(), 1);
    }
}
