use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path};

use forge_core::EvalTask;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalFixture {
    pub task: EvalTask,
    #[serde(default)]
    pub verifier_overlay: Vec<VerifierOverlay>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifierOverlay {
    pub source_path: String,
    pub destination_path: String,
    pub sha256: String,
}

#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("failed to read fixture `{path}`: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid fixture JSON `{path}`: {source}")]
    Json {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Task(#[from] forge_core::EvaluationError),
    #[error("duplicate task id `{0}`")]
    DuplicateTaskId(String),
    #[error("invalid overlay path `{0}`")]
    UnsafeOverlayPath(String),
    #[error("invalid overlay sha256 `{0}`")]
    InvalidOverlayDigest(String),
    #[error("overlay digests do not match verifier asset fingerprints for task `{0}`")]
    OverlayFingerprintMismatch(String),
}

pub fn load_fixture(path: impl AsRef<Path>) -> Result<EvalFixture, FixtureError> {
    let path = path.as_ref();
    let display = path.display().to_string();
    let bytes = fs::read(path).map_err(|source| FixtureError::Read {
        path: display.clone(),
        source,
    })?;
    let fixture: EvalFixture =
        serde_json::from_slice(&bytes).map_err(|source| FixtureError::Json {
            path: display,
            source,
        })?;

    validate_fixture(&fixture)?;
    Ok(fixture)
}

pub fn load_corpus(dir: impl AsRef<Path>) -> Result<Vec<EvalFixture>, FixtureError> {
    let dir = dir.as_ref();
    let mut paths = fs::read_dir(dir)
        .map_err(|source| FixtureError::Read {
            path: dir.display().to_string(),
            source,
        })?
        .map(|entry| {
            entry.map(|value| value.path()).map_err(|source| FixtureError::Read {
                path: dir.display().to_string(),
                source,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| path.extension() == Some(OsStr::new("json")));
    paths.sort();

    let mut fixtures = Vec::with_capacity(paths.len());
    let mut ids = BTreeSet::new();
    for path in paths {
        let fixture = load_fixture(path)?;
        if !ids.insert(fixture.task.id.clone()) {
            return Err(FixtureError::DuplicateTaskId(fixture.task.id));
        }
        fixtures.push(fixture);
    }
    fixtures.sort_by(|left, right| left.task.id.cmp(&right.task.id));
    Ok(fixtures)
}

fn validate_fixture(fixture: &EvalFixture) -> Result<(), FixtureError> {
    fixture.task.validate()?;

    let mut overlay_digests = Vec::with_capacity(fixture.verifier_overlay.len());
    for overlay in &fixture.verifier_overlay {
        if !safe_relative_file(&overlay.source_path) {
            return Err(FixtureError::UnsafeOverlayPath(
                overlay.source_path.clone(),
            ));
        }
        if !safe_relative_file(&overlay.destination_path) {
            return Err(FixtureError::UnsafeOverlayPath(
                overlay.destination_path.clone(),
            ));
        }
        if !full_sha256(&overlay.sha256) {
            return Err(FixtureError::InvalidOverlayDigest(overlay.sha256.clone()));
        }
        overlay_digests.push(overlay.sha256.clone());
    }

    overlay_digests.sort();
    let mut declared = fixture.task.verifier.asset_fingerprints.clone();
    declared.sort();
    if overlay_digests != declared {
        return Err(FixtureError::OverlayFingerprintMismatch(
            fixture.task.id.clone(),
        ));
    }

    Ok(())
}

fn safe_relative_file(value: &str) -> bool {
    let path = Path::new(value);
    !path.as_os_str().is_empty()
        && value != "."
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn full_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
