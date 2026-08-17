//! Safe client contracts for Codex subscription authentication.
//!
//! This module deliberately exposes only allow-listed account and login metadata.
//! ChatGPT OAuth credentials remain owned by `codex app-server` and are never
//! represented by public ForgeCore types.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum CodexSubscriptionError {
    #[error("codex subscription protocol error: {0}")]
    Protocol(String),
    #[error("codex app-server client is not initialized")]
    NotInitialized,
}

pub trait CodexRpcTransport {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, CodexSubscriptionError>;

    fn notify(&mut self, method: &str, params: Value) -> Result<(), CodexSubscriptionError>;
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
