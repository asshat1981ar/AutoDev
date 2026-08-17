use chrono::{TimeZone, Utc};
use forge_core::{
    rank_options, render_architecture_report, ArchitectureAlternative, ArchitectureCriterion,
    ArchitectureDecision, ArchitectureEvidenceError, ArchitectureOption, ArchitectureReportInput,
    CriterionScore, DecisionMaturity, EvidenceClass, EvidenceRecord, Reversibility,
};

fn ts() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 17, 8, 0, 0).unwrap()
}

fn evidence(
    id: &str,
    class: EvidenceClass,
    source_system: &str,
    source_reference: &str,
    claim: &str,
) -> EvidenceRecord {
    EvidenceRecord::new(
        id,
        "obj-w1",
        claim,
        class,
        source_system,
        source_reference,
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
        rationale: if rejected {
            "rejected after comparison".into()
        } else {
            "selected after comparison".into()
        },
        rejected,
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
        "docs://serde",
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
        "docs://serde",
        ts(),
        101,
        "normalized finding",
        "library API changes",
    )
    .unwrap_err();
    assert_eq!(
        invalid_confidence,
        ArchitectureEvidenceError::InvalidConfidence(101)
    );
}

#[test]
fn evidence_record_fingerprint_is_content_derived_and_stable() {
    let first = evidence(
        "ev-1",
        EvidenceClass::RepoObserved,
        "github",
        "repo://forge-core",
        "ForgeCore owns trusted execution",
    );
    let second = evidence(
        "ev-1",
        EvidenceClass::RepoObserved,
        "github",
        "repo://forge-core",
        "ForgeCore owns trusted execution",
    );
    let changed = EvidenceRecord::new(
        "ev-1",
        "obj-w1",
        "ForgeCore owns trusted execution",
        EvidenceClass::RepoObserved,
        "github",
        "repo://forge-core",
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
    let records = vec![evidence(
        "ev-hypothesis",
        EvidenceClass::Hypothesis,
        "agent",
        "hypothesis://1",
        "A graph database might improve W1",
    )];

    let decision = ArchitectureDecision {
        id: "dec-hypothesis".into(),
        objective_id: "obj-w1".into(),
        decision: "Adopt graph database".into(),
        alternatives: vec![alternative("keep local domain", true)],
        contradiction: "query flexibility vs operational complexity".into(),
        selected_option: "graph database".into(),
        rationale: "hypothesis only".into(),
        evidence_refs: vec!["ev-hypothesis".into()],
        reversibility: Reversibility::Moderate,
        risks: vec!["unproven operational cost".into()],
        invalidation_conditions: vec!["local domain satisfies retrieval needs".into()],
        maturity: DecisionMaturity::Verified,
    };

    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Hypothesis gate".into(),
        desired_outcome: "Reject unsupported verification".into(),
        evidence: records,
        decisions: vec![decision],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::UnsupportedVerifiedDecision("dec-hypothesis".into())
    );
}

#[test]
fn verified_decision_rejects_unknown_evidence_reference() {
    let decision = ArchitectureDecision {
        id: "dec-unknown".into(),
        objective_id: "obj-w1".into(),
        decision: "Use normalized domain".into(),
        alternatives: vec![alternative("embed connector payloads", true)],
        contradiction: "breadth vs stability".into(),
        selected_option: "normalized domain".into(),
        rationale: "stable boundary".into(),
        evidence_refs: vec!["missing".into()],
        reversibility: Reversibility::Easy,
        risks: vec![],
        invalidation_conditions: vec!["normalization becomes lossy".into()],
        maturity: DecisionMaturity::Verified,
    };

    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "Unknown evidence".into(),
        desired_outcome: "Reject dangling evidence refs".into(),
        evidence: vec![],
        decisions: vec![decision],
        options: vec![],
    };

    assert_eq!(
        render_architecture_report(&input).unwrap_err(),
        ArchitectureEvidenceError::UnknownEvidenceReference(
            "dec-unknown".into(),
            "missing".into()
        )
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
    let equal_a = ArchitectureOption {
        name: "alpha".into(),
        description: "tie A".into(),
        scores: vec![],
    };
    let equal_b = ArchitectureOption {
        name: "beta".into(),
        description: "tie B".into(),
        scores: vec![],
    };

    assert_eq!(high.total_score(), 11);
    let ranked = rank_options(&[equal_b, equal_a, high]);
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
            "repo://forge-core",
            "ForgeCore already owns trusted execution evidence",
        ),
        evidence(
            "ev-context7",
            EvidenceClass::Documented,
            "context7",
            "docs://serde",
            "Serde supports serialization of local domain types",
        ),
        evidence(
            "ev-alphaxiv",
            EvidenceClass::ResearchSupported,
            "alphaxiv",
            "paper://agent-evidence",
            "Evidence-linked workflows support auditability",
        ),
        evidence(
            "ev-hf",
            EvidenceClass::Documented,
            "hugging-face",
            "hub://normalized-fixture",
            "External ecosystem findings can be normalized before trusted boundaries",
        ),
    ];

    let decision = ArchitectureDecision {
        id: "dec-1".into(),
        objective_id: "obj-w1".into(),
        decision: "Keep connector payloads outside ForgeCore domain contracts".into(),
        alternatives: vec![
            alternative("embed connector SDK payloads", true),
            alternative("normalize at orchestration boundary", false),
        ],
        contradiction: "research breadth vs trusted-boundary stability".into(),
        selected_option: "normalize at orchestration boundary".into(),
        rationale: "keeps ForgeCore deterministic and SaaS-neutral".into(),
        evidence_refs: records.iter().map(|item| item.id.clone()).collect(),
        reversibility: Reversibility::Easy,
        risks: vec!["normalizers may lose source-specific detail".into()],
        invalidation_conditions: vec![
            "ForgeCore requires a connector-native capability that cannot be normalized safely".into(),
        ],
        maturity: DecisionMaturity::Verified,
    };

    let input = ArchitectureReportInput {
        objective_id: "obj-w1".into(),
        title: "ConnectorForge W1".into(),
        desired_outcome: "Evidence-backed architecture decisions without SaaS coupling".into(),
        evidence: records,
        decisions: vec![decision],
        options: vec![ArchitectureOption {
            name: "normalized-local-domain".into(),
            description: "Repository-native W1 types".into(),
            scores: vec![
                CriterionScore {
                    criterion: ArchitectureCriterion::EvidenceStrength,
                    weight: 3,
                    score: 5,
                },
                CriterionScore {
                    criterion: ArchitectureCriterion::ImplementationCost,
                    weight: -1,
                    score: 2,
                },
            ],
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
