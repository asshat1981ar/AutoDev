//! Repository-local evaluation adapter for AutoDev.
//!
//! This crate remains outside ForgeCore's trusted authorization/execution
//! boundary. It owns experiment orchestration and local evaluation adapters.

mod error;
mod fixture;
mod runner;
mod verifier;
mod workspace;

pub use error::RunnerError;
pub use fixture::{load_corpus, load_fixture, EvalFixture, FixtureError, VerifierOverlay};
pub use runner::{AttemptDriver, AttemptMetadata, EvaluationRunner};
pub use verifier::{apply_verifier_overlays, run_verifier, StepExecution};
pub use workspace::{changed_paths, materialize_checkout, IsolatedCheckout};
