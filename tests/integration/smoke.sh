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
run_test "--help shows usage" "NAME" "$OC" --help
run_test "-h shows usage" "SYNOPSIS" "$OC" -h

# --- Global flag validation ---
run_test "missing --feature-file arg errors" "--feature-file requires an argument" "$OC" --feature-file
run_test "unknown option errors" "Unknown option" "$OC" --unknown

# --- Fast path (no --feature-file) reaches podman check ---
# These fail at "podman: command not found" which confirms parsing succeeded
run_test "default path reaches podman" "podman: command not found" "$OC"

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

# --- Web mode port validation ---
run_test "web mode with port reaches podman" "podman: command not found" "$OC" -w -p 5000

echo ""
echo "Results: $PASS passed, $FAIL failed"
exit $((FAIL > 0 ? 1 : 0))
