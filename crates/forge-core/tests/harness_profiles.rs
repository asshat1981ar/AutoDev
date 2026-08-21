use std::collections::BTreeSet;

use forge_core::{
    default_harness_profiles, HarnessAssetKind, HarnessAssetRef, HarnessError, HarnessKind,
    HarnessProfile, HarnessRegistry, HarnessStage,
};

fn asset(id: &str) -> HarnessAssetRef {
    HarnessAssetRef {
        id: id.to_string(),
        version: "1".to_string(),
        kind: HarnessAssetKind::Skill,
        required: true,
    }
}

fn stage(id: &str) -> HarnessStage {
    HarnessStage {
        id: id.to_string(),
        objective: format!("execute {id}"),
        assets: vec![asset("superpowers:test-driven-development")],
        verification: vec![format!("verify {id} independently")],
        parallel_group: None,
        approval_gate: false,
    }
}

fn profile() -> HarnessProfile {
    HarnessProfile {
        id: "test-harness".to_string(),
        version: "0.1.0".to_string(),
        name: "Test Harness".to_string(),
        kind: HarnessKind::Sdlc,
        objective: "exercise the normalized harness protocol".to_string(),
        triggers: vec!["feature".to_string()],
        stages: vec![stage("discover"), stage("verify")],
        success_metrics: vec!["verified outcome".to_string()],
        memory_policy: vec!["record evidence-backed outcomes".to_string()],
        improvement_policy: vec!["promote only after independent evaluation".to_string()],
    }
}

#[test]
fn valid_profile_round_trips_through_json() {
    let profile = profile();
    profile.validate().expect("valid profile");

    let json = serde_json::to_string(&profile).expect("serialize harness profile");
    let decoded: HarnessProfile = serde_json::from_str(&json).expect("deserialize harness profile");

    assert_eq!(decoded, profile);
}

#[test]
fn registry_rejects_duplicate_profile_ids() {
    let mut registry = HarnessRegistry::new();
    registry.register(profile()).expect("first registration");

    assert_eq!(
        registry.register(profile()),
        Err(HarnessError::DuplicateProfile("test-harness".to_string()))
    );
}

#[test]
fn profile_rejects_duplicate_stage_ids() {
    let mut candidate = profile();
    candidate.stages = vec![stage("build"), stage("build")];

    assert_eq!(
        candidate.validate(),
        Err(HarnessError::DuplicateStage {
            profile_id: "test-harness".to_string(),
            stage_id: "build".to_string(),
        })
    );
}

#[test]
fn profile_rejects_stage_without_independent_verification() {
    let mut candidate = profile();
    candidate.stages[0].verification.clear();

    assert_eq!(
        candidate.validate(),
        Err(HarnessError::MissingVerification {
            profile_id: "test-harness".to_string(),
            stage_id: "discover".to_string(),
        })
    );
}

#[test]
fn profile_rejects_empty_asset_identity() {
    let mut candidate = profile();
    candidate.stages[0].assets[0].id.clear();

    assert_eq!(
        candidate.validate(),
        Err(HarnessError::InvalidAsset {
            profile_id: "test-harness".to_string(),
            stage_id: "discover".to_string(),
            reason: "asset id must not be empty".to_string(),
        })
    );
}

#[test]
fn default_catalog_contains_exactly_five_stable_profiles() {
    let registry = default_harness_profiles();
    let actual: Vec<(&str, HarnessKind)> = registry
        .profiles()
        .iter()
        .map(|profile| (profile.id.as_str(), profile.kind))
        .collect();

    assert_eq!(
        actual,
        vec![
            ("forgeflow-sdlc", HarnessKind::Sdlc),
            ("sprintmesh-agile", HarnessKind::Agile),
            ("idea-tournament", HarnessKind::Innovation),
            ("optiforge-optimizer", HarnessKind::Optimizer),
            ("harnessforge-meta", HarnessKind::Meta),
        ]
    );
}

#[test]
fn every_builtin_profile_is_valid_and_evidence_bearing() {
    let registry = default_harness_profiles();

    for profile in registry.profiles() {
        profile.validate().expect("built-in profile must validate");
        assert!(
            profile.stages.len() >= 3,
            "{} must have at least three stages",
            profile.id
        );

        let mut stage_ids = BTreeSet::new();
        for stage in &profile.stages {
            assert!(
                stage_ids.insert(stage.id.as_str()),
                "{} repeats stage {}",
                profile.id,
                stage.id
            );
            assert!(
                !stage.verification.is_empty(),
                "{}:{} must carry independent verification",
                profile.id,
                stage.id
            );

            let mut assets = BTreeSet::new();
            for asset in &stage.assets {
                assert!(
                    assets.insert((asset.id.as_str(), asset.version.as_str())),
                    "{}:{} repeats asset {}@{}",
                    profile.id,
                    stage.id,
                    asset.id,
                    asset.version
                );
            }
        }
    }
}
