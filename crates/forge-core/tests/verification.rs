//! Integration tests for the verification fabric: checks run independently of
//! code generation and their results can be recorded as evidence.

use forge_core as fc;
use forge_core::{
    mock_verifier, verdict_from_report, VerificationContext, VerificationFabric, VerificationKind,
};

#[test]
fn verification_is_independent_of_generation() {
    // A fabric of checks that has nothing to do with how code was generated.
    let fabric = VerificationFabric::new()
        .with(
            VerificationKind::UnitTests,
            mock_verifier(VerificationKind::UnitTests, true),
        )
        .with(
            VerificationKind::Build,
            mock_verifier(VerificationKind::Build, true),
        )
        .with(
            VerificationKind::Lint,
            mock_verifier(VerificationKind::Lint, true),
        )
        .with(
            VerificationKind::StaticAnalysis,
            mock_verifier(VerificationKind::StaticAnalysis, true),
        )
        .with(
            VerificationKind::Security,
            mock_verifier(VerificationKind::Security, true),
        );

    let ctx = VerificationContext {
        workspace: ".".to_string(),
        changed: vec!["src/lib.rs".to_string()],
    };
    let report = fabric.run(&ctx);
    assert_eq!(report.results.len(), 5); // all five check kinds
    assert_eq!(report.overall, fc::VerificationVerdict::Pass);
    // The fabric speaks the orchestrator's verdict language.
    assert_eq!(verdict_from_report(&report), fc::Verdict::Pass);
}

#[test]
fn security_failure_blocks_the_verdict() {
    let fabric = VerificationFabric::new()
        .with(
            VerificationKind::Build,
            mock_verifier(VerificationKind::Build, true),
        )
        .with(
            VerificationKind::Security,
            mock_verifier(VerificationKind::Security, false),
        );
    let report = fabric.run(&VerificationContext {
        workspace: ".".to_string(),
        changed: vec![],
    });
    assert_eq!(report.overall, fc::VerificationVerdict::Fail);
    assert_eq!(verdict_from_report(&report), fc::Verdict::Fail);
}

#[test]
fn report_serializes_as_evidence() {
    let fabric = VerificationFabric::new().with(
        VerificationKind::UnitTests,
        mock_verifier(VerificationKind::UnitTests, true),
    );
    let report = fabric.run(&VerificationContext {
        workspace: ".".to_string(),
        changed: vec!["a.rs".to_string()],
    });
    // The report is a self-contained JSON document (evidence-ready).
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["overall"], "pass");
    assert_eq!(json["results"][0]["kind"], "unit_tests");
    assert_eq!(json["results"][0]["status"], "passed");
}

#[test]
fn command_verifier_refuses_invalid_workspace() {
    // Adversarial: a verifier must not be able to run a command in a path
    // the kernel has not validated as a workspace. The verifier should
    // fail closed with an Errored status, not run with an arbitrary cwd.
    use forge_core::command_verifier;
    let fabric = VerificationFabric::new().with(
        VerificationKind::Build,
        command_verifier(
            VerificationKind::Build,
            "true",
            vec![String::from("--help")],
        ),
    );
    let report = fabric.run(&VerificationContext {
        workspace: "/this/path/does/not/exist/at/all".to_string(),
        changed: vec![],
    });
    let result = &report.results[0];
    assert_eq!(result.status, fc::VerificationStatus::Errored);
    assert!(
        result.summary.contains("invalid workspace"),
        "summary should name the confinement failure: {}",
        result.summary
    );
}
