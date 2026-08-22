use chrono::Utc;
use forge_core::amcx_bridge::{
    project_context, project_evidence, project_plan, project_verification, AmcxBridgeError,
    AmcxSourceIdentity, VerifiedArtifactRef,
};
use forge_core::{
    ContextItem, ContextPack, Evidence, ExecPlan, ExecutionRecord, ExecutionStatus, Finding,
    PlanBudget, PlanMilestone, PolicyOutcome, VerificationKind, VerificationReport,
    VerificationResult, VerificationStatus, VerificationVerdict,
};
use serde_json::json;
use sha2::{Digest, Sha256};

fn source() -> AmcxSourceIdentity {
    AmcxSourceIdentity {
        repository: "github:asshat1981ar/AutoDev".into(),
        revision: "deadbeef".into(),
        worktree: ".worktrees/amcx-bridge".into(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn verified_artifact(reference: &str, bytes: &[u8]) -> VerifiedArtifactRef {
    let digest = sha256_hex(bytes);
    VerifiedArtifactRef::from_bytes(reference, bytes, &digest).unwrap()
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

    assert_eq!(projected.source, source());
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
fn verified_artifact_handle_binds_reference_digest_and_bytes() {
    let bytes = br#"{"verdict":"pass"}"#;
    let digest = sha256_hex(bytes);
    let artifact = VerifiedArtifactRef::from_bytes("evidence:verification-report-1", bytes, &digest)
        .unwrap();

    assert_eq!(artifact.reference(), "evidence:verification-report-1");
    assert_eq!(artifact.sha256(), digest);

    assert_eq!(
        VerifiedArtifactRef::from_bytes(
            "evidence:verification-report-1",
            bytes,
            &"b".repeat(64),
        ),
        Err(AmcxBridgeError::InvalidArtifactDigest)
    );
    assert_eq!(
        VerifiedArtifactRef::from_bytes("", bytes, &digest),
        Err(AmcxBridgeError::MissingArtifactReference)
    );
}

#[test]
fn verification_projection_preserves_provenance_without_authority() {
    let report = verification_report();
    let report_bytes = serde_json::to_vec(&report).unwrap();
    let artifact = verified_artifact("evidence:verification-report-1", &report_bytes);
    let projected = project_verification(source(), &report, &artifact).unwrap();
    assert_eq!(projected.verdict, "pass");
    assert_eq!(projected.checks, vec!["unit_tests"]);
    assert_eq!(projected.report_ref, "evidence:verification-report-1");
    assert_eq!(projected.report_sha256, artifact.sha256());
    assert_eq!(projected.completed_at, report.completed_at.to_rfc3339());

    let json = serde_json::to_value(projected).unwrap();
    assert!(json.get("authorization").is_none());
    assert!(json.get("approval").is_none());
    assert!(json.get("approval_ref").is_none());
}

#[test]
fn context_projection_is_reference_only_and_requires_bound_artifact() {
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
    let artifact_bytes = b"immutable context artifact bytes";
    let artifact = verified_artifact("cas:context-1", artifact_bytes);

    let projected = project_context(source(), &pack, &artifact).unwrap();
    assert_eq!(projected.query, "amcx bridge");
    assert_eq!(projected.item_count, 1);
    assert_eq!(projected.total_bytes, 40);
    assert_eq!(projected.artifact_ref, "cas:context-1");
    assert_eq!(projected.artifact_sha256, artifact.sha256());

    let serialized = serde_json::to_string(&projected).unwrap();
    assert!(!serialized.contains("sensitive source body"));
}

#[test]
fn blank_source_identity_fails_closed() {
    let report = verification_report();
    let report_bytes = serde_json::to_vec(&report).unwrap();
    let report_artifact = verified_artifact("evidence:verification-report-1", &report_bytes);

    for mutate in ["repository", "revision", "worktree"] {
        let mut blank = source();
        match mutate {
            "repository" => blank.repository = "   ".into(),
            "revision" => blank.revision = "   ".into(),
            "worktree" => blank.worktree = "   ".into(),
            _ => unreachable!(),
        }
        assert_eq!(
            project_verification(blank, &report, &report_artifact),
            Err(AmcxBridgeError::MissingIdentity)
        );
    }
}
