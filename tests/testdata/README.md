# E2E Test Data and Reference

These files support end-to-end testing of the `--feature-file` flag and
devcontainer feature integration. The E2E test runner is at
`tests/e2e/run-test` and uses `tests/testdata/test-project/` as its workspace.

## Quick Start

```bash
# From the project root
cd /path/to/opencode-container

# See available tests
./tests/e2e/run-test

# Run a specific test
./tests/e2e/run-test 1    # Fast path TUI
./tests/e2e/run-test 2    # Common utils feature path
./tests/e2e/run-test 4    # Merge collision test
./tests/e2e/run-test all  # Quick sanity suite
```

The E2E runner auto-initializes `tests/testdata/test-project/` with `git init`
on first run (the `.git` directory is `.gitignore`d). This ensures correct
project scoping without polluting the parent repository.

## Manual Invocation (advanced)

If you prefer to run tests manually from a custom workspace:

```bash
# Create a workspace outside the repo
cd /tmp
mkdir -p opencode-test-workspace
cd opencode-test-workspace
git init

# Invoke directly
/path/to/opencode-container/bin/opencode-container tui
/path/to/opencode-container/bin/opencode-container \
  --feature-file /path/to/opencode-container/tests/testdata/features-common-utils.json \
  tui
```

## Feature JSON Files

- `features-common-utils.json` — Common utils (zsh, git, etc.)
- `features-common-utils-override.json` — Common utils (zsh disabled)
- `features-go.json` — Go
- `features-python.json` — Python

> **Note:** Do not use `ghcr.io/devcontainers/features/node:1` — our base image
> is `node:22-slim` which already includes Node.js. The node feature installs via
> nvm and causes library conflicts (`libatomic.so.1` missing).

## Test Cases

### 1. Fast path — no features

```bash
./tests/e2e/run-test 1
```

**Expected:** Container starts with `localhost/opencode-container`, TUI opens.
No devcontainer CLI involved.

### 2. Feature path — Common utils features

```bash
./tests/e2e/run-test 2
```

**Expected:**
- `npx` installs `@devcontainers/cli` and `node-jq` on first use (slow)
- `devcontainer up` builds container with common-utils feature installed
- TUI opens; `zsh --version` inside container should show installed zsh
- Subsequent runs are faster (cached)

### 3. Feature path — Multiple features merged

```bash
./tests/e2e/run-test 3
```

**Expected:** Both common-utils (zsh) and Go installed inside container. Check with
`zsh --version` and `go version`.

### 4. Feature path — Merge collision, last file wins

`features-common-utils.json` enables zsh; `features-common-utils-override.json` disables it.
When both are passed, the last file's options for the same feature key should override
the first.

```bash
./tests/e2e/run-test 4
```

**Expected:** zsh is NOT installed (disabled by override). Also inspect the generated
`devcontainer.json` to confirm `.features["ghcr.io/devcontainers/features/common-utils:2"].installZsh`
is `false`.

### 5. Feature path — Web mode with features

```bash
./tests/e2e/run-test 5
```

**Expected:**
- Webserver starts on port 5000
- URL printed to stdout
- Ctrl+C stops container cleanly

### 6. Auto-detect hint — Project with .devcontainer directory

```bash
./tests/e2e/run-test 6
```

**Expected:** Stderr prints:
```
Hint: Found .devcontainer/devcontainer.json — use --feature-file .devcontainer/devcontainer.json to include its features
```

Then runs without features (fast path).

### 7. Feature path — Using workspace's own .devcontainer

```bash
./tests/e2e/run-test 7
```

**Expected:** Rust feature installed inside container. Check with `rustc --version`.

### 8. Rebuild — Force image rebuild

```bash
./tests/e2e/run-test 8
```

**Expected:** Container image rebuilds from Containerfile.debian before starting.
Note: Podman still caches unchanged layers, so you may see `--> Using cache`.
For a truly cache-less rebuild, run `podman rmi localhost/opencode-container` first.

### 9. Inspect generated devcontainer.json

```bash
./tests/e2e/run-test 2   # (or any feature-path test to generate config)
./tests/e2e/run-test 9   # Pretty-prints the last generated config
```

**Expected:** Valid JSON with `.features` containing the tested features, `.image`
pointing to `localhost/opencode-container`, and correct mount paths.

To inspect manually:
```bash
PROJECT_ID=$(echo -n "$(pwd)/tests/testdata/test-project" | base64 | tr '+/' '-_' | tr -d '=')
cat ~/.cache/opencode/$PROJECT_ID/devcontainer.json | npx --yes node-jq .
```

## How to Verify Feature Installation

The TUI looks the same regardless of installed features. Verify by running
commands **inside the container**:

1. **From the TUI:** Open a terminal (usually `Ctrl+`` backtick) and run:
   ```bash
   zsh --version    # For common-utils test
   go version       # For Go test
   rustc --version  # For Rust test
   ```

2. **From another terminal** while TUI is running:
   ```bash
   podman exec -it $(podman ps --filter "label=opencode.project.id" --format "{{.Names}}" | head -1) bash
   # Then run verification commands inside
   ```

## Cleanup Between Tests

```bash
# Stop only test-project containers
podman ps --filter "label=opencode.project.id" --format "{{.Names}}"
podman stop <container-name>

# Or stop all opencode containers
podman ps -q --filter "label=opencode.project.id" | xargs -r podman stop -t 5

# Remove images (force rebuild on next run)
podman rmi localhost/opencode-container

# Clear generated devcontainer configs
rm -rf ~/.cache/opencode/
```

> **Warning:** Never run `podman ps -q | xargs podman stop` — this stops ALL
> containers on your machine, including unrelated work sessions.

## Why no Node.js feature tests?

The base image (`node:22-slim`) already includes Node.js. Installing the
`ghcr.io/devcontainers/features/node` feature on top causes conflicts (the
feature installs Node via nvm, which can break the base image's Node setup
and trigger missing shared library errors like `libatomic.so.1`).

Always avoid devcontainer features that duplicate what the base image already
provides.
