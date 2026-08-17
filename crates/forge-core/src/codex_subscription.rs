//! Safe client contracts for Codex subscription authentication.
//!
//! This module deliberately exposes only allow-listed account, login, and usage
//! metadata. ChatGPT OAuth credentials remain owned by `codex app-server` and
//! are never represented by public ForgeCore types.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum CodexSubscriptionError {
    #[error("codex subscription protocol error: {0}")]
    Protocol(String),
    #[error("codex app-server provider unavailable: {0}")]
    ProviderUnavailable(String),
    #[error("codex app-server client is not initialized")]
    NotInitialized,
}

pub trait CodexRpcTransport {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, CodexSubscriptionError>;

    fn notify(&mut self, method: &str, params: Value) -> Result<(), CodexSubscriptionError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexServerInfo {
    pub user_agent: String,
    pub platform_family: String,
    pub platform_os: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CodexLoginStart {
    Browser {
        login_id: String,
        auth_url: String,
    },
    DeviceCode {
        login_id: String,
        verification_url: String,
        user_code: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexAccount {
    pub authenticated: bool,
    pub auth_mode: Option<String>,
    pub plan_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexRateLimitWindow {
    pub used_percent: i32,
    pub window_duration_mins: Option<i64>,
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexCredits {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexRateLimitSnapshot {
    pub limit_id: Option<String>,
    pub limit_name: Option<String>,
    pub primary: Option<CodexRateLimitWindow>,
    pub secondary: Option<CodexRateLimitWindow>,
    pub credits: Option<CodexCredits>,
    pub plan_type: Option<String>,
    pub reached_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexRateLimits {
    pub default: CodexRateLimitSnapshot,
    pub by_limit_id: BTreeMap<String, CodexRateLimitSnapshot>,
    pub reset_credits_available: Option<i64>,
}

pub struct CodexSubscriptionClient<T: CodexRpcTransport> {
    transport: T,
    initialized: bool,
}

impl<T: CodexRpcTransport> CodexSubscriptionClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            initialized: false,
        }
    }

    /// Construct a client around a transport that has already completed the
    /// app-server initialization handshake. Primarily useful for deterministic
    /// transport tests and externally managed connections.
    pub fn new_initialized_for_test(transport: T) -> Self {
        Self {
            transport,
            initialized: true,
        }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Move the already-initialized app-server connection into another safe
    /// protocol adapter without exposing any authentication credential.
    pub fn into_transport(self) -> T {
        self.transport
    }

    pub fn initialize(&mut self, version: &str) -> Result<CodexServerInfo, CodexSubscriptionError> {
        if self.initialized {
            return Err(CodexSubscriptionError::Protocol(
                "client is already initialized".into(),
            ));
        }
        let version = version.trim();
        if version.is_empty() {
            return Err(CodexSubscriptionError::Protocol(
                "client version is required".into(),
            ));
        }

        let value = self.transport.request(
            "initialize",
            json!({
                "clientInfo": {
                    "name": "autodev",
                    "title": "AutoDev",
                    "version": version,
                }
            }),
        )?;
        let server = parse_server_info(&value)?;
        self.transport.notify("initialized", json!({}))?;
        self.initialized = true;
        Ok(server)
    }

    pub fn start_browser_login(&mut self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.ensure_initialized()?;
        let value = self
            .transport
            .request("account/login/start", json!({"type": "chatgpt"}))?;
        parse_browser_login(value)
    }

    pub fn start_device_code_login(&mut self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.ensure_initialized()?;
        let value = self
            .transport
            .request("account/login/start", json!({"type": "chatgptDeviceCode"}))?;
        parse_device_code_login(value)
    }

    pub fn account(&mut self) -> Result<CodexAccount, CodexSubscriptionError> {
        self.ensure_initialized()?;
        let value = self.transport.request("account/read", json!({}))?;
        Ok(parse_account(&value))
    }

    pub fn rate_limits(&mut self) -> Result<CodexRateLimits, CodexSubscriptionError> {
        self.ensure_initialized()?;
        let value = self
            .transport
            .request("account/rateLimits/read", json!({}))?;
        parse_rate_limits(&value)
    }

    pub fn logout(&mut self) -> Result<(), CodexSubscriptionError> {
        self.ensure_initialized()?;
        self.transport.request("account/logout", json!({}))?;
        Ok(())
    }

    fn ensure_initialized(&self) -> Result<(), CodexSubscriptionError> {
        if self.initialized {
            Ok(())
        } else {
            Err(CodexSubscriptionError::NotInitialized)
        }
    }
}

fn required_string(value: &Value, key: &str) -> Result<String, CodexSubscriptionError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| CodexSubscriptionError::Protocol(format!("missing {key}")))
}

fn optional_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn parse_server_info(value: &Value) -> Result<CodexServerInfo, CodexSubscriptionError> {
    Ok(CodexServerInfo {
        user_agent: required_string(value, "userAgent")?,
        platform_family: required_string(value, "platformFamily")?,
        platform_os: required_string(value, "platformOs")?,
    })
}

fn parse_browser_login(value: Value) -> Result<CodexLoginStart, CodexSubscriptionError> {
    if value.get("type").and_then(Value::as_str) != Some("chatgpt") {
        return Err(CodexSubscriptionError::Protocol(
            "unexpected browser login response type".into(),
        ));
    }
    Ok(CodexLoginStart::Browser {
        login_id: required_string(&value, "loginId")?,
        auth_url: required_string(&value, "authUrl")?,
    })
}

fn parse_device_code_login(value: Value) -> Result<CodexLoginStart, CodexSubscriptionError> {
    if value.get("type").and_then(Value::as_str) != Some("chatgptDeviceCode") {
        return Err(CodexSubscriptionError::Protocol(
            "unexpected device-code login response type".into(),
        ));
    }
    Ok(CodexLoginStart::DeviceCode {
        login_id: required_string(&value, "loginId")?,
        verification_url: required_string(&value, "verificationUrl")?,
        user_code: required_string(&value, "userCode")?,
    })
}

fn parse_account(value: &Value) -> CodexAccount {
    let account = value.get("account").filter(|account| !account.is_null());
    let auth_mode = account
        .and_then(|account| account.get("type"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let plan_type = account
        .and_then(|account| account.get("planType"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    CodexAccount {
        authenticated: account.is_some(),
        auth_mode,
        plan_type,
    }
}

fn parse_rate_limits(value: &Value) -> Result<CodexRateLimits, CodexSubscriptionError> {
    let default_value = value
        .get("rateLimits")
        .ok_or_else(|| CodexSubscriptionError::Protocol("missing rateLimits".into()))?;
    let default = parse_rate_limit_snapshot(default_value)?;

    let mut by_limit_id = BTreeMap::new();
    if let Some(entries) = value.get("rateLimitsByLimitId").and_then(Value::as_object) {
        for (limit_id, snapshot) in entries {
            by_limit_id.insert(limit_id.clone(), parse_rate_limit_snapshot(snapshot)?);
        }
    }

    let reset_credits_available = value
        .get("rateLimitResetCredits")
        .and_then(|summary| summary.get("availableCount"))
        .and_then(Value::as_i64);

    Ok(CodexRateLimits {
        default,
        by_limit_id,
        reset_credits_available,
    })
}

fn parse_rate_limit_snapshot(
    value: &Value,
) -> Result<CodexRateLimitSnapshot, CodexSubscriptionError> {
    if !value.is_object() {
        return Err(CodexSubscriptionError::Protocol(
            "rate-limit snapshot must be an object".into(),
        ));
    }

    Ok(CodexRateLimitSnapshot {
        limit_id: optional_string(value, "limitId"),
        limit_name: optional_string(value, "limitName"),
        primary: parse_rate_limit_window(value.get("primary"))?,
        secondary: parse_rate_limit_window(value.get("secondary"))?,
        credits: parse_credits(value.get("credits"))?,
        plan_type: optional_string(value, "planType"),
        reached_type: optional_string(value, "rateLimitReachedType"),
    })
}

fn parse_rate_limit_window(
    value: Option<&Value>,
) -> Result<Option<CodexRateLimitWindow>, CodexSubscriptionError> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let used_percent = value
        .get("usedPercent")
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| CodexSubscriptionError::Protocol("invalid usedPercent".into()))?;

    Ok(Some(CodexRateLimitWindow {
        used_percent,
        window_duration_mins: value.get("windowDurationMins").and_then(Value::as_i64),
        resets_at: value.get("resetsAt").and_then(Value::as_i64),
    }))
}

fn parse_credits(value: Option<&Value>) -> Result<Option<CodexCredits>, CodexSubscriptionError> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let has_credits = value
        .get("hasCredits")
        .and_then(Value::as_bool)
        .ok_or_else(|| CodexSubscriptionError::Protocol("invalid hasCredits".into()))?;
    let unlimited = value
        .get("unlimited")
        .and_then(Value::as_bool)
        .ok_or_else(|| CodexSubscriptionError::Protocol("invalid unlimited".into()))?;

    Ok(Some(CodexCredits {
        has_credits,
        unlimited,
        balance: optional_string(value, "balance"),
    }))
}
