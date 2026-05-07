# Tests

## Integration (dry-run)

Tests that exercise argument parsing, validation, help output, and shell
completions without requiring Podman, devcontainer CLI, or jq.

```bash
./tests/integration/smoke.sh
```

## E2E (requires container runtime)

Tests that build images, start containers, and verify the full lifecycle.
These require Podman (or Docker), devcontainer CLI (or npx), and jq.

```bash
# From the project root
./tests/e2e/run-test

# See available tests
./tests/e2e/run-test

# Run specific tests
./tests/e2e/run-test 1    # Fast path TUI
./tests/e2e/run-test 2    # Common utils feature path
./tests/e2e/run-test 4    # Merge collision test
./tests/e2e/run-test all  # Quick sanity suite (1, 2, 6)
```

The E2E test workspace lives in `tests/testdata/test-project/`. It is
auto-initialized with `git init` on first run (the `.git` directory is
`.gitignore`d). This ensures correct project scoping without polluting the
parent git repository.

See `tests/testdata/README.md` for full E2E test documentation including
verification commands and cleanup instructions.

## Pre-commit check (quick)

```bash
bash -n bin/opencode-container        # Syntax check
zsh test-completion.sh               # Completion tests
./tests/integration/smoke.sh         # Dry-run smoke tests
```
