#!/usr/bin/env bash
set -euo pipefail

AST_GREP_BIN="${AST_GREP_BIN:-ast-grep}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/crates/forge-core/src"
cat >"$TMP/crates/forge-core/src/nested.rs" <<'RS'
use std::{process::Command};
fn direct() {
    let _ = Command::new("nested-direct").output();
}
RS
cat >"$TMP/crates/forge-core/src/nested_alias.rs" <<'RS'
use std::{process::Command as ProcessCommand};
fn renamed() {
    let _ = ProcessCommand::new("nested-alias").output();
}
RS

rule="$(cat "$ROOT/.ast-grep/rules/rust-no-direct-process-command.yml")"
output="$(cd "$TMP" && "$AST_GREP_BIN" scan --inline-rules "$rule" crates/forge-core/src 2>&1 || true)"
grep -Fq 'Command::new("nested-direct")' <<<"$output"
grep -Fq 'ProcessCommand::new("nested-alias")' <<<"$output"

grep -Eq 'types:.*edited|-[[:space:]]*edited' "$ROOT/.github/workflows/ci.yml" || {
  echo "pull_request edited activity is not configured" >&2
  exit 1
}

echo "CodeRabbit follow-up regressions: PASS"
