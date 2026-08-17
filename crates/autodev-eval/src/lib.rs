//! Repository-local evaluation adapter for AutoDev.
//!
//! This crate remains outside ForgeCore's trusted authorization/execution
//! boundary. It owns experiment orchestration and local evaluation adapters.

mod fixture;

pub use fixture::{load_corpus, load_fixture, EvalFixture, FixtureError, VerifierOverlay};
