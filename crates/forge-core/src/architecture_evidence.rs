//! Architecture and research evidence for ConnectorForge Workshop 1.
//!
//! This module is deliberately separate from execution evidence. It represents
//! normalized findings and design decisions, but it never authorizes or executes
//! an agent action. Connector-specific payloads must be normalized before they
//! cross this boundary.

use std::collections::BTreeMap;
use std::fmt::Write;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evidence::sha256_hex;

/// Provenance class for a normalized architecture finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    RepoObserved,
    Documented,
    ResearchSupported,
    ExperimentallyVerified,
    Inferred,
    Hypothesis,
}

impl EvidenceClass {
    /// Whether this class is strong enough to support a verified decision.
    pub fn can_satisfy_verified_gate(self) -> bool {
        matches!(
            self,
            Self::RepoObserved
                | Self::Documented
                | Self::ResearchSupported
                | Self::ExperimentallyVerified
        )
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::RepoObserved => "repo_observed",
            Self::Documented => "documented",
            Self::ResearchSupported => "research_supported",
            Self::ExperimentallyVerified => "experimentally_verified",
            Self::Inferred => "inferred",
            Self::Hypothesis => "hypothesis",
        }
    }
}

/// A connector-neutral architecture finding with provenance and invalidation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    pub id: String,
    pub objective_id: String,
    pub claim: String,
    pub evidence_class: EvidenceClass,
    pub source_system: String,
    pub source_reference: String,
    pub observed_at: DateTime<Utc>,
    /// Confidence expressed as an integer percentage from 0 through 100.
    pub confidence: u8,
    /// SHA-256 of the normalized source content supplied at construction time.
    pub content_fingerprint: String,
    pub invalidation_condition: String,
}

impl EvidenceRecord {
    /// Construct a validated normalized evidence record.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        objective_id: impl Into<String>,
        claim: impl Into<String>,
        evidence_class: EvidenceClass,
        source_system: impl Into<String>,
        source_reference: impl Into<String>,
        observed_at: DateTime<Utc>,
        confidence: u8,
        normalized_content: &str,
        invalidation_condition: impl Into<String>,
    ) -> Result<Self, ArchitectureEvidenceError> {
        let id = required(id.into(), "id")?;
        let objective_id = required(objective_id.into(), "objective_id")?;
        let claim = required(claim.into(), "claim")?;
        let source_system = required(source_system.into(), "source_system")?;
        let source_reference = required(source_reference.into(), "source_reference")?;
        let invalidation_condition =
            required(invalidation_condition.into(), "invalidation_condition")?;
        if normalized_content.trim().is_empty() {
            return Err(ArchitectureEvidenceError::EmptyField("normalized_content"));
        }
        if confidence > 100 {
            return Err(ArchitectureEvidenceError::InvalidConfidence(confidence));
        }

        Ok(Self {
            id,
            objective_id,
            claim,
            evidence_class,
            source_system,
            source_reference,
            observed_at,
            confidence,
            content_fingerprint: sha256_hex(normalized_content.as_bytes()),
            invalidation_condition,
        })
    }

    /// Whether this record can independently contribute to a verified gate.
    pub fn can_satisfy_verified_gate(&self) -> bool {
        self.evidence_class.can_satisfy_verified_gate()
    }
}

/// One considered architecture alternative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureAlternative {
    pub name: String,
    pub rationale: String,
    pub rejected: bool,
}

/// How difficult a decision is to reverse after adoption.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reversibility {
    Easy,
    Moderate,
    Difficult,
    Irreversible,
}

impl Reversibility {
    fn as_str(self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Moderate => "moderate",
            Self::Difficult => "difficult",
            Self::Irreversible => "irreversible",
        }
    }
}

/// Maturity of an architecture decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionMaturity {
    Experimental,
    Verified,
}

impl DecisionMaturity {
    fn as_str(self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::Verified => "verified",
        }
    }
}

/// An evidence-linked architecture decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDecision {
    pub id: String,
    pub objective_id: String,
    pub decision: String,
    pub alternatives: Vec<ArchitectureAlternative>,
    pub contradiction: String,
    pub selected_option: String,
    pub rationale: String,
    pub evidence_refs: Vec<String>,
    pub reversibility: Reversibility,
    pub risks: Vec<String>,
    pub invalidation_conditions: Vec<String>,
    pub maturity: DecisionMaturity,
}

