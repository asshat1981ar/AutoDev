//! Deterministic ForgeLoop dispatch planning.
//!
//! This module binds a [`crate::skill::SkillRoute`] to registered logical
//! agents. Dispatch remains model-free and side-effect free: it only selects an
//! eligible [`crate::agent::AgentProfile`] and carries that profile's model
//! requirement forward as planning evidence.

use serde::{Deserialize, Serialize};

use crate::{
    AgentProfile, AgentRegistry, AgentRole, Capability, ModelRequirement, SkillRegistry, SkillRoute,
};

/// One skill assigned to one logical agent profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillAssignment {
    pub skill_id: String,
    pub agent_role: AgentRole,
    pub model: ModelRequirement,
    pub reasons: Vec<String>,
}

/// A selected skill that could not be dispatched safely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnassignedSkill {
    pub skill_id: String,
    pub reason: String,
}

/// Complete deterministic dispatch result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchPlan {
    pub assignments: Vec<SkillAssignment>,
    pub unassigned: Vec<UnassignedSkill>,
}

/// Bind routed skills to eligible agent profiles.
///
/// An agent is eligible only when:
/// - its role appears in the skill's compatible roles;
/// - its policy risk ceiling covers the skill's normal risk;
/// - it owns every capability required by the skill.
///
/// When multiple profiles qualify, the planner prefers the profile with the
/// fewest capabilities beyond those required by the skill (least privilege),
/// then uses the role wire-name for deterministic tie breaking.
pub fn plan_dispatch(
    route: &SkillRoute,
    skills: &SkillRegistry,
    agents: &AgentRegistry,
) -> DispatchPlan {
    let mut assignments = Vec::new();
    let mut unassigned = Vec::new();

    for routed in &route.selected {
        let Some(skill) = skills.get(&routed.skill_id) else {
            unassigned.push(UnassignedSkill {
                skill_id: routed.skill_id.clone(),
                reason: "selected skill is not present in registry".to_string(),
            });
            continue;
        };

        let mut eligible: Vec<&AgentProfile> = agents
            .profiles()
            .iter()
            .filter(|profile| skill.compatible_roles.contains(&profile.role))
            .filter(|profile| risk_rank(profile.policy.risk_ceiling) >= risk_rank(skill.risk))
            .filter(|profile| has_all_capabilities(profile, &skill.required_capabilities))
            .collect();

        eligible.sort_by(|a, b| {
            excess_capabilities(a, &skill.required_capabilities)
                .cmp(&excess_capabilities(b, &skill.required_capabilities))
                .then_with(|| a.role.as_str().cmp(b.role.as_str()))
        });

        match eligible.first() {
            Some(profile) => assignments.push(SkillAssignment {
                skill_id: skill.id.clone(),
                agent_role: profile.role,
                model: profile.model.clone(),
                reasons: vec![
                    format!("compatible_role:{}", profile.role.as_str()),
                    "capabilities:satisfied".to_string(),
                    format!("risk_ceiling:{}", profile.policy.risk_ceiling.as_str()),
                    "selection:least_privilege".to_string(),
                ],
            }),
            None => unassigned.push(UnassignedSkill {
                skill_id: skill.id.clone(),
                reason: "no registered agent satisfies role, capability, and risk requirements"
                    .to_string(),
            }),
        }
    }

    DispatchPlan {
        assignments,
        unassigned,
    }
}

fn has_all_capabilities(profile: &AgentProfile, required: &[Capability]) -> bool {
    required
        .iter()
        .all(|cap| profile.capabilities.iter().any(|owned| owned == cap))
}

fn excess_capabilities(profile: &AgentProfile, required: &[Capability]) -> usize {
    profile
        .capabilities
        .iter()
        .filter(|owned| !required.iter().any(|cap| cap == *owned))
        .count()
}

fn risk_rank(risk: crate::RiskLevel) -> u8 {
    match risk {
        crate::RiskLevel::Low => 0,
        crate::RiskLevel::Medium => 1,
        crate::RiskLevel::High => 2,
        crate::RiskLevel::Critical => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{default_profiles, default_skills, route_skills, DevelopmentContract};

    fn registry() -> AgentRegistry {
        let mut agents = AgentRegistry::new();
        for profile in default_profiles() {
            agents.register(profile);
        }
        agents
    }

    #[test]
    fn dispatches_build_skill_to_developer() {
        let skills = default_skills();
        let route = route_skills(
            &skills,
            &DevelopmentContract::new("continue development and implement a feature"),
            1,
        );
        let plan = plan_dispatch(&route, &skills, &registry());
        assert_eq!(plan.assignments.len(), 1);
        assert_eq!(plan.assignments[0].skill_id, "build-vertical-slice");
        assert_eq!(plan.assignments[0].agent_role, AgentRole::Developer);
        assert!(plan.unassigned.is_empty());
    }

    #[test]
    fn preserves_model_requirement_as_dispatch_evidence() {
        let skills = default_skills();
        let route = route_skills(&skills, &DevelopmentContract::new("implement a feature"), 1);
        let plan = plan_dispatch(&route, &skills, &registry());
        assert_eq!(plan.assignments[0].model.preferred, "qwen2.5-coder");
        assert!(plan.assignments[0].model.min_context_tokens >= 8192);
    }

    #[test]
    fn reports_unassigned_when_capabilities_do_not_fit() {
        let skills = default_skills();
        let route = route_skills(&skills, &DevelopmentContract::new("implement a feature"), 1);
        let agents = AgentRegistry::new();
        let plan = plan_dispatch(&route, &skills, &agents);
        assert!(plan.assignments.is_empty());
        assert_eq!(plan.unassigned.len(), 1);
    }
}
