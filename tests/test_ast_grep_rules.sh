#!/usr/bin/env bash
set -euo pipefail

AST_GREP_BIN="${AST_GREP_BIN:-ast-grep}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

command -v "$AST_GREP_BIN" >/dev/null 2>&1 || {
  echo "ast-grep is required to run ast-grep rule regression tests" >&2
  exit 2
}

scan_rule() {
  local rule_path="$1"
  local scan_path="$2"
  local fixture_root="$3"
  local rule
  rule="$(cat "$ROOT/$rule_path")"
  (cd "$fixture_root" && "$AST_GREP_BIN" scan --inline-rules "$rule" "$scan_path" 2>&1 || true)
}

require_reported() {
  local output="$1"
  local needle="$2"
  grep -Fq "$needle" <<<"$output" || {
    echo "Expected ast-grep diagnostic was not reported: $needle" >&2
    printf '%s\n' "$output" >&2
    exit 1
  }
}

require_not_reported() {
  local output="$1"
  local needle="$2"
  if grep -Fq "$needle" <<<"$output"; then
    echo "Unexpected ast-grep diagnostic was reported: $needle" >&2
    printf '%s\n' "$output" >&2
    exit 1
  fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# Observer CLI: every stdlib process-spawning API covered by the policy must report.
mkdir -p "$TMP/observer/scripts"
cat >"$TMP/observer/scripts/autodev-cli.py" <<'PY'
import os
import subprocess

subprocess.run(["observer-run"])
subprocess.call(["observer-call"])
subprocess.Popen(["observer-popen"])
subprocess.check_call(["observer-check-call"])
subprocess.check_output(["observer-check-output"])
os.system("observer-system")
os.popen("observer-os-popen")
print("observer-safe")
PY
observer_output="$(scan_rule ".ast-grep/rules/observer-no-process-exec.yml" "scripts/autodev-cli.py" "$TMP/observer")"
for expected in \
  'subprocess.run(["observer-run"])' \
  'subprocess.call(["observer-call"])' \
  'subprocess.Popen(["observer-popen"])' \
  'subprocess.check_call(["observer-check-call"])' \
  'subprocess.check_output(["observer-check-output"])' \
  'os.system("observer-system")' \
  'os.popen("observer-os-popen")'; do
  require_reported "$observer_output" "$expected"
done
require_not_reported "$observer_output" 'print("observer-safe")'

# Python shell=True: detect direct, receiver-alias, imported-function alias, and
# first/middle/last keyword positions while avoiding unrelated shell parameters.
mkdir -p "$TMP/python/scripts"
cat >"$TMP/python/scripts/fixture.py" <<'PY'
import subprocess
import subprocess as sp
from subprocess import run as execute

subprocess.run(shell=True)
subprocess.run(["direct-middle"], shell=True, check=True)
subprocess.run(["direct-last"], check=True, shell=True)
sp.run(["alias-receiver"], shell=True)
execute(["alias-function"], shell=True)
subprocess.run(["safe-shell-false"], shell=False)
unrelated("safe-unrelated", shell=True)
PY
python_output="$(scan_rule ".ast-grep/rules/python-no-shell-true.yml" "scripts/fixture.py" "$TMP/python")"
for expected in \
  'subprocess.run(shell=True)' \
  'subprocess.run(["direct-middle"], shell=True, check=True)' \
  'subprocess.run(["direct-last"], check=True, shell=True)' \
  'sp.run(["alias-receiver"], shell=True)' \
  'execute(["alias-function"], shell=True)'; do
  require_reported "$python_output" "$expected"
done
require_not_reported "$python_output" 'safe-shell-false'
require_not_reported "$python_output" 'safe-unrelated'

# ForgeCore: both fully-qualified and imported Command::new spellings report.
mkdir -p "$TMP/rust/crates/forge-core/src"
cat >"$TMP/rust/crates/forge-core/src/qualified.rs" <<'RS'
fn qualified() {
    let _ = std::process::Command::new("qualified-git").output();
}
RS
cat >"$TMP/rust/crates/forge-core/src/imported.rs" <<'RS'
use std::process::Command;
fn imported() {
    let _ = Command::new("imported-git").output();
}
RS
cat >"$TMP/rust/crates/forge-core/src/safe.rs" <<'RS'
struct Command;
impl Command {
    fn new(_: &str) -> Self { Self }
}
fn safe() {
    let _ = Command::new("local-command-type");
}
RS
rust_output="$(scan_rule ".ast-grep/rules/rust-no-direct-process-command.yml" "crates/forge-core/src" "$TMP/rust")"
require_reported "$rust_output" 'std::process::Command::new("qualified-git")'
require_reported "$rust_output" 'Command::new("imported-git")'
require_not_reported "$rust_output" 'local-command-type'

# PR trust-boundary declaration: exactly one top-level state, with categories
# nested under the impact-present state.
grep -Fq 'Select exactly one top-level declaration:' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq -- '- [ ] No trusted execution or authorization changes' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq -- '- [ ] Trusted-boundary impact present' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq -- '  - [ ] ForgeCore / Workspace / AuthorizationGrant changed' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"

echo "ast-grep rule regression tests: PASS"