impl ArchitectureDecision {
    /// Validate references and evidence-gate semantics against normalized evidence.
    pub fn validate(
        &self,
        evidence: &BTreeMap<String, EvidenceRecord>,
    ) -> Result<(), ArchitectureEvidenceError> {
        required_ref(&self.id, "id")?;
        required_ref(&self.objective_id, "objective_id")?;
        required_ref(&self.decision, "decision")?;
        required_ref(&self.contradiction, "contradiction")?;
        required_ref(&self.selected_option, "selected_option")?;
        required_ref(&self.rationale, "rationale")?;

        if !self
            .alternatives
            .iter()
            .any(|alternative| alternative.rejected)
        {
            return Err(ArchitectureEvidenceError::MissingRejectedAlternative(
                self.id.clone(),
            ));
        }
        if self.invalidation_conditions.is_empty()
            || self
                .invalidation_conditions
                .iter()
                .all(|condition| condition.trim().is_empty())
        {
            return Err(ArchitectureEvidenceError::MissingInvalidationCondition(
                self.id.clone(),
            ));
        }

        let mut has_supported_evidence = false;
        for evidence_id in &self.evidence_refs {
            let record = evidence.get(evidence_id).ok_or_else(|| {
                ArchitectureEvidenceError::UnknownEvidenceReference(
                    self.id.clone(),
                    evidence_id.clone(),
                )
            })?;
            has_supported_evidence |= record.can_satisfy_verified_gate();
        }

        if self.maturity == DecisionMaturity::Verified && !has_supported_evidence {
            return Err(ArchitectureEvidenceError::UnsupportedVerifiedDecision(
                self.id.clone(),
            ));
        }

        Ok(())
    }
}

/// Failure modes for normalized architecture evidence and reports.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArchitectureEvidenceError {
    #[error("field `{0}` must not be empty")]
    EmptyField(&'static str),
    #[error("confidence must be between 0 and 100, got {0}")]
    InvalidConfidence(u8),
    #[error("duplicate evidence id `{0}`")]
    DuplicateEvidenceId(String),
    #[error("{item_kind} `{item_id}` belongs to objective `{actual}`; expected `{expected}`")]
    ObjectiveMismatch {
        item_kind: &'static str,
        item_id: String,
        expected: String,
        actual: String,
    },
    #[error("decision `{0}` must include at least one rejected alternative")]
    MissingRejectedAlternative(String),
    #[error("decision `{0}` references unknown evidence `{1}`")]
    UnknownEvidenceReference(String, String),
    #[error("verified decision `{0}` has no gate-satisfying evidence")]
    UnsupportedVerifiedDecision(String),
    #[error("decision `{0}` must include at least one invalidation condition")]
    MissingInvalidationCondition(String),
}

/// Criteria used to compare architecture options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureCriterion {
    Impact,
    EvidenceStrength,
    Reversibility,
    ImplementationCost,
    OperationalComplexity,
    SecurityRisk,
    ContextBurden,
    Reuse,
    KnowledgeGain,
    FailureIsolation,
}

impl ArchitectureCriterion {
    fn as_str(self) -> &'static str {
        match self {
            Self::Impact => "impact",
            Self::EvidenceStrength => "evidence_strength",
            Self::Reversibility => "reversibility",
            Self::ImplementationCost => "implementation_cost",
            Self::OperationalComplexity => "operational_complexity",
            Self::SecurityRisk => "security_risk",
            Self::ContextBurden => "context_burden",
            Self::Reuse => "reuse",
            Self::KnowledgeGain => "knowledge_gain",
            Self::FailureIsolation => "failure_isolation",
        }
    }
}

/// Weighted score for one architecture criterion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriterionScore {
    pub criterion: ArchitectureCriterion,
    pub weight: i32,
    pub score: i32,
}

/// One candidate architecture with deterministic weighted scoring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureOption {
    pub name: String,
    pub description: String,
    pub scores: Vec<CriterionScore>,
}

impl ArchitectureOption {
    pub fn total_score(&self) -> i32 {
        self.scores
            .iter()
            .map(|entry| entry.weight * entry.score)
            .sum()
    }
}

/// Rank highest score first, with name as a stable tie-breaker.
pub fn rank_options(options: &[ArchitectureOption]) -> Vec<ArchitectureOption> {
    let mut ranked = options.to_vec();
    ranked.sort_by(|left, right| {
        right
            .total_score()
            .cmp(&left.total_score())
            .then_with(|| left.name.cmp(&right.name))
    });
    ranked
}

/// Complete deterministic input for a W1 Markdown report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureReportInput {
    pub objective_id: String,
    pub title: String,
    pub desired_outcome: String,
    pub evidence: Vec<EvidenceRecord>,
    pub decisions: Vec<ArchitectureDecision>,
    pub options: Vec<ArchitectureOption>,
}

