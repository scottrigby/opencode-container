#!/bin/sh
set -e

cd /code

# opencode treats non-git directories as a single global project rooted at "/",
# which breaks per-directory isolation in the web UI (sessions get directory="/",
# sidebar navigates to Lw, etc.). Initialise an empty git repo only when /code
# is not already inside one. This is harmless and reversible (rm -rf .git).
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  git init >/dev/null 2>&1 || true
fi

exec opencode "$@"
