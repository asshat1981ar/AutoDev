use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecPlanStatus {
    Planned,
    Running,
    Interrupted,
    Blocked,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlanReferences {
    pub task_ids: Vec<String>,
    pub run_ids: Vec<String>,
    pub envelope_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBudget {
    pub max_replans: u32,
    pub max_attempts_per_milestone: u32,
    pub replans_used: u32,
}

impl PlanBudget {
    pub fn new(max_replans: u32, max_attempts_per_milestone: u32) -> Self {
        Self {
            max_replans,
            max_attempts_per_milestone,
            replans_used: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanMilestone {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub attempts: u32,
}

impl PlanMilestone {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            completed: false,
            attempts: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDecision {
    pub summary: String,
    pub rationale: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDiscovery {
    pub detail: String,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanCheckpoint {
    pub id: String,
    pub plan_id: String,
    pub status: ExecPlanStatus,
    pub references: PlanReferences,
    pub budget: PlanBudget,
    pub milestones: Vec<PlanMilestone>,
    pub decisions: Vec<PlanDecision>,
    pub discoveries: Vec<PlanDiscovery>,
    pub interruption_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecPlan {
    pub id: String,
    pub goal: String,
    pub status: ExecPlanStatus,
    pub references: PlanReferences,
    pub budget: PlanBudget,
    pub milestones: Vec<PlanMilestone>,
    pub decisions: Vec<PlanDecision>,
    pub discoveries: Vec<PlanDiscovery>,
    pub interruption_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ExecPlanError {
    #[error("plan id and goal must be non-blank")]
    EmptyIdentityOrGoal,
    #[error("plan budgets must be greater than zero and replans used cannot exceed the maximum")]
    InvalidBudget,
    #[error("replan budget exhausted")]
    ReplanBudgetExhausted,
    #[error("milestone ids must be non-blank and unique")]
    InvalidMilestones,
    #[error("all milestones must be complete before the plan can complete")]
    IncompleteMilestones,
    #[error("invalid exec plan lifecycle transition")]
    InvalidTransition,
    #[error("interrupted effectful work must be reconciled before resume")]
    ReconciliationRequired,
    #[error("milestone attempt budget exhausted")]
    MilestoneAttemptBudgetExhausted,
    #[error("unknown milestone")]
    UnknownMilestone,
}

impl ExecPlan {
    pub fn new(id: impl Into<String>, goal: impl Into<String>, budget: PlanBudget) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            goal: goal.into(),
            status: ExecPlanStatus::Planned,
            references: PlanReferences::default(),
            budget,
            milestones: Vec::new(),
            decisions: Vec::new(),
            discoveries: Vec::new(),
            interruption_reason: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn validate(&self) -> Result<(), ExecPlanError> {
        if self.id.trim().is_empty() || self.goal.trim().is_empty() {
            return Err(ExecPlanError::EmptyIdentityOrGoal);
        }
        if self.budget.max_replans == 0
            || self.budget.max_attempts_per_milestone == 0
            || self.budget.replans_used > self.budget.max_replans
        {
            return Err(ExecPlanError::InvalidBudget);
        }

        let mut milestone_ids = HashSet::with_capacity(self.milestones.len());
        for milestone in &self.milestones {
            if milestone.id.trim().is_empty() || !milestone_ids.insert(milestone.id.as_str()) {
                return Err(ExecPlanError::InvalidMilestones);
            }
            if milestone.attempts > self.budget.max_attempts_per_milestone {
                return Err(ExecPlanError::MilestoneAttemptBudgetExhausted);
            }
        }

        if self.status == ExecPlanStatus::Completed
            && self.milestones.iter().any(|milestone| !milestone.completed)
        {
            return Err(ExecPlanError::IncompleteMilestones);
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), ExecPlanError> {
        if self.status != ExecPlanStatus::Planned {
            return Err(ExecPlanError::InvalidTransition);
        }
        self.status = ExecPlanStatus::Running;
        self.interruption_reason = None;
        self.touch();
        Ok(())
    }

    pub fn interrupt(&mut self, reason: impl Into<String>) -> Result<(), ExecPlanError> {
        if self.status != ExecPlanStatus::Running {
            return Err(ExecPlanError::InvalidTransition);
        }
        self.status = ExecPlanStatus::Interrupted;
        self.interruption_reason = Some(reason.into());
        self.touch();
        Ok(())
    }

    pub fn block(&mut self, reason: impl Into<String>) -> Result<(), ExecPlanError> {
        if !matches!(
            self.status,
            ExecPlanStatus::Planned | ExecPlanStatus::Running | ExecPlanStatus::Interrupted
        ) {
            return Err(ExecPlanError::InvalidTransition);
        }
        self.status = ExecPlanStatus::Blocked;
        self.interruption_reason = Some(reason.into());
        self.touch();
        Ok(())
    }

    pub fn resume(&mut self, reconciled: bool) -> Result<(), ExecPlanError> {
        match self.status {
            ExecPlanStatus::Interrupted if !reconciled => {
                Err(ExecPlanError::ReconciliationRequired)
            }
            ExecPlanStatus::Interrupted | ExecPlanStatus::Blocked => {
                self.status = ExecPlanStatus::Running;
                self.interruption_reason = None;
                self.touch();
                Ok(())
            }
            _ => Err(ExecPlanError::InvalidTransition),
        }
    }

    pub fn cancel(&mut self) -> Result<(), ExecPlanError> {
        if matches!(
            self.status,
            ExecPlanStatus::Completed | ExecPlanStatus::Cancelled
        ) {
            return Err(ExecPlanError::InvalidTransition);
        }
        self.status = ExecPlanStatus::Cancelled;
        self.touch();
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), ExecPlanError> {
        if !matches!(
            self.status,
            ExecPlanStatus::Running | ExecPlanStatus::Blocked
        ) {
            return Err(ExecPlanError::InvalidTransition);
        }
        if self.milestones.iter().any(|milestone| !milestone.completed) {
            return Err(ExecPlanError::IncompleteMilestones);
        }
        self.status = ExecPlanStatus::Completed;
        self.interruption_reason = None;
        self.touch();
        Ok(())
    }

    pub fn start_milestone_attempt(&mut self, milestone_id: &str) -> Result<u32, ExecPlanError> {
        let milestone = self
            .milestones
            .iter_mut()
            .find(|milestone| milestone.id == milestone_id)
            .ok_or(ExecPlanError::UnknownMilestone)?;
        if milestone.attempts >= self.budget.max_attempts_per_milestone {
            return Err(ExecPlanError::MilestoneAttemptBudgetExhausted);
        }
        milestone.attempts += 1;
        let attempts = milestone.attempts;
        self.touch();
        Ok(attempts)
    }

    pub fn checkpoint(&self, id: impl Into<String>) -> Result<PlanCheckpoint, ExecPlanError> {
        self.validate()?;
        Ok(PlanCheckpoint {
            id: id.into(),
            plan_id: self.id.clone(),
            status: self.status,
            references: self.references.clone(),
            budget: self.budget.clone(),
            milestones: self.milestones.clone(),
            decisions: self.decisions.clone(),
            discoveries: self.discoveries.clone(),
            interruption_reason: self.interruption_reason.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }

    pub fn record_decision(&mut self, summary: impl Into<String>, rationale: impl Into<String>) {
        self.decisions.push(PlanDecision {
            summary: summary.into(),
            rationale: rationale.into(),
            at: Utc::now(),
        });
        self.touch();
    }

    pub fn record_discovery(&mut self, detail: impl Into<String>) {
        self.discoveries.push(PlanDiscovery {
            detail: detail.into(),
            at: Utc::now(),
        });
        self.touch();
    }

    pub fn consume_replan(&mut self, reason: impl Into<String>) -> Result<(), ExecPlanError> {
        if self.budget.replans_used >= self.budget.max_replans {
            return Err(ExecPlanError::ReplanBudgetExhausted);
        }
        self.budget.replans_used += 1;
        self.record_decision("Replan", reason);
        if !matches!(
            self.status,
            ExecPlanStatus::Completed | ExecPlanStatus::Cancelled
        ) {
            self.status = ExecPlanStatus::Planned;
            self.interruption_reason = None;
            self.touch();
        }
        Ok(())
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}