/// Validate and render an evidence-linked architecture report.
pub fn render_architecture_report(
    input: &ArchitectureReportInput,
) -> Result<String, ArchitectureEvidenceError> {
    required_ref(&input.objective_id, "objective_id")?;
    required_ref(&input.title, "title")?;
    required_ref(&input.desired_outcome, "desired_outcome")?;

    let mut evidence_by_id = BTreeMap::new();
    for record in &input.evidence {
        if record.objective_id != input.objective_id {
            return Err(ArchitectureEvidenceError::ObjectiveMismatch {
                item_kind: "evidence",
                item_id: record.id.clone(),
                expected: input.objective_id.clone(),
                actual: record.objective_id.clone(),
            });
        }
        if evidence_by_id
            .insert(record.id.clone(), record.clone())
            .is_some()
        {
            return Err(ArchitectureEvidenceError::DuplicateEvidenceId(
                record.id.clone(),
            ));
        }
    }
    for decision in &input.decisions {
        if decision.objective_id != input.objective_id {
            return Err(ArchitectureEvidenceError::ObjectiveMismatch {
                item_kind: "decision",
                item_id: decision.id.clone(),
                expected: input.objective_id.clone(),
                actual: decision.objective_id.clone(),
            });
        }
        decision.validate(&evidence_by_id)?;
    }

    let mut evidence = input.evidence.clone();
    evidence.sort_by(|left, right| {
        left.source_system
            .cmp(&right.source_system)
            .then_with(|| left.id.cmp(&right.id))
    });
    let mut decisions = input.decisions.clone();
    decisions.sort_by(|left, right| left.id.cmp(&right.id));
    let options = rank_options(&input.options);

    let mut report = String::new();
    writeln!(report, "# Architecture Evidence Report: {}", input.title)
        .expect("writing to String cannot fail");
    writeln!(report).expect("writing to String cannot fail");
    writeln!(report, "**Objective:** `{}`", input.objective_id)
        .expect("writing to String cannot fail");
    writeln!(report, "**Desired outcome:** {}", input.desired_outcome)
        .expect("writing to String cannot fail");

    writeln!(report, "\n## Evidence").expect("writing to String cannot fail");
    for record in evidence {
        writeln!(report, "\n### {} — {}", record.id, record.claim)
            .expect("writing to String cannot fail");
        writeln!(report, "- Class: `{}`", record.evidence_class.as_str())
            .expect("writing to String cannot fail");
        writeln!(report, "- Source system: `{}`", record.source_system)
            .expect("writing to String cannot fail");
        writeln!(report, "- Source reference: `{}`", record.source_reference)
            .expect("writing to String cannot fail");
        writeln!(
            report,
            "- Observed at: `{}`",
            record.observed_at.to_rfc3339()
        )
        .expect("writing to String cannot fail");
        writeln!(report, "- Confidence: {}", record.confidence)
            .expect("writing to String cannot fail");
        writeln!(report, "- Fingerprint: `{}`", record.content_fingerprint)
            .expect("writing to String cannot fail");
        writeln!(
            report,
            "- Invalidation condition: {}",
            record.invalidation_condition
        )
        .expect("writing to String cannot fail");
    }

    writeln!(report, "\n## Decisions").expect("writing to String cannot fail");
    for decision in decisions {
        writeln!(report, "\n### {} — {}", decision.id, decision.decision)
            .expect("writing to String cannot fail");
        writeln!(report, "- Maturity: `{}`", decision.maturity.as_str())
            .expect("writing to String cannot fail");
        writeln!(report, "- Contradiction: {}", decision.contradiction)
            .expect("writing to String cannot fail");
        writeln!(report, "- Selected option: {}", decision.selected_option)
            .expect("writing to String cannot fail");
        writeln!(report, "- Rationale: {}", decision.rationale)
            .expect("writing to String cannot fail");
        writeln!(
            report,
            "- Reversibility: `{}`",
            decision.reversibility.as_str()
        )
        .expect("writing to String cannot fail");
        writeln!(
            report,
            "- Evidence refs: {}",
            decision.evidence_refs.join(", ")
        )
        .expect("writing to String cannot fail");
        for alternative in decision.alternatives {
            writeln!(
                report,
                "- Alternative: {} — {} — {}",
                alternative.name,
                if alternative.rejected {
                    "rejected"
                } else {
                    "selected"
                },
                alternative.rationale
            )
            .expect("writing to String cannot fail");
        }
        for risk in decision.risks {
            writeln!(report, "- Risk: {}", risk).expect("writing to String cannot fail");
        }
        for condition in decision.invalidation_conditions {
            writeln!(report, "- Invalidation condition: {}", condition)
                .expect("writing to String cannot fail");
        }
    }

    writeln!(report, "\n## Ranked Options").expect("writing to String cannot fail");
    for (index, option) in options.into_iter().enumerate() {
        writeln!(
            report,
            "\n{}. **{}** — total score {}",
            index + 1,
            option.name,
            option.total_score()
        )
        .expect("writing to String cannot fail");
        writeln!(report, "   - {}", option.description).expect("writing to String cannot fail");
        let mut scores = option.scores;
        scores.sort_by_key(|entry| entry.criterion);
        for entry in scores {
            writeln!(
                report,
                "   - `{}`: weight {} × score {} = {}",
                entry.criterion.as_str(),
                entry.weight,
                entry.score,
                entry.weight * entry.score
            )
            .expect("writing to String cannot fail");
        }
    }

    Ok(report)
}

fn required(value: String, field: &'static str) -> Result<String, ArchitectureEvidenceError> {
    if value.trim().is_empty() {
        Err(ArchitectureEvidenceError::EmptyField(field))
    } else {
        Ok(value)
    }
}

fn required_ref(value: &str, field: &'static str) -> Result<(), ArchitectureEvidenceError> {
    if value.trim().is_empty() {
        Err(ArchitectureEvidenceError::EmptyField(field))
    } else {
        Ok(())
    }
}
