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

# Observer CLI: dotted, receiver-alias, and directly imported process APIs report.
mkdir -p "$TMP/observer/scripts"
cat >"$TMP/observer/scripts/autodev-cli.py" <<'PY'
import os
import os as operating_system
import subprocess
import subprocess as sp
from os import popen as open_pipe
from os import system
from subprocess import check_call
from subprocess import check_output as execute
from subprocess import run as execute_run

subprocess.run(["observer-run"])
subprocess.call(["observer-call"])
subprocess.Popen(["observer-popen"])
subprocess.check_call(["observer-check-call"])
subprocess.check_output(["observer-check-output"])
os.system("observer-system")
os.popen("observer-os-popen")
sp.run(["observer-alias-run"])
sp.check_call(["observer-alias-check-call"])
operating_system.system("observer-alias-system")
execute(["observer-imported-check-output"])
check_call(["observer-imported-check-call"])
system("observer-imported-system")
open_pipe("observer-imported-popen")
execute_run(["observer-imported-run"])
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
  'os.popen("observer-os-popen")' \
  'sp.run(["observer-alias-run"])' \
  'sp.check_call(["observer-alias-check-call"])' \
  'operating_system.system("observer-alias-system")' \
  'execute(["observer-imported-check-output"])' \
  'check_call(["observer-imported-check-call"])' \
  'system("observer-imported-system")' \
  'open_pipe("observer-imported-popen")' \
  'execute_run(["observer-imported-run"])'; do
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

# ForgeCore process creation: qualified, direct import, grouped import, and renamed
# import bindings report without flagging an unrelated local Command type.
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
cat >"$TMP/rust/crates/forge-core/src/grouped.rs" <<'RS'
use std::process::{Command};
fn grouped() {
    let _ = Command::new("grouped-git").output();
}
RS
cat >"$TMP/rust/crates/forge-core/src/renamed.rs" <<'RS'
use std::process::Command as ProcessCommand;
fn renamed() {
    let _ = ProcessCommand::new("renamed-git").output();
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
require_reported "$rust_output" 'Command::new("grouped-git")'
require_reported "$rust_output" 'ProcessCommand::new("renamed-git")'
require_not_reported "$rust_output" 'local-command-type'

# ForgeCore unchecked-result rule: unwrap/expect report, safe propagation does not.
cat >"$TMP/rust/crates/forge-core/src/unchecked.rs" <<'RS'
fn unchecked(result: Result<u8, &'static str>) {
    let _ = result.unwrap();
    let _ = result.expect("forgecore-expect");
}
RS
cat >"$TMP/rust/crates/forge-core/src/propagated.rs" <<'RS'
fn propagated(result: Result<u8, &'static str>) -> Result<u8, &'static str> {
    let value = result?;
    Ok(value)
}
RS
unwrap_output="$(scan_rule ".ast-grep/rules/forgecore-no-unwrap.yml" "crates/forge-core/src" "$TMP/rust")"
require_reported "$unwrap_output" 'result.unwrap()'
require_reported "$unwrap_output" 'result.expect("forgecore-expect")'
require_not_reported "$unwrap_output" 'let value = result?'

# PR trust-boundary declaration: exactly one top-level state, with categories
# nested under the impact-present state.
grep -Fq 'Select exactly one top-level declaration:' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq -- '- [ ] No trusted execution or authorization changes' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq -- '- [ ] Trusted-boundary impact present' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq -- '  - [ ] ForgeCore / Workspace / AuthorizationGrant changed' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"

# Template-shape coverage only. Submitted PR evidence is enforced separately by
# scripts/validate_pr_evidence.py in the pull_request workflow.
grep -Fq '### Commands run' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq '### Evidence' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq 'bash tests/test_ast_grep_rules.sh' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq 'CI run / artifact / output:' "$ROOT/.github/PULL_REQUEST_TEMPLATE.md"
grep -Fq 'python scripts/validate_pr_evidence.py "$GITHUB_EVENT_PATH"' "$ROOT/.github/workflows/ci.yml"

# ast-grep installation must be integrity-backed; global ad-hoc npm installation
# is forbidden. The GREEN implementation may use a lockfile or a verified release.
if grep -Fq 'npm install --global @ast-grep/cli' "$ROOT/.github/workflows/ci.yml"; then
  echo "CI still uses ad-hoc global ast-grep installation" >&2
  exit 1
fi
if ! grep -Eq 'npm ci|sha256sum .*check|sha256sum -c|sha256sum --check' "$ROOT/.github/workflows/ci.yml"; then
  echo "CI must use a lock-backed or checksum-verified ast-grep installation" >&2
  exit 1
fi

echo "ast-grep rule regression tests: PASS"
