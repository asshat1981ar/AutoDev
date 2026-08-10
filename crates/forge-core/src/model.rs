//! A provider-neutral model fabric.
//!
//! This module defines the abstraction the orchestration layer talks to, and a
//! concrete Ollama provider. Crucially, **Ollama-specific behavior lives inside
//! [`OllamaProvider`] and never leaks into orchestration** — the orchestrator
//! only sees the [`ModelProvider`] trait and the neutral request/response types.
//!
//! A [`MockProvider`] is provided so orchestration can be tested without an
//! actual model.

use serde::{Deserialize, Serialize};

/// The capabilities a model exposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelCapabilities {
    pub chat: bool,
    pub completion: bool,
    pub embeddings: bool,
    pub tools: bool,
    pub vision: bool,
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        ModelCapabilities {
            chat: true,
            completion: true,
            embeddings: false,
            tools: false,
            vision: false,
        }
    }
}

/// A model known to a provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    /// The provider-unique model id (e.g. "qwen2.5-coder:latest").
    pub id: String,
    /// Display name.
    pub name: String,
    /// Approximate size in bytes, if known.
    pub size: Option<u64>,
    /// The capabilities this model exposes.
    pub capabilities: ModelCapabilities,
}

/// A message in a chat conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// The role: "system", "user", or "assistant".
    pub role: String,
    /// The content.
    pub content: String,
}

/// A request to a model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRequest {
    /// The target model id.
    pub model: String,
    /// Optional chat messages (used for chat-style generation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
    /// Optional single prompt (used for completion-style generation).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Sampling options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<ModelOptions>,
}

/// Sampling options for a request.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ModelOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

impl Default for ModelOptions {
    fn default() -> Self {
        ModelOptions {
            temperature: Some(0.7),
            max_tokens: None,
        }
    }
}

/// Token usage reported by a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// A response from a model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelResponse {
    /// The model that produced the response.
    pub model: String,
    /// The generated content.
    pub content: String,
    /// Token usage.
    pub usage: Usage,
    /// Provider-measured durations in nanoseconds (0 if unknown).
    pub load_ns: u64,
    pub eval_ns: u64,
    /// The provider that served this response ("mock", "ollama", ...).
    pub provider: String,
}

/// The health of a model provider or a specific model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelHealth {
    /// The provider/model is reachable and ready.
    Healthy,
    /// Reachable but degraded (e.g. not loaded).
    Degraded,
    /// Unreachable or unavailable.
    Unavailable,
}

/// Errors produced by the model fabric.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    #[error("model provider error: {0}")]
    Provider(String),
    #[error("no model '{0}' available from provider")]
    ModelNotFound(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("request failed with status {0}: {1}")]
    Http(u16, String),
}

/// A provider-neutral interface for talking to models.
///
/// Orchestration depends only on this trait and the neutral request/response
/// types, so swapping Ollama for another provider (or a mock) is seamless and
/// no provider-specific behavior leaks upward.
pub trait ModelProvider {
    /// List the models this provider can serve.
    fn list_models(&self) -> Result<Vec<Model>, ModelError>;
    /// Generate a completion-style response.
    fn generate(&self, req: &ModelRequest) -> Result<ModelResponse, ModelError>;
    /// Generate a chat-style response.
    fn chat(&self, req: &ModelRequest) -> Result<ModelResponse, ModelError>;
    /// Report provider health.
    fn health(&self) -> ModelHealth;
}

/// The factors a routing policy may consider when choosing a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingFactor {
    Capability,
    Context,
    Latency,
    Availability,
    Privacy,
    Cost,
    LocalRemote,
    TaskType,
}

/// A declarative routing policy: which factors matter and with what weight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingPolicy {
    /// The factors considered, in priority order (first = most important).
    pub factors: Vec<RoutingFactor>,
}

impl Default for RoutingPolicy {
    fn default() -> Self {
        RoutingPolicy {
            factors: vec![
                RoutingFactor::Capability,
                RoutingFactor::Availability,
                RoutingFactor::LocalRemote,
                RoutingFactor::Privacy,
                RoutingFactor::Cost,
                RoutingFactor::Latency,
                RoutingFactor::Context,
                RoutingFactor::TaskType,
            ],
        }
    }
}

