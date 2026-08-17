use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use autodev_server::{router, AppState, CodexAccountService};
use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    response::Response,
};
use forge_core::{
    CodexAccount, CodexCredits, CodexLoginStart, CodexRateLimitSnapshot, CodexRateLimitWindow,
    CodexRateLimits, CodexSubscriptionError,
};
use serde_json::{json, Value};
use tower::ServiceExt;

#[derive(Clone)]
struct FakeCodexService {
    logout_called: Arc<AtomicBool>,
}

impl CodexAccountService for FakeCodexService {
    fn account(&self) -> Result<CodexAccount, CodexSubscriptionError> {
        Ok(CodexAccount {
            authenticated: true,
            auth_mode: Some("chatgpt".into()),
            plan_type: Some("plus".into()),
        })
    }

    fn start_browser_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        Ok(CodexLoginStart::Browser {
            login_id: "browser-login".into(),
            auth_url: "https://chatgpt.com/auth".into(),
        })
    }

    fn start_device_code_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        Ok(CodexLoginStart::DeviceCode {
            login_id: "device-login".into(),
            verification_url: "https://auth.openai.com/codex/device".into(),
            user_code: "ABCD-1234".into(),
        })
    }

    fn rate_limits(&self) -> Result<CodexRateLimits, CodexSubscriptionError> {
        Ok(CodexRateLimits {
            default: CodexRateLimitSnapshot {
                limit_id: Some("codex".into()),
                limit_name: Some("Codex".into()),
                primary: Some(CodexRateLimitWindow {
                    used_percent: 42,
                    window_duration_mins: Some(300),
                    resets_at: Some(1_800_000_000),
                }),
                secondary: None,
                credits: Some(CodexCredits {
                    has_credits: true,
                    unlimited: false,
                    balance: Some("7.50".into()),
                }),
                plan_type: Some("plus".into()),
                reached_type: None,
            },
            by_limit_id: BTreeMap::new(),
            reset_credits_available: Some(2),
        })
    }

    fn logout(&self) -> Result<(), CodexSubscriptionError> {
        self.logout_called.store(true, Ordering::SeqCst);
        Ok(())
    }
}

struct SensitiveFailureService;

impl CodexAccountService for SensitiveFailureService {
    fn account(&self) -> Result<CodexAccount, CodexSubscriptionError> {
        Err(CodexSubscriptionError::Protocol(
            "refreshToken=must-not-escape".into(),
        ))
    }

    fn start_browser_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.account()?;
        unreachable!()
    }

    fn start_device_code_login(&self) -> Result<CodexLoginStart, CodexSubscriptionError> {
        self.account()?;
        unreachable!()
    }

    fn rate_limits(&self) -> Result<CodexRateLimits, CodexSubscriptionError> {
        self.account()?;
        unreachable!()
    }

    fn logout(&self) -> Result<(), CodexSubscriptionError> {
        self.account()?;
        unreachable!()
    }
}

async fn body_json(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    serde_json::from_slice(&bytes).expect("JSON response")
}

async fn call(app: axum::Router, method: &str, uri: &str) -> Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .expect("request"),
    )
    .await
    .expect("response")
}

#[tokio::test]
async fn codex_account_and_login_routes_expose_only_safe_metadata() {
    let logout_called = Arc::new(AtomicBool::new(false));
    let service = Arc::new(FakeCodexService {
        logout_called: logout_called.clone(),
    });
    let app = router(AppState::with_codex_service(None, service));

    let account = call(app.clone(), "GET", "/api/v1/codex/account").await;
    assert_eq!(account.status(), StatusCode::OK);
    let account_json = body_json(account).await;
    assert_eq!(
        account_json,
        json!({
            "authenticated": true,
            "auth_mode": "chatgpt",
            "plan_type": "plus"
        })
    );
    assert!(!account_json.to_string().to_lowercase().contains("token"));

    let browser = call(app.clone(), "POST", "/api/v1/codex/login/browser").await;
    assert_eq!(browser.status(), StatusCode::OK);
    assert_eq!(
        body_json(browser).await,
        json!({
            "type": "browser",
            "login_id": "browser-login",
            "auth_url": "https://chatgpt.com/auth"
        })
    );

    let device = call(app, "POST", "/api/v1/codex/login/device-code").await;
    assert_eq!(device.status(), StatusCode::OK);
    assert_eq!(
        body_json(device).await,
        json!({
            "type": "device_code",
            "login_id": "device-login",
            "verification_url": "https://auth.openai.com/codex/device",
            "user_code": "ABCD-1234"
        })
    );
}

#[tokio::test]
async fn codex_rate_limits_and_logout_are_safe_control_plane_operations() {
    let logout_called = Arc::new(AtomicBool::new(false));
    let service = Arc::new(FakeCodexService {
        logout_called: logout_called.clone(),
    });
    let app = router(AppState::with_codex_service(None, service));

    let limits = call(app.clone(), "GET", "/api/v1/codex/rate-limits").await;
    assert_eq!(limits.status(), StatusCode::OK);
    let limits_json = body_json(limits).await;
    assert_eq!(limits_json["default"]["limit_id"], "codex");
    assert_eq!(limits_json["default"]["primary"]["used_percent"], 42);
    assert_eq!(limits_json["reset_credits_available"], 2);
    assert!(!limits_json.to_string().to_lowercase().contains("token"));

    let logout = call(app, "POST", "/api/v1/codex/logout").await;
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);
    assert!(logout_called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn codex_unavailable_and_protocol_errors_fail_closed_without_leaking_details() {
    let unavailable = router(AppState::new(None));
    let unavailable_response = call(unavailable, "GET", "/api/v1/codex/account").await;
    assert_eq!(unavailable_response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        body_json(unavailable_response).await,
        json!({"error": "codex_provider_unavailable"})
    );

    let sensitive = router(AppState::with_codex_service(
        None,
        Arc::new(SensitiveFailureService),
    ));
    let protocol_response = call(sensitive, "GET", "/api/v1/codex/account").await;
    assert_eq!(protocol_response.status(), StatusCode::BAD_GATEWAY);
    let protocol_json = body_json(protocol_response).await;
    assert_eq!(
        protocol_json,
        json!({"error": "codex_provider_protocol_error"})
    );
    assert!(!protocol_json.to_string().contains("must-not-escape"));
}
