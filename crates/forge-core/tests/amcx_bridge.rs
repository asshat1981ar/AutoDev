use chrono::Utc;
use forge_core::amcx_bridge::{
    project_context, project_evidence, project_plan, project_verification, AmcxBridgeError,
    AmcxSourceIdentity,
};
use forge_core::{
    ContextItem, ContextPack, Evidence, ExecPlan, ExecutionRecord, ExecutionStatus, Finding,
    PlanBudget, PlanMilestone, PolicyOutcome, VerificationKind, VerificationReport,
    VerificationResult, VerificationStatus, VerificationVerdict,
};
use serde_json::json;

fn source() -> AmcxSourceIdentity {
    AmcxSourceIdentity {
        repository: "github:asshat1981ar/AutoDev".into(),
        revision: "deadbeef".into(),
        worktree: ".worktrees/amcx-bridge".into(),
    }
}

fn evidence() -> Evidence {
    Evidence::from_record(ExecutionRecord {
        id: "evidence-1".into(),
        task_id: "task-1".into(),
        agent_id: "agent-1".into(),
        action_id: "action-1".into(),
        action: json!({"id":"action-1","type":"read_file"}),
        policy: PolicyOutcome::Allow,
        status: ExecutionStatus::Succeeded,
        started_at: Utc::now(),
        completed_at: Utc::now(),
        error: None,
        artifacts: vec![],
        verification: None,
    })
}

fn verification_report() -> VerificationReport {
    let now = Utc::now();
    VerificationReport {
        results: vec![VerificationResult {
            kind: VerificationKind::UnitTests,
            status: VerificationStatus::Passed,
            tool: "cargo test".into(),
            summary: "passed".into(),
            findings: Vec::<Finding>::new(),
            started_at: now,
            completed_at: now,
        }],
        overall: VerificationVerdict::Pass,
        completed_at: now,
    }
}

#[test]
fn plan_projection_retains_identity_without_mutating_plan() {
    let mut plan = ExecPlan::new("plan-1", "bridge AMCX", PlanBudget::new(2, 2));
    plan.add_milestone(PlanMilestone::new("m1", "projection"))
        .unwrap();
    plan.start().unwrap();
    let checkpoint = plan.checkpoint("checkpoint-1").unwrap();
    let before_status = plan.status();
    let before_milestones = plan.milestones().to_vec();

    let projected = project_plan(source(), &plan, &checkpoint).unwrap();

    assert_eq!(projected.plan_id, "plan-1");
    assert_eq!(projected.checkpoint_id, "checkpoint-1");
    assert_eq!(projected.status, "running");
    assert_eq!(plan.status(), before_status);
    assert_eq!(plan.milestones(), before_milestones.as_slice());
}

#[test]
fn evidence_projection_requires_verified_fingerprint() {
    let good = evidence();
    let projected = project_evidence(source(), &good).unwrap();
    assert_eq!(projected.evidence_id, "evidence-1");
    assert_eq!(projected.fingerprint_sha256, good.fingerprint.digest);

    let mut tampered = good.clone();
    tampered.record.status = ExecutionStatus::Failed;
    assert_eq!(
        project_evidence(source(), &tampered),
        Err(AmcxBridgeError::InvalidEvidenceFingerprint)
    );
}

#[test]
fn verification_projection_preserves_verdict_as_evidence_only() {
    let projected = project_verification(source(), &verification_report()).unwrap();
    assert_eq!(projected.verdict, "pass");
    assert_eq!(projected.checks, vec!["unit_tests"]);

    let json = serde_json::to_value(projected).unwrap();
    assert!(json.get("authorization").is_none());
    assert!(json.get("approval").is_none());
    assert!(json.get("approval_ref").is_none());
}

#[test]
fn context_projection_is_reference_only_and_requires_sha256() {
    let pack = ContextPack {
        query: "amcx bridge".into(),
        items: vec![ContextItem {
            path: "crates/forge-core/src/lib.rs".into(),
            score: 12,
            reasons: vec!["source".into()],
            content: "sensitive source body must not be copied".into(),
        }],
        total_bytes: 40,
    };
    let digest = "a".repeat(64);

    let projected = project_context(source(), &pack, "cas:context-1", &digest).unwrap();
    assert_eq!(projected.query, "amcx bridge");
    assert_eq!(projected.item_count, 1);
    assert_eq!(projected.total_bytes, 40);
    assert_eq!(projected.artifact_ref, "cas:context-1");
    assert_eq!(projected.artifact_sha256, digest);

    let serialized = serde_json::to_string(&projected).unwrap();
    assert!(!serialized.contains("sensitive source body"));

    assert_eq!(
        project_context(source(), &pack, "cas:context-1", "abc123"),
        Err(AmcxBridgeError::InvalidArtifactDigest)
    );
}

#[test]
fn blank_source_identity_fails_closed() {
    let mut blank = source();
    blank.repository = "   ".into();
    assert_eq!(
        project_verification(blank, &verification_report()),
        Err(AmcxBridgeError::MissingIdentity)
    );

    let pack = ContextPack {
        query: "x".into(),
        items: vec![],
        total_bytes: 0,
    };
    assert_eq!(
        project_context(source(), &pack, "", &"a".repeat(64)),
        Err(AmcxBridgeError::MissingArtifactReference)
    );
}
