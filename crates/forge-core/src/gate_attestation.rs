//! Kernel-signed attestations for client-side policy gates (ADR-007).
//!
//! GitHubOpsGem hooks run outside the trusted boundary (Claude Code in Termux,
//! CI runners). They compute gate facts locally, but their verdicts are only
//! claims. This module turns a claim into kernel-owned evidence:
//!
//! * the kernel normalizes the target ref and applies its own protected-ref
//!   check, which a client cannot remove or downgrade;
//! * the kernel derives the decision (`PASS` / `WARN` / `BLOCK`) itself — a
//!   request that tries to supply one is rejected at deserialization;
//! * the result is signed with an HMAC-SHA256 key held only by the trusted
//!   process. The key never leaves it, so a model or hook cannot mint one.
//!
//! An attestation is **evidence, not authority**. It never becomes an
//! [`crate::policy::AuthorizationGrant`] and cannot discharge a
//! `RequireApproval` decision; human approval stays the only way to do that.
//!
//! This module performs no I/O. Callers supply the key and the clock.

use chrono::{DateTime, SecondsFormat, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::evidence::sha256_hex;

type HmacSha256 = Hmac<Sha256>;

/// Wire version of the signed body. Bump on any change to [`SignedBody`].
pub const GATE_ATTESTATION_VERSION: &str = "gem-gate/1";
/// Minimum signing-key length. Shorter keys are rejected at construction.
pub const MIN_GATE_KEY_BYTES: usize = 32;
/// Upper bound on client-reported violations per request.
pub const MAX_GATE_VIOLATIONS: usize = 64;
/// Upper bound on any single text field, in characters.
pub const MAX_GATE_TEXT_CHARS: usize = 512;
/// Upper bound on the serialized `facts` object, in bytes.
pub const MAX_GATE_FACTS_BYTES: usize = 64 * 1024;

/// Tools that write directly to a branch. A protected target for any of these
/// is a kernel violation regardless of what the client reports.
const BRANCH_WRITE_TOOLS: &[&str] = &[
    "push_files",
    "create_or_update_file",
    "delete_file",
    "git_push",
    "git_force_push",
];

/// Enforcement mode requested by the client for its own (heuristic) rules.
/// Kernel violations always block, whatever the mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateMode {
    Warn,
    Enforce,
}

/// Kernel-derived gate decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateDecision {
    Pass,
    Warn,
    Block,
}

/// An untrusted claim from a gate client. Unknown fields (for example a
/// client-supplied `decision`) are rejected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateAttestationRequest {
    pub repository: String,
    pub tool: String,
    pub target_ref: String,
    pub mode: GateMode,
    pub session_id: String,
    #[serde(default)]
    pub violations: Vec<String>,
    /// Lowercase hex SHA-256 of the outbound write payload, when there is one.
    #[serde(default)]
    pub payload_sha256: Option<String>,
    /// Opaque client metrics (debt scores, ABI receipts). Hashed, not stored.
    #[serde(default)]
    pub facts: serde_json::Value,
}

/// Kernel-owned, signed gate evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateAttestation {
    pub version: String,
    pub attestation_id: String,
    pub repository: String,
    pub tool: String,
    /// Normalized ref (`refs/heads/main` → `main`).
    pub target_ref: String,
    pub mode: GateMode,
    pub decision: GateDecision,
    pub client_violations: Vec<String>,
    pub kernel_violations: Vec<String>,
    pub payload_sha256: Option<String>,
    pub facts_sha256: String,
    pub session_id: String,
    /// RFC 3339, second precision, UTC.
    pub issued_at: String,
    pub key_id: String,
    /// Lowercase hex HMAC-SHA256 over the canonical signed body.
    pub signature: String,
}

