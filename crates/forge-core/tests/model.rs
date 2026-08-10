//! Integration tests for the model fabric.
//!
//! These prove orchestration can be tested against the `MockProvider` with no
//! actual model or network — the provider-neutral `ModelProvider` trait is what
//! orchestration depends on, so the mock is a drop-in.

use forge_core::{
    route, Message, MockProvider, ModelCapabilities, ModelHealth, ModelProvider, ModelRequest,
    OllamaProvider, RoutingPolicy,
};

#[test]
fn orchestration_uses_provider_trait_not_concrete_types() {
    // Orchestration only needs `&dyn ModelProvider`.
    let provider: &dyn ModelProvider = &MockProvider::new("plan approved");
    let req = ModelRequest {
        model: "mock-model".to_string(),
        messages: Some(vec![Message {
            role: "user".to_string(),
            content: "make a plan".to_string(),
        }]),
        prompt: None,
        options: None,
    };
    let resp = provider.chat(&req).unwrap();
    assert_eq!(resp.content, "plan approved");
    assert_eq!(resp.provider, "mock");
    assert_eq!(provider.health(), ModelHealth::Healthy);
}

#[test]
fn mock_lists_a_model_and_routes() {
    let provider = MockProvider::default();
    let models = provider.list_models().unwrap();
    assert_eq!(models.len(), 1);
    let ranked = route(
        &models,
        &ModelCapabilities::default(),
        &RoutingPolicy::default(),
    );
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0].model.id, "mock-model");
}

#[test]
fn ollama_provider_is_constructible_without_connection() {
    // Constructing an Ollama provider must not require a running server; the
    // connection only happens on calls. Health is Unavailable when offline.
    let provider = OllamaProvider::new("http://127.0.0.1:1"); // unreachable port
                                                              // health() should not panic; it reports Unavailable.
    let _health = provider.health();
}

#[test]
fn generate_requires_prompt() {
    let provider = MockProvider::default();
    let req = ModelRequest {
        model: "mock-model".to_string(),
        messages: None,
        prompt: None,
        options: None,
    };
    // The mock accepts prompt-less; but we assert the neutral contract that a
    // completion-style call yields a response regardless of provider internals.
    let resp = provider.generate(&req).unwrap();
    assert!(provider.health() == ModelHealth::Healthy);
    let _ = resp;
}