impl RoutingPolicy {
    /// Whether a model satisfies the *capability* factor for a task.
    pub fn supports(&self, model: &Model, requested: &ModelCapabilities) -> bool {
        requested.chat <= model.capabilities.chat
            && requested.completion <= model.capabilities.completion
            && requested.tools <= model.capabilities.tools
            && requested.vision <= model.capabilities.vision
    }
}

/// A scored routing candidate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteCandidate {
    pub model: Model,
    pub score: u32,
}

/// Score candidate models against a routing policy + requested capabilities.
///
/// Returns models sorted best-first. Higher score = better fit. This is a
/// deterministic, dependency-light scoring function intended to be extended as
/// providers begin reporting latency/cost.
pub fn route(
    models: &[Model],
    requested: &ModelCapabilities,
    policy: &RoutingPolicy,
) -> Vec<RouteCandidate> {
    let mut candidates: Vec<RouteCandidate> = Vec::new();
    for model in models {
        if !policy.supports(model, requested) {
            continue;
        }
        let mut score: u32 = 0;
        for (i, factor) in policy.factors.iter().enumerate() {
            let priority = (policy.factors.len() - i) as u32;
            match factor {
                RoutingFactor::Capability => score += priority * 10,
                RoutingFactor::Availability => score += priority * 8,
                RoutingFactor::LocalRemote => score += priority * 6,
                RoutingFactor::Privacy => score += priority * 5,
                RoutingFactor::Cost => {
                    // Smaller models score higher on cost.
                    if model.size.unwrap_or(u64::MAX) < 8_000_000_000 {
                        score += priority * 4;
                    }
                }
                RoutingFactor::Latency | RoutingFactor::Context | RoutingFactor::TaskType => {
                    score += priority * 2;
                }
            }
        }
        candidates.push(RouteCandidate {
            model: model.clone(),
            score,
        });
    }
    candidates.sort_by_key(|c| std::cmp::Reverse(c.score));
    candidates
}
/// An [`ModelProvider`] backed by a local Ollama server.
///
/// Talks to the Ollama HTTP API (`/api/tags`, `/api/generate`, `/api/chat`,
/// `/api/ps`) over a configurable base URL (default `http://localhost:11434`).
/// All Ollama-specific wire details are confined to this type.
pub struct OllamaProvider {
    base_url: String,
    agent: ureq::Agent,
}

impl OllamaProvider {
    /// Connect to an Ollama server at `base_url`.
    pub fn new(base_url: impl Into<String>) -> Self {
        OllamaProvider {
            base_url: base_url.into(),
            agent: ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(30))
                .build(),
        }
    }

    /// Connect to the default local Ollama server.
    pub fn local() -> Self {
        OllamaProvider::new("http://localhost:11434")
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }
}

/// The Ollama `/api/tags` response.
#[derive(Debug, Deserialize)]
struct OllamaTags {
    models: Vec<OllamaModel>,
}

/// An entry in `/api/tags`.
#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    size: Option<u64>,
    capabilities: Option<Vec<String>>,
}

/// The normalized Ollama completion/chat response body.
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: Option<String>,
    response: Option<String>,
    message: Option<OllamaMessage>,
    /// Present in the wire format; normalized output does not surface it.
    #[allow(dead_code)]
    done: bool,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    eval_duration: u64,
    #[serde(default)]
    prompt_eval_count: u32,
    #[serde(default)]
    eval_count: u32,
}

#[derive(Debug, Deserialize)]
struct OllamaMessage {
    content: String,
}
impl ModelProvider for OllamaProvider {
    fn list_models(&self) -> Result<Vec<Model>, ModelError> {
        let body = self
            .agent
            .get(&self.url("/api/tags"))
            .call()
            .map_err(map_ureq_err)?
            .into_string()
            .map_err(|e| ModelError::Network(e.to_string()))?;
        let tags: OllamaTags =
            serde_json::from_str(&body).map_err(|e| ModelError::Provider(e.to_string()))?;
        Ok(tags
            .models
            .into_iter()
            .map(|m| Model {
                id: m.name.clone(),
                name: m.name,
                size: m.size,
                capabilities: capabilities_from_flags(m.capabilities.as_deref().unwrap_or(&[])),
            })
            .collect())
    }