/// Errors raised while building or checking an attestation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GateAttestationError {
    #[error("gate signing key must be at least {min} bytes")]
    KeyTooShort { min: usize },
    #[error("gate request field `{0}` is required")]
    MissingField(&'static str),
    #[error("gate request field `{0}` exceeds {MAX_GATE_TEXT_CHARS} characters")]
    FieldTooLong(&'static str),
    #[error("gate request carries more than {MAX_GATE_VIOLATIONS} violations")]
    TooManyViolations,
    #[error("gate request facts exceed {MAX_GATE_FACTS_BYTES} bytes")]
    FactsTooLarge,
    #[error("payload_sha256 must be 64 lowercase hex characters")]
    InvalidPayloadDigest,
}

/// The HMAC key held by the trusted process. `Debug` never prints the secret.
#[derive(Clone)]
pub struct GateSigningKey {
    secret: Vec<u8>,
    key_id: String,
}

impl GateSigningKey {
    /// Build a key, rejecting anything shorter than [`MIN_GATE_KEY_BYTES`].
    pub fn new(secret: impl AsRef<[u8]>) -> Result<Self, GateAttestationError> {
        let secret = secret.as_ref().to_vec();
        if secret.len() < MIN_GATE_KEY_BYTES {
            return Err(GateAttestationError::KeyTooShort {
                min: MIN_GATE_KEY_BYTES,
            });
        }
        let mut id_input = b"gem-gate-key-id:".to_vec();
        id_input.extend_from_slice(&secret);
        let key_id = sha256_hex(&id_input)[..16].to_string();
        Ok(Self { secret, key_id })
    }

    /// A public, non-reversible identifier for the key.
    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    fn mac(&self) -> HmacSha256 {
        HmacSha256::new_from_slice(&self.secret).expect("HMAC accepts keys of any length")
    }
}

impl std::fmt::Debug for GateSigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GateSigningKey")
            .field("key_id", &self.key_id)
            .field("secret", &"<redacted>")
            .finish()
    }
}

#[derive(Serialize)]
struct SignedBody<'a> {
    version: &'a str,
    repository: &'a str,
    tool: &'a str,
    target_ref: &'a str,
    mode: GateMode,
    decision: GateDecision,
    client_violations: &'a [String],
    kernel_violations: &'a [String],
    payload_sha256: Option<&'a str>,
    facts_sha256: &'a str,
    session_id: &'a str,
    issued_at: &'a str,
    key_id: &'a str,
}

impl<'a> SignedBody<'a> {
    fn of(att: &'a GateAttestation) -> Self {
        Self {
            version: &att.version,
            repository: &att.repository,
            tool: &att.tool,
            target_ref: &att.target_ref,
            mode: att.mode,
            decision: att.decision,
            client_violations: &att.client_violations,
            kernel_violations: &att.kernel_violations,
            payload_sha256: att.payload_sha256.as_deref(),
            facts_sha256: &att.facts_sha256,
            session_id: &att.session_id,
            issued_at: &att.issued_at,
            key_id: &att.key_id,
        }
    }

    fn bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("signed body is plain data")
    }
}

/// Strip remote/ref prefixes so `refs/heads/main`, `origin/main` and `main`
/// are treated alike.
pub fn normalize_ref(target: &str) -> String {
    let mut name = target.trim();
    if let Some(rest) = name.strip_prefix("refs/heads/") {
        name = rest;
    } else if let Some(rest) = name.strip_prefix("refs/remotes/") {
        name = rest.split_once('/').map_or(rest, |(_, branch)| branch);
    } else if let Some(rest) = name.strip_prefix("heads/") {
        name = rest;
    }
    for remote in ["origin/", "upstream/"] {
        if let Some(rest) = name.strip_prefix(remote) {
            name = rest;
            break;
        }
    }
    name.to_string()
}

