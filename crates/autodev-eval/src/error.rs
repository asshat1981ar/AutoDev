use thiserror::Error;

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("git command failed: {0}")]
    Git(String),
    #[error("process `{0}` is unavailable")]
    MissingExecutable(String),
    #[error("overlay source failed integrity check: `{0}`")]
    OverlayIntegrity(String),
    #[error("unsafe overlay destination `{0}`")]
    UnsafeOverlayDestination(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Fixture(#[from] crate::fixture::FixtureError),
    #[error(transparent)]
    Evaluation(#[from] forge_core::EvaluationError),
}
