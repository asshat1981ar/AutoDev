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
    task_ids: Vec<String>,
    run_ids: Vec<String>,
    envelope_ids: Vec<String>,
}

impl PlanReferences {
    pub fn task_ids(&self) -> &[String] {
        &self.task_ids
    }

    pub fn run_ids(&self) -> &[String] {
        &self.run_ids
    }

    pub fn envelope_ids(&self) -> &[String] {
        &self.envelope_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanBudget {
    max_replans: u32,
    max_attempts_per_milestone: u32,
    replans_used: u32,
}

impl PlanBudget {
    pub fn new(max_replans: u32, max_attempts_per_milestone: u32) -> Self {
        Self {
            max_replans,
            max_attempts_per_milestone,
            replans_used: 0,
        }
    }

    pub fn max_replans(&self) -> u32 {
        self.max_replans
    }

    pub fn max_attempts_per_milestone(&self) -> u32 {
        self.max_attempts_per_milestone
    }

    pub fn replans_used(&self) -> u32 {
        self.replans_used
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanMilestone {
    id: String,
    title: String,
    completed: bool,
    attempts: u32,
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDecision {
    summary: String,
    rationale: String,
    at: DateTime<Utc>,
}

impl PlanDecision {
    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn rationale(&self) -> &str {
        &self.rationale
    }

    pub fn at(&self) -> &DateTime<Utc> {
        &self.at
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanDiscovery {
    detail: String,
    at: DateTime<Utc>,
}

impl PlanDiscovery {
    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub fn at(&self) -> &DateTime<Utc> {
        &self.at
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedPlanCheckpoint")]
pub struct PlanCheckpoint {
    id: String,
    plan_id: String,
    status: ExecPlanStatus,
    references: PlanReferences,
    budget: PlanBudget,
    milestones: Vec<PlanMilestone>,
    decisions: Vec<PlanDecision>,
    discoveries: Vec<PlanDiscovery>,
    interruption_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct UncheckedPlanCheckpoint {
    id: String,
    plan_id: String,
    status: ExecPlanStatus,
    references: PlanReferences,
    budget: PlanBudget,
    milestones: Vec<PlanMilestone>,
    decisions: Vec<PlanDecision>,
    discoveries: Vec<PlanDiscovery>,
    interruption_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl PlanCheckpoint {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }

    pub fn status(&self) -> ExecPlanStatus {
        self.status
    }

    pub fn references(&self) -> &PlanReferences {
        &self.references
    }

    pub fn budget(&self) -> &PlanBudget {
        &self.budget
    }

    pub fn milestones(&self) -> &[PlanMilestone] {
        &self.milestones
    }

    pub fn decisions(&self) -> &[PlanDecision] {
        &self.decisions
    }

    pub fn discoveries(&self) -> &[PlanDiscovery] {
        &self.discoveries
    }

    pub fn interruption_reason(&self) -> Option<&str> {
        self.interruption_reason.as_deref()
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "UncheckedExecPlan")]
pub struct ExecPlan {
    id: String,
    goal: String,
    status: ExecPlanStatus,
    references: PlanReferences,
    budget: PlanBudget,
    milestones: Vec<PlanMilestone>,
    decisions: Vec<PlanDecision>,
    discoveries: Vec<PlanDiscovery>,
    interruption_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct UncheckedExecPlan {
    id: String,
    goal: String,
    status: ExecPlanStatus,
    references: PlanReferences,
    budget: PlanBudget,
    milestones: Vec<PlanMilestone>,
    decisions: Vec<PlanDecision>,
    discoveries: Vec<PlanDiscovery>,
    interruption_reason: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
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
    #[error("checkpoint id must be non-blank")]
    InvalidCheckpointId,
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn goal(&self) -> &str {
        &self.goal
    }

    pub fn status(&self) -> ExecPlanStatus {
        self.status
    }

    pub fn references(&self) -> &PlanReferences {
        &self.references
    }

    pub fn budget(&self) -> &PlanBudget {
        &self.budget
    }

    pub fn milestones(&self) -> &[PlanMilestone] {
        &self.milestones
    }

    pub fn decisions(&self) -> &[PlanDecision] {
        &self.decisions
    }

    pub fn discoveries(&self) -> &[PlanDiscovery] {
        &self.discoveries
    }

    pub fn interruption_reason(&self) -> Option<&str> {
        self.interruption_reason.as_deref()
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn add_milestone(&mut self, milestone: PlanMilestone) -> Result<(), ExecPlanError> {
        if self.status != ExecPlanStatus::Planned {
            return Err(ExecPlanError::InvalidTransition);
        }
        if milestone.id.trim().is_empty()
            || milestone.completed
            || milestone.attempts != 0
            || self
                .milestones
                .iter()
                .any(|existing| existing.id == milestone.id)
        {
            return Err(ExecPlanError::InvalidMilestones);
        }
        self.milestones.push(milestone);
        self.touch();
        Ok(())
    }

    pub fn complete_milestone(&mut self, milestone_id: &str) -> Result<(), ExecPlanError> {
        if !matches!(
            self.status,
            ExecPlanStatus::Running | ExecPlanStatus::Blocked
        ) {
            return Err(ExecPlanError::InvalidTransition);
        }
        let milestone = self
            .milestones
            .iter_mut()
            .find(|milestone| milestone.id == milestone_id)
            .ok_or(ExecPlanError::UnknownMilestone)?;
        milestone.completed = true;
        self.touch();
        Ok(())
    }

    pub fn add_task_reference(&mut self, task_id: impl Into<String>) {
        self.references.task_ids.push(task_id.into());
        self.touch();
    }

    pub fn add_run_reference(&mut self, run_id: impl Into<String>) {
        self.references.run_ids.push(run_id.into());
        self.touch();
    }

    pub fn add_envelope_reference(&mut self, envelope_id: impl Into<String>) {
        self.references.envelope_ids.push(envelope_id.into());
        self.touch();
    }

    pub fn validate(&self) -> Result<(), ExecPlanError> {
        if self.id.trim().is_empty() || self.goal.trim().is_empty() {
            return Err(ExecPlanError::EmptyIdentityOrGoal);
        }
        validate_plan_state(self.status, &self.budget, &self.milestones)
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
            ExecPlanStatus::Planned | ExecPlanStatus::Running
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
        if self.status != ExecPlanStatus::Running {
            return Err(ExecPlanError::InvalidTransition);
        }
        let milestone = self
            .milestones
            .iter_mut()
            .find(|milestone| milestone.id == milestone_id)
            .ok_or(ExecPlanError::UnknownMilestone)?;
        if milestone.completed {
            return Err(ExecPlanError::InvalidTransition);
        }
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
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ExecPlanError::InvalidCheckpointId);
        }
        Ok(PlanCheckpoint {
            id,
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
        if matches!(
            self.status,
            ExecPlanStatus::Completed | ExecPlanStatus::Cancelled | ExecPlanStatus::Failed
        ) {
            return Err(ExecPlanError::InvalidTransition);
        }
        if self.status == ExecPlanStatus::Interrupted {
            return Err(ExecPlanError::ReconciliationRequired);
        }
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

fn validate_plan_state(
    status: ExecPlanStatus,
    budget: &PlanBudget,
    milestones: &[PlanMilestone],
) -> Result<(), ExecPlanError> {
    if budget.max_replans == 0
        || budget.max_attempts_per_milestone == 0
        || budget.replans_used > budget.max_replans
    {
        return Err(ExecPlanError::InvalidBudget);
    }

    let mut milestone_ids = HashSet::with_capacity(milestones.len());
    for milestone in milestones {
        if milestone.id.trim().is_empty() || !milestone_ids.insert(milestone.id.as_str()) {
            return Err(ExecPlanError::InvalidMilestones);
        }
        if milestone.attempts > budget.max_attempts_per_milestone {
            return Err(ExecPlanError::MilestoneAttemptBudgetExhausted);
        }
    }

    if status == ExecPlanStatus::Completed
        && milestones.iter().any(|milestone| !milestone.completed)
    {
        return Err(ExecPlanError::IncompleteMilestones);
    }
    Ok(())
}

impl TryFrom<UncheckedPlanCheckpoint> for PlanCheckpoint {
    type Error = ExecPlanError;

    fn try_from(unchecked: UncheckedPlanCheckpoint) -> Result<Self, Self::Error> {
        if unchecked.id.trim().is_empty() {
            return Err(ExecPlanError::InvalidCheckpointId);
        }
        if unchecked.plan_id.trim().is_empty() {
            return Err(ExecPlanError::EmptyIdentityOrGoal);
        }
        validate_plan_state(unchecked.status, &unchecked.budget, &unchecked.milestones)?;
        Ok(Self {
            id: unchecked.id,
            plan_id: unchecked.plan_id,
            status: unchecked.status,
            references: unchecked.references,
            budget: unchecked.budget,
            milestones: unchecked.milestones,
            decisions: unchecked.decisions,
            discoveries: unchecked.discoveries,
            interruption_reason: unchecked.interruption_reason,
            created_at: unchecked.created_at,
            updated_at: unchecked.updated_at,
        })
    }
}

impl TryFrom<UncheckedExecPlan> for ExecPlan {
    type Error = ExecPlanError;

    fn try_from(unchecked: UncheckedExecPlan) -> Result<Self, Self::Error> {
        let plan = Self {
            id: unchecked.id,
            goal: unchecked.goal,
            status: unchecked.status,
            references: unchecked.references,
            budget: unchecked.budget,
            milestones: unchecked.milestones,
            decisions: unchecked.decisions,
            discoveries: unchecked.discoveries,
            interruption_reason: unchecked.interruption_reason,
            created_at: unchecked.created_at,
            updated_at: unchecked.updated_at,
        };
        plan.validate()?;
        Ok(plan)
    }
}
