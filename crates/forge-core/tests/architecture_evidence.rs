use chrono::{TimeZone, Utc};
use forge_core::rank_options;
use forge_core::render_architecture_report;
use forge_core::ArchitectureAlternative;
use forge_core::ArchitectureCriterion;
use forge_core::ArchitectureDecision;
use forge_core::ArchitectureEvidenceError;
use forge_core::ArchitectureOption;
use forge_core::ArchitectureReportInput;
use forge_core::CriterionScore;
use forge_core::DecisionMaturity;
use forge_core::EvidenceClass;
use forge_core::EvidenceRecord;
use forge_core::Reversibility;

fn ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 17, 8, 0, 0).unwrap()
}

fn evidence(id: &str, class: EvidenceClass, system: &str, claim: &str) -> EvidenceRecord {
    EvidenceRecord::new(
        id,
        "obj-w1",
        claim,
        class,
        system,
        "fixture://source",
        ts(),
        90,
        claim,
        "source or repository state materially changes",
    )
    .unwrap()
}

fn alternative(name: &str, rejected: bool) -> ArchitectureAlternative {
    ArchitectureAlternative {
        name: name.into(),
        rationale: "compared against selected architecture".into(),
        rejected,
    }
}

fn decision(
    id: &str,
    maturity: DecisionMaturity,
    evidence_refs: Vec<String>,
) -> ArchitectureDecision {
    ArchitectureDecision {
        id: id.into(),
        objective_id: "obj-w1".into(),
        decision: "Keep connector payloads outside ForgeCore contracts".into(),
        alternatives: vec![alternative("embed connector payloads", true)],
        contradiction: "research breadth vs trusted-boundary stability".into(),
        selected_option: "normalize at orchestration boundary".into(),
        rationale: "keeps ForgeCore deterministic and SaaS-neutral".into(),
        evidence_refs,
        reversibility: Reversibility::Easy,
        risks: vec!["normalizers may lose source-specific detail".into()],
        invalidation_conditions: vec!["normalization becomes materially lossy".into()],
        maturity,
    }
}

#[test]
fn evidence_classes_expose_verified_gate_semantics() {
    assert!(EvidenceClass::RepoObserved.can_satisfy_verified_gate());
    assert!(EvidenceClass::Documented.can_satisfy_verified_gate());
    assert!(EvidenceClass::ResearchSupported.can_satisfy_verified_gate());
    assert!(EvidenceClass::ExperimentallyVerified.can_satisfy_verified_gate());
    assert!(!EvidenceClass::Inferred.can_satisfy_verified_gate());
    assert!(!EvidenceClass::Hypothesis.can_satisfy_verified_gate());
}

#[test]
fn evidence_record_validates_required_fields_and_confidence() {
    let empty_claim = EvidenceRecord::new(
        "ev-1",
        "obj-w1",
        "",
        EvidenceClass::Documented,
        "context7",
        "fixture://source",
        ts(),
        90,
        "normalized finding",
        "library API changes",
    )
    .unwrap_err();
    assert_eq!(empty_claim, ArchitectureEvidenceError::EmptyField("claim"));

    let invalid_confidence = EvidenceRecord::new(
        "ev-2",
        "obj-w1",
        "Serde supports serialization",
        EvidenceClass::Documented,
        "context7",
        "fixture://source",
        ts(),
        101,
        "normalized finding",
        "library API changes",
    )
    .unwrap_err();
    assert_eq!(
        invalid_confidence,
        ArchitectureEvidenceError::InvalidConfidence(101),
    );
}

#[test]
fn fingerprint_is_content_derived_and_stable() {
    let first = evidence(
        "ev-1",
        EvidenceClass::RepoObserved,
        "github",
        "ForgeCore owns trusted execution",
    );
    let second = evidence(
        "ev-1",
        EvidenceClass::RepoObserved,
        "github",
        "ForgeCore owns trusted execution",
    );
    let changed = EvidenceRecord::new(
        "ev-1",
        "obj-w1",
        "ForgeCore owns trusted execution",
        EvidenceClass::RepoObserved,
        "github",
        "fixture://source",
        ts(),
        90,
        "different normalized content",
        "source or repository state materially changes",
    )
    .unwrap();

    assert_eq!(first.content_fingerprint, second.content_fingerprint);
    assert_ne!(first.content_fingerprint, changed.content_fingerprint);
}