    fn generate(&self, req: &ModelRequest) -> Result<ModelResponse, ModelError> {
        let prompt = req
            .prompt
            .clone()
            .ok_or_else(|| ModelError::Provider("generate requires a prompt".into()))?;
        let temperature = req.options.and_then(|o| o.temperature);
        let mut body = serde_json::json!({
            "model": req.model,
            "prompt": prompt,
            "stream": false,
        });
        if let Some(t) = temperature {
            body["options"] = serde_json::json!({ "temperature": t });
        }
        let resp = self.post_json("/api/generate", &body)?;
        let content = resp.response.unwrap_or_default();
        Ok(ModelResponse {
            model: resp.model.unwrap_or_else(|| req.model.clone()),
            content,
            usage: Usage {
                prompt_tokens: resp.prompt_eval_count,
                completion_tokens: resp.eval_count,
            },
            load_ns: resp.total_duration,
            eval_ns: resp.eval_duration,
            provider: "ollama".to_string(),
        })
    }

    fn chat(&self, req: &ModelRequest) -> Result<ModelResponse, ModelError> {
        let messages = req
            .messages
            .clone()
            .ok_or_else(|| ModelError::Provider("chat requires messages".into()))?;
        let temperature = req.options.and_then(|o| o.temperature);
        let mut body = serde_json::json!({
            "model": req.model,
            "messages": messages,
            "stream": false,
        });
        if let Some(t) = temperature {
            body["options"] = serde_json::json!({ "temperature": t });
        }
        let resp = self.post_json("/api/chat", &body)?;
        let content = resp.message.map(|m| m.content).unwrap_or_default();
        Ok(ModelResponse {
            model: resp.model.unwrap_or_else(|| req.model.clone()),
            content,
            usage: Usage {
                prompt_tokens: resp.prompt_eval_count,
                completion_tokens: resp.eval_count,
            },
            load_ns: resp.total_duration,
            eval_ns: resp.eval_duration,
            provider: "ollama".to_string(),
        })
    }

    fn health(&self) -> ModelHealth {
        match self.agent.get(&self.url("/api/ps")).call() {
            Ok(_) => ModelHealth::Healthy,
            Err(_) => ModelHealth::Unavailable,
        }
    }
}

impl OllamaProvider {
    /// POST a JSON body to an Ollama path and parse the normalized response.
    fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<OllamaResponse, ModelError> {
        let text = self
            .agent
            .post(&self.url(path))
            .send_json(body)
            .map_err(map_ureq_err)?
            .into_string()
            .map_err(|e| ModelError::Network(e.to_string()))?;
        serde_json::from_str(&text).map_err(|e| ModelError::Provider(e.to_string()))
    }
}

/// Map a `ureq::Error` into a [`ModelError`].
fn map_ureq_err(e: ureq::Error) -> ModelError {
    match e {
        ureq::Error::Status(status, _) => ModelError::Http(status, "request failed".to_string()),
        ureq::Error::Transport(t) => ModelError::Network(t.to_string()),
    }
}

/// Map Ollama capability flag strings to a [`ModelCapabilities`].
fn capabilities_from_flags(flags: &[String]) -> ModelCapabilities {
    let has = |f: &str| flags.iter().any(|x| x == f);
    ModelCapabilities {
        chat: has("chat") || flags.is_empty(),
        completion: has("completion") || flags.is_empty(),
        embeddings: has("embeddings"),
        tools: has("tools"),
        vision: has("vision"),
    }
}
/// A deterministic mock provider for tests and orchestration development.
///
/// Returns canned responses with fixed content and usage, and always reports
/// healthy. No network access.
pub struct MockProvider {
    pub respond_with: String,
}

impl MockProvider {
    /// Create a mock that always responds with `respond_with`.
    pub fn new(respond_with: impl Into<String>) -> Self {
        MockProvider {
            respond_with: respond_with.into(),
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        MockProvider::new("mock response")
    }
}

impl ModelProvider for MockProvider {
    fn list_models(&self) -> Result<Vec<Model>, ModelError> {
        Ok(vec![Model {
            id: "mock-model".to_string(),
            name: "mock-model".to_string(),
            size: Some(1_000_000_000),
            capabilities: ModelCapabilities {
                chat: true,
                completion: true,
                embeddings: false,
                tools: true,
                vision: false,
            },
        }])
    }