/// Whether a normalized ref is protected from direct writes.
pub fn is_protected_ref(normalized: &str) -> bool {
    let name = normalized.to_ascii_lowercase();
    matches!(
        name.as_str(),
        "main" | "master" | "production" | "prod" | "trunk" | "release" | "hotfix"
    ) || ["release/", "release-", "hotfix/", "hotfix-"]
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn required(value: &str, field: &'static str) -> Result<String, GateAttestationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(GateAttestationError::MissingField(field));
    }
    if trimmed.chars().count() > MAX_GATE_TEXT_CHARS {
        return Err(GateAttestationError::FieldTooLong(field));
    }
    Ok(trimmed.to_string())
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(value.get(i..i + 2)?, 16).ok())
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Validate a client claim, apply kernel checks, derive the decision and sign.
pub fn attest(
    request: &GateAttestationRequest,
    key: &GateSigningKey,
    now: DateTime<Utc>,
) -> Result<GateAttestation, GateAttestationError> {
    let repository = required(&request.repository, "repository")?;
    let tool = required(&request.tool, "tool")?;
    let target_ref = normalize_ref(&required(&request.target_ref, "target_ref")?);
    let session_id = required(&request.session_id, "session_id")?;

    if request.violations.len() > MAX_GATE_VIOLATIONS {
        return Err(GateAttestationError::TooManyViolations);
    }
    let mut client_violations = Vec::with_capacity(request.violations.len());
    for violation in &request.violations {
        let text = violation.trim();
        if text.is_empty() {
            continue;
        }
        if text.chars().count() > MAX_GATE_TEXT_CHARS {
            return Err(GateAttestationError::FieldTooLong("violations"));
        }
        client_violations.push(text.to_string());
    }

    let payload_sha256 = match request.payload_sha256.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(digest) if is_sha256_hex(digest) => Some(digest.to_string()),
        Some(_) => return Err(GateAttestationError::InvalidPayloadDigest),
    };

    let facts = serde_json::to_vec(&request.facts).expect("facts are JSON");
    if facts.len() > MAX_GATE_FACTS_BYTES {
        return Err(GateAttestationError::FactsTooLarge);
    }

    let mut kernel_violations = Vec::new();
    if BRANCH_WRITE_TOOLS.contains(&tool.to_ascii_lowercase().as_str())
        && is_protected_ref(&target_ref)
    {
        kernel_violations.push(format!(
            "kernel: direct write to protected ref '{target_ref}' is forbidden"
        ));
    }

    let decision = if !kernel_violations.is_empty() {
        GateDecision::Block
    } else if client_violations.is_empty() {
        GateDecision::Pass
    } else {
        match request.mode {
            GateMode::Warn => GateDecision::Warn,
            GateMode::Enforce => GateDecision::Block,
        }
    };

    let mut attestation = GateAttestation {
        version: GATE_ATTESTATION_VERSION.to_string(),
        attestation_id: String::new(),
        repository,
        tool,
        target_ref,
        mode: request.mode,
        decision,
        client_violations,
        kernel_violations,
        payload_sha256,
        facts_sha256: sha256_hex(&facts),
        session_id,
        issued_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        key_id: key.key_id().to_string(),
        signature: String::new(),
    };
    let body = SignedBody::of(&attestation).bytes();
    attestation.attestation_id = sha256_hex(&body)[..32].to_string();
    let mut mac = key.mac();
    mac.update(&body);
    attestation.signature = encode_hex(&mac.finalize().into_bytes());
    Ok(attestation)
}

/// [`attest`] with the current UTC time, for callers without their own clock.
pub fn attest_now(
    request: &GateAttestationRequest,
    key: &GateSigningKey,
) -> Result<GateAttestation, GateAttestationError> {
    attest(request, key, Utc::now())
}