#[test]
fn hypothesis_only_decision_cannot_be_verified() {
    let record = evidence(
        "ev-hypothesis",
        EvidenceClass::Hypothesis,
        "agent",
        "A graph database might improve W1",
    );
    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Hypothesis gate".into(),
        desired_outcome: "Reject unsupported verification".into(),
        evidence: vec![record],
        decisions: vec![decision(
            "dec-hypothesis",
            DecisionMaturity::Verified,
            vec!["ev-hypothesis".into()],
        )],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::UnsupportedVerifiedDecision("dec-hypothesis".into()),
    );
}

#[test]
fn verified_decision_rejects_unknown_evidence_reference() {
    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Unknown evidence".into(),
        desired_outcome: "Reject dangling evidence refs".into(),
        evidence: vec![],
        decisions: vec![decision(
            "dec-unknown",
            DecisionMaturity::Verified,
            vec!["missing".into()],
        )],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::UnknownEvidenceReference(
            "dec-unknown".into(),
            "missing".into(),
        ),
    );
}

#[test]
fn option_scoring_is_weighted_and_ranking_is_deterministic() {
    let high = ArchitectureOption {
        name: "normalized-local-domain".into(),
        description: "Repository-native contracts".into(),
        scores: vec![
            CriterionScore {
                criterion: ArchitectureCriterion::EvidenceStrength,
                weight: 3,
                score: 5,
            },
            CriterionScore {
                criterion: ArchitectureCriterion::ImplementationCost,
                weight: -2,
                score: 2,
            },
        ],
    };
    let alpha = ArchitectureOption {
        name: "alpha".into(),
        description: "tie A".into(),
        scores: vec![],
    };
    let beta = ArchitectureOption {
        name: "beta".into(),
        description: "tie B".into(),
        scores: vec![],
    };

    assert_eq!(high.total_score(), 11);
    let ranked = rank_options(&[beta, alpha, high]);
    assert_eq!(ranked[0].name, "normalized-local-domain");
    assert_eq!(ranked[1].name, "alpha");
    assert_eq!(ranked[2].name, "beta");
}

#[test]
fn normalized_connector_findings_render_without_live_connectors() {
    let records = vec![
        evidence(
            "ev-github",
            EvidenceClass::RepoObserved,
            "github",
            "ForgeCore owns trusted execution evidence",
        ),
        evidence(
            "ev-context7",
            EvidenceClass::Documented,
            "context7",
            "Serde supports serialization of local domain types",
        ),
        evidence(
            "ev-alphaxiv",
            EvidenceClass::ResearchSupported,
            "alphaxiv",
            "Evidence-linked workflows support auditability",
        ),
        evidence(
            "ev-hf",
            EvidenceClass::Documented,
            "hugging-face",
            "External ecosystem findings can be normalized",
        ),
    ];
    let evidence_refs = records.iter().map(|item| item.id.clone()).collect();
    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "ConnectorForge W1".into(),
        desired_outcome: "Evidence-backed architecture without SaaS coupling".into(),
        evidence: records,
        decisions: vec![decision(
            "dec-1",
            DecisionMaturity::Verified,
            evidence_refs,
        )],
        options: vec![ArchitectureOption {
            name: "normalized-local-domain".into(),
            description: "Repository-native W1 types".into(),
            scores: vec![CriterionScore {
                criterion: ArchitectureCriterion::EvidenceStrength,
                weight: 3,
                score: 5,
            }],
        }],
    };

    let first = render_architecture_report(&input).unwrap();
    let second = render_architecture_report(&input).unwrap();
    assert_eq!(first, second);
    assert!(first.contains("# Architecture Evidence Report: ConnectorForge W1"));
    assert!(first.contains("## Evidence"));
    assert!(first.contains("github"));
    assert!(first.contains("context7"));
    assert!(first.contains("alphaxiv"));
    assert!(first.contains("hugging-face"));
    assert!(first.contains("## Decisions"));
    assert!(first.contains("## Ranked Options"));
    assert!(first.contains("normalized-local-domain"));
}
