use std::collections::VecDeque;

use forge_core::{
    CodexLoginStart, CodexRateLimits, CodexRpcTransport, CodexSubscriptionClient,
    CodexSubscriptionError,
};
use serde_json::{json, Value};

#[derive(Default)]
struct FakeTransport {
    calls: Vec<(String, Value)>,
    responses: VecDeque<Value>,
}

impl FakeTransport {
    fn with_responses(responses: impl IntoIterator<Item = Value>) -> Self {
        Self {
            calls: Vec::new(),
            responses: responses.into_iter().collect(),
        }
    }
}

impl CodexRpcTransport for FakeTransport {
    fn request(&mut self, method: &str, params: Value) -> Result<Value, CodexSubscriptionError> {
        self.calls.push((method.to_string(), params));
        self.responses
            .pop_front()
            .ok_or_else(|| CodexSubscriptionError::Protocol("missing fake response".into()))
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<(), CodexSubscriptionError> {
        self.calls.push((method.to_string(), params));
        Ok(())
    }
}

#[test]
fn initialization_sends_initialize_then_initialized() {
    let transport = FakeTransport::with_responses([json!({
        "userAgent": "codex-app-server/1.0.0",
        "codexHome": "/private/codex-home",
        "platformFamily": "unix",
        "platformOs": "linux"
    })]);
    let mut client = CodexSubscriptionClient::new(transport);

    client.initialize("0.1.0").expect("client initializes");

    assert_eq!(
        client.transport().calls,
        vec![
            (
                "initialize".into(),
                json!({
                    "clientInfo": {
                        "name": "autodev",
                        "title": "AutoDev",
                        "version": "0.1.0"
                    }
                })
            ),
            ("initialized".into(), json!({})),
        ]
    );
}

#[test]
fn login_is_rejected_before_initialization_without_transport_io() {
    let transport = FakeTransport::default();
    let mut client = CodexSubscriptionClient::new(transport);

    let error = client
        .start_browser_login()
        .expect_err("login must require initialization");

    assert!(matches!(error, CodexSubscriptionError::NotInitialized));
    assert!(client.transport().calls.is_empty());
}

#[test]
fn repeated_initialization_is_rejected_locally() {
    let transport = FakeTransport::with_responses([json!({
        "userAgent": "codex-app-server/1.0.0",
        "codexHome": "/private/codex-home",
        "platformFamily": "unix",
        "platformOs": "linux"
    })]);
    let mut client = CodexSubscriptionClient::new(transport);
    client.initialize("0.1.0").expect("first initialization");

    let error = client
        .initialize("0.1.0")
        .expect_err("second initialization must fail");

    assert!(matches!(error, CodexSubscriptionError::Protocol(_)));
    assert_eq!(client.transport().calls.len(), 2);
}

#[test]
fn browser_login_uses_managed_chatgpt_mode_only() {
    let transport = FakeTransport::with_responses([json!({
        "type": "chatgpt",
        "loginId": "login-1",
        "authUrl": "https://chatgpt.com/auth"
    })]);
    let mut client = CodexSubscriptionClient::new_initialized_for_test(transport);

    let result = client.start_browser_login().expect("browser login starts");

    assert_eq!(
        result,
        CodexLoginStart::Browser {
            login_id: "login-1".into(),
            auth_url: "https://chatgpt.com/auth".into(),
        }
    );
    assert_eq!(
        client.transport().calls,
        vec![("account/login/start".into(), json!({"type": "chatgpt"}))]
    );
}

#[test]
fn device_code_login_uses_managed_device_code_mode_only() {
    let transport = FakeTransport::with_responses([json!({
        "type": "chatgptDeviceCode",
        "loginId": "login-2",
        "verificationUrl": "https://auth.openai.com/codex/device",
        "userCode": "ABCD-1234"
    })]);
    let mut client = CodexSubscriptionClient::new_initialized_for_test(transport);

    let result = client
        .start_device_code_login()
        .expect("device-code login starts");

    assert_eq!(
        result,
        CodexLoginStart::DeviceCode {
            login_id: "login-2".into(),
            verification_url: "https://auth.openai.com/codex/device".into(),
            user_code: "ABCD-1234".into(),
        }
    );
    assert_eq!(
        client.transport().calls,
        vec![(
            "account/login/start".into(),
            json!({"type": "chatgptDeviceCode"})
        )]
    );
}

#[test]
fn account_maps_only_safe_subscription_metadata() {
    let transport = FakeTransport::with_responses([json!({
        "account": {
            "type": "chatgpt",
            "email": "person@example.com",
            "planType": "plus",
            "accessToken": "must-not-escape",
            "refreshToken": "must-not-escape"
        },
        "requiresOpenaiAuth": true
    })]);
    let mut client = CodexSubscriptionClient::new_initialized_for_test(transport);

    let account = client.account().expect("account maps");
    let serialized = serde_json::to_value(&account).expect("account serializes");
    let serialized_text = serialized.to_string();

    assert!(account.authenticated);
    assert_eq!(account.auth_mode.as_deref(), Some("chatgpt"));
    assert_eq!(account.plan_type.as_deref(), Some("plus"));
    assert!(!serialized_text.contains("accessToken"));
    assert!(!serialized_text.contains("refreshToken"));
    assert!(!serialized_text.contains("must-not-escape"));
}

#[test]
fn rate_limits_map_only_safe_usage_metadata() {
    let transport = FakeTransport::with_responses([json!({
        "rateLimits": {
            "limitId": "codex",
            "limitName": "Codex",
            "primary": {
                "usedPercent": 42,
                "windowDurationMins": 300,
                "resetsAt": 1_800_000_000
            },
            "secondary": null,
            "credits": {
                "hasCredits": true,
                "unlimited": false,
                "balance": "12.50"
            },
            "planType": "plus",
            "rateLimitReachedType": null,
            "accessToken": "must-not-escape"
        },
        "rateLimitsByLimitId": {
            "codex": {
                "limitId": "codex",
                "limitName": "Codex",
                "primary": {
                    "usedPercent": 42,
                    "windowDurationMins": 300,
                    "resetsAt": 1_800_000_000
                },
                "secondary": null,
                "credits": null,
                "planType": "plus",
                "rateLimitReachedType": null
            }
        },
        "rateLimitResetCredits": {
            "availableCount": 2,
            "refreshToken": "must-not-escape"
        }
    })]);
    let mut client = CodexSubscriptionClient::new_initialized_for_test(transport);

    let limits: CodexRateLimits = client.rate_limits().expect("rate limits map");

    assert_eq!(limits.default.limit_id.as_deref(), Some("codex"));
    assert_eq!(limits.default.plan_type.as_deref(), Some("plus"));
    assert_eq!(
        limits
            .default
            .primary
            .as_ref()
            .map(|window| window.used_percent),
        Some(42)
    );
    assert_eq!(limits.reset_credits_available, Some(2));
    assert!(limits.by_limit_id.contains_key("codex"));

    let serialized = serde_json::to_string(&limits).expect("rate limits serialize");
    assert!(!serialized.contains("accessToken"));
    assert!(!serialized.contains("refreshToken"));
    assert!(!serialized.contains("must-not-escape"));
    assert_eq!(
        client.transport().calls,
        vec![("account/rateLimits/read".into(), json!({}))]
    );
}

#[test]
fn logout_uses_managed_account_logout() {
    let transport = FakeTransport::with_responses([json!({})]);
    let mut client = CodexSubscriptionClient::new_initialized_for_test(transport);

    client.logout().expect("logout succeeds");

    assert_eq!(
        client.transport().calls,
        vec![("account/logout".into(), json!({}))]
    );
}
