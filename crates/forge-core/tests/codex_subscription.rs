use std::collections::VecDeque;

use forge_core::{
    CodexLoginStart, CodexRpcTransport, CodexSubscriptionClient, CodexSubscriptionError,
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
    fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Value, CodexSubscriptionError> {
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
