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

See `e2e/` for future e2e test scripts. Currently manual testing is
documented in `../PLAN.md` section 8.

## Pre-commit check (quick)

```bash
bash -n bin/opencode-container        # Syntax check
zsh test-completion.sh               # Completion tests
./tests/integration/smoke.sh         # Dry-run smoke tests
```