/// Check an attestation against the key. Any edited field, a foreign key, or a
/// different wire version fails. Comparison is constant time.
pub fn verify_attestation(attestation: &GateAttestation, key: &GateSigningKey) -> bool {
    if attestation.version != GATE_ATTESTATION_VERSION || attestation.key_id != key.key_id() {
        return false;
    }
    let body = SignedBody::of(attestation).bytes();
    if sha256_hex(&body)[..32] != attestation.attestation_id {
        return false;
    }
    let Some(signature) = decode_hex(&attestation.signature) else {
        return false;
    };
    let mut mac = key.mac();
    mac.update(&body);
    mac.verify_slice(&signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn key() -> GateSigningKey {
        GateSigningKey::new([7u8; 32]).expect("32-byte key")
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 7, 17, 30, 0).unwrap()
    }

    fn request(
        tool: &str,
        target: &str,
        mode: GateMode,
        violations: &[&str],
    ) -> GateAttestationRequest {
        GateAttestationRequest {
            repository: "asshat1981ar/autodev".into(),
            tool: tool.into(),
            target_ref: target.into(),
            mode,
            session_id: "session-1".into(),
            violations: violations.iter().map(|v| v.to_string()).collect(),
            payload_sha256: None,
            facts: serde_json::json!({"psc": 0.0, "dur": 0.0}),
        }
    }

    #[test]
    fn clean_request_passes_and_verifies() {
        let att = attest(
            &request("create_pull_request", "feat/x", GateMode::Enforce, &[]),
            &key(),
            now(),
        )
        .unwrap();
        assert_eq!(att.decision, GateDecision::Pass);
        assert_eq!(att.issued_at, "2026-11-07T17:30:00Z");
        assert!(verify_attestation(&att, &key()));
    }

    #[test]
    fn kernel_blocks_protected_push_even_when_client_reports_nothing_in_warn_mode() {
        for target in [
            "main",
            "refs/heads/main",
            "origin/master",
            "release/1.2",
            "refs/remotes/upstream/trunk",
        ] {
            let att = attest(
                &request("push_files", target, GateMode::Warn, &[]),
                &key(),
                now(),
            )
            .unwrap();
            assert_eq!(att.decision, GateDecision::Block, "{target}");
            assert_eq!(att.kernel_violations.len(), 1, "{target}");
        }
    }

    #[test]
    fn feature_branches_are_not_protected() {
        for target in ["feat/main-menu", "refs/heads/fix/12-crash", "mainline"] {
            let att = attest(
                &request("push_files", target, GateMode::Enforce, &[]),
                &key(),
                now(),
            )
            .unwrap();
            assert_eq!(att.decision, GateDecision::Pass, "{target}");
        }
    }

    #[test]
    fn client_violations_follow_mode() {
        let warn = attest(
            &request("merge_pull_request", "feat/x", GateMode::Warn, &["PSC=0.6"]),
            &key(),
            now(),
        )
        .unwrap();
        let enforce = attest(
            &request(
                "merge_pull_request",
                "feat/x",
                GateMode::Enforce,
                &["PSC=0.6"],
            ),
            &key(),
            now(),
        )
        .unwrap();
        assert_eq!(warn.decision, GateDecision::Warn);
        assert_eq!(enforce.decision, GateDecision::Block);
    }

    #[test]
    fn client_cannot_supply_a_decision() {
        let raw = r#"{"repository":"r","tool":"push_files","target_ref":"main","mode":"warn","session_id":"s","decision":"PASS"}"#;
        assert!(serde_json::from_str::<GateAttestationRequest>(raw).is_err());
    }

    #[test]
    fn tampering_with_any_signed_field_fails_verification() {
        let att = attest(
            &request("merge_pull_request", "feat/x", GateMode::Enforce, &["x"]),
            &key(),
            now(),
        )
        .unwrap();
        let mut flipped = att.clone();
        flipped.decision = GateDecision::Pass;
        assert!(!verify_attestation(&flipped, &key()));
        let mut cleared = att.clone();
        cleared.client_violations.clear();
        assert!(!verify_attestation(&cleared, &key()));
        let mut retargeted = att.clone();
        retargeted.target_ref = "feat/y".into();
        assert!(!verify_attestation(&retargeted, &key()));
        let mut forged = att;
        forged.signature = "00".repeat(32);
        assert!(!verify_attestation(&forged, &key()));
    }

    #[test]
    fn other_keys_and_versions_fail_verification() {
        let att = attest(
            &request("create_pull_request", "feat/x", GateMode::Enforce, &[]),
            &key(),
            now(),
        )
        .unwrap();
        let other = GateSigningKey::new([9u8; 32]).unwrap();
        assert!(!verify_attestation(&att, &other));
        let mut old = att;
        old.version = "gem-gate/0".into();
        assert!(!verify_attestation(&old, &key()));
    }

    #[test]
    fn short_keys_are_rejected_and_debug_redacts_secret() {
        assert_eq!(
            GateSigningKey::new([1u8; 16]).unwrap_err(),
            GateAttestationError::KeyTooShort {
                min: MIN_GATE_KEY_BYTES
            }
        );
        let printed = format!("{:?}", key());
        assert!(printed.contains("<redacted>") && !printed.contains("7, 7"));
    }

    #[test]
    fn request_limits_fail_closed() {
        let mut blank = request("push_files", "feat/x", GateMode::Enforce, &[]);
        blank.session_id = "  ".into();
        assert_eq!(
            attest(&blank, &key(), now()).unwrap_err(),
            GateAttestationError::MissingField("session_id")
        );

        let many: Vec<String> = (0..=MAX_GATE_VIOLATIONS).map(|i| format!("v{i}")).collect();
        let mut flood = request("push_files", "feat/x", GateMode::Enforce, &[]);
        flood.violations = many;
        assert_eq!(
            attest(&flood, &key(), now()).unwrap_err(),
            GateAttestationError::TooManyViolations
        );

        let mut digest = request("push_files", "feat/x", GateMode::Enforce, &[]);
        digest.payload_sha256 = Some("NOT-HEX".into());
        assert_eq!(
            attest(&digest, &key(), now()).unwrap_err(),
            GateAttestationError::InvalidPayloadDigest
        );

        let mut big = request("push_files", "feat/x", GateMode::Enforce, &[]);
        big.facts = serde_json::json!({"blob": "x".repeat(MAX_GATE_FACTS_BYTES)});
        assert_eq!(
            attest(&big, &key(), now()).unwrap_err(),
            GateAttestationError::FactsTooLarge
        );
    }

    #[test]
    fn json_round_trip_keeps_signature_valid() {
        let att = attest(
            &request("merge_pull_request", "feat/x", GateMode::Warn, &["a"]),
            &key(),
            now(),
        )
        .unwrap();
        let wire = serde_json::to_string(&att).unwrap();
        let back: GateAttestation = serde_json::from_str(&wire).unwrap();
        assert!(verify_attestation(&back, &key()));
    }
}
