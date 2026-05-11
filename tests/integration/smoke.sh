#!/bin/bash
# SPDX-License-Identifier: Apache-2.0
set -uo pipefail

# Smoke tests for opencode-container (dry-run, no Podman/devcontainer required)
# These tests exercise argument parsing, validation, help output, and shell
# completions without starting any containers.

SCRIPT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
OC="${SCRIPT_DIR}/bin/opencode-container"
PASS=0
FAIL=0

run_test() {
  local desc="$1"
  shift
  local expected="$1"
  shift
  local output
  local status=0

  output="$($@ 2>&1)" || status=$?

  # Use -e to pass pattern safely (handles leading dashes)
  if echo "$output" | grep -qF -e "$expected"; then
    echo "PASS  $desc"
    ((PASS++))
  else
    echo "FAIL  $desc"
    echo "       expected match: $expected"
    echo "       output:"
    echo "$output" | sed 's/^/         /'
    ((FAIL++))
  fi
}

echo "=== opencode-container dry-run smoke tests ==="
echo ""

# --- Help and usage ---
run_test "--help shows usage" "Commands:" "$OC" --help
run_test "-h shows usage" "Commands:" "$OC" -h

# --- Global flag validation ---
run_test "missing --feature-file arg errors" "'--feature-file <PATH>' but none was supplied" "$OC" --feature-file
run_test "missing --env-file arg errors" "'--env-file <PATH>' but none was supplied" "$OC" --env-file
run_test "missing --env arg errors" "'--env <VAR=value>' but none was supplied" "$OC" --env
run_test "missing --local-env arg errors" "'--local-env <VAR>' but none was supplied" "$OC" --local-env
run_test "unknown option errors" "unexpected argument '--unknown'" "$OC" --unknown

# --- Fast path (no --feature-file) reaches container runtime ---
# These fail at a container/TTY error which confirms parsing succeeded
run_test "default path reaches container runtime" "cannot attach stdin to a TTY" "$OC"

# --- Feature path (reaches devcontainer check) ---
# Create a temp feature file; the script will validate JSON, then fail at devcontainer up
# because podman is not available in this test environment.
TMP_JSON="$(mktemp)"
echo '{"features":{"ghcr.io/devcontainers/features/common-utils:2":{}}}' > "$TMP_JSON"
run_test "--feature-file reaches devcontainer up (no podman)" "devcontainer up failed" "$OC" --feature-file "$TMP_JSON"
rm -f "$TMP_JSON"

# --- Completion scripts read from files ---
BASH_COMP="$($OC completion --bash)"
ZSH_COMP="$($OC completion --zsh)"

run_test "bash completion has --feature-file" "--feature-file" echo "$BASH_COMP"
run_test "zsh completion has --feature-file" "--feature-file" echo "$ZSH_COMP"
run_test "bash completion file exists" "complete -F" echo "$BASH_COMP"
run_test "zsh completion file exists" "compdef" echo "$ZSH_COMP"

# --- Web mode passthrough ---
# Web mode fails at container start (different error depending on environment)
run_test "web mode reaches container runtime" "Container failed to start" "$OC" run web
run_test "web mode custom port reaches runtime" "Container failed to start" "$OC" run web --port 5000
run_test "web mode custom hostname reaches runtime" "Container failed to start" "$OC" run web --hostname 127.0.0.1

# --- Legacy -- passthrough ---
run_test "legacy -- web reaches container runtime" "Container failed to start" "$OC" -- web
run_test "legacy -b -- web reaches container runtime" "Container failed to start" "$OC" -b -- web

# --- Environment variable flags ---
run_test "-e/--env reaches container runtime" "cannot attach stdin to a TTY" "$OC" -e FOO=bar
run_test "--env reaches container runtime" "cannot attach stdin to a TTY" "$OC" --env FOO=bar
run_test "--local-env reaches container runtime" "cannot attach stdin to a TTY" "$OC" --local-env HOME

# --- Argument passthrough ---
run_test "run passes args to opencode" "cannot attach stdin to a TTY" "$OC" run --some-opencode-flag
# --help after -- should pass through to opencode (clap handles bare --help)
run_test "--help after -- passes through" "cannot attach stdin to a TTY" "$OC" -- --help

# --- Completion mutual exclusion ---
run_test "completion --bash --zsh is rejected" "cannot be used with" "$OC" completion --bash --zsh

echo ""
echo "Results: $PASS passed, $FAIL failed"
exit $((FAIL > 0 ? 1 : 0))