    fn generate(&self, _req: &ModelRequest) -> Result<ModelResponse, ModelError> {
        Ok(ModelResponse {
            model: "mock-model".to_string(),
            content: self.respond_with.clone(),
            usage: Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
            },
            load_ns: 0,
            eval_ns: 0,
            provider: "mock".to_string(),
        })
    }

    fn chat(&self, req: &ModelRequest) -> Result<ModelResponse, ModelError> {
        self.generate(req)
    }

    fn health(&self) -> ModelHealth {
        ModelHealth::Healthy
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_provider_generates_without_network() {
        let provider = MockProvider::new("hello");
        let req = ModelRequest {
            model: "mock-model".to_string(),
            messages: None,
            prompt: Some("hi".to_string()),
            options: None,
        };
        let resp = provider.generate(&req).unwrap();
        assert_eq!(resp.content, "hello");
        assert_eq!(resp.provider, "mock");
        assert_eq!(resp.usage.completion_tokens, 5);
        assert_eq!(provider.health(), ModelHealth::Healthy);
    }

    #[test]
    fn mock_provider_chat_works() {
        let provider = MockProvider::default();
        let req = ModelRequest {
            model: "mock-model".to_string(),
            messages: Some(vec![Message {
                role: "user".to_string(),
                content: "hi".to_string(),
            }]),
            prompt: None,
            options: None,
        };
        let resp = provider.chat(&req).unwrap();
        assert_eq!(resp.content, "mock response");
    }

    #[test]
    fn routing_selects_models_that_support_requested_capabilities() {
        let models = vec![
            Model {
                id: "chat-only".to_string(),
                name: "chat-only".to_string(),
                size: Some(2_000_000_000),
                capabilities: ModelCapabilities {
                    chat: true,
                    completion: false,
                    embeddings: false,
                    tools: false,
                    vision: false,
                },
            },
            Model {
                id: "full".to_string(),
                name: "full".to_string(),
                size: Some(16_000_000_000),
                capabilities: ModelCapabilities {
                    chat: true,
                    completion: true,
                    embeddings: false,
                    tools: true,
                    vision: false,
                },
            },
        ];
        let policy = RoutingPolicy::default();
        // Request tools -> only "full" qualifies.
        let requested = ModelCapabilities {
            chat: true,
            completion: false,
            embeddings: false,
            tools: true,
            vision: false,
        };
        let ranked = route(&models, &requested, &policy);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].model.id, "full");
        assert!(ranked[0].score > 0);
    }

    #[test]
    fn routing_prefers_smaller_models_on_cost() {
        let models = vec![
            Model {
                id: "big".to_string(),
                name: "big".to_string(),
                size: Some(30_000_000_000),
                capabilities: ModelCapabilities::default(),
            },
            Model {
                id: "small".to_string(),
                name: "small".to_string(),
                size: Some(1_000_000_000),
                capabilities: ModelCapabilities::default(),
            },
        ];
        let policy = RoutingPolicy {
            factors: vec![RoutingFactor::Cost],
        };
        let requested = ModelCapabilities::default();
        let ranked = route(&models, &requested, &policy);
        assert_eq!(ranked[0].model.id, "small");
    }

    #[test]
    fn routing_drops_models_lacking_capabilities() {
        let models = vec![Model {
            id: "no-vision".to_string(),
            name: "no-vision".to_string(),
            size: Some(1),
            capabilities: ModelCapabilities::default(),
        }];
        let requested = ModelCapabilities {
            chat: true,
            completion: true,
            embeddings: false,
            tools: false,
            vision: true, // no-vision lacks this
        };
        let ranked = route(&models, &requested, &RoutingPolicy::default());
        assert!(ranked.is_empty());
    }

    #[test]
    fn routing_policy_supports_checks_capabilities() {
        let model = Model {
            id: "m".to_string(),
            name: "m".to_string(),
            size: None,
            capabilities: ModelCapabilities::default(),
        };
        let policy = RoutingPolicy::default();
        assert!(policy.supports(&model, &ModelCapabilities::default()));
        assert!(!policy.supports(
            &model,
            &ModelCapabilities {
                chat: true,
                completion: true,
                embeddings: false,
                tools: true, // model has no tools
                vision: false,
            }
        ));
    }
}
