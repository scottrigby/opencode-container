# Manual E2E Test Data

These files support manual end-to-end testing of the `--feature-file` flag and
devcontainer feature integration. Run tests from a **sibling directory** of the
`opencode-container/` project to verify correct project scoping.

## Prerequisites

- Podman (or Docker)
- Node.js + npm (for `npx`)
- `opencode-container` on your PATH or invoked via full path

## Setup

```bash
# From outside the opencode-container repo, create a sibling test workspace
cd /tmp
mkdir -p opencode-test-workspace
cd opencode-test-workspace

# Symlink or copy the opencode-container binary to your PATH
# Or invoke it directly:
#   /path/to/opencode-container/bin/opencode-container
```

## Test Cases

### 1. Fast path — no features

```bash
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container tui
```

**Expected:** Container starts with `localhost/opencode-container`, TUI opens.
No devcontainer CLI involved.

### 2. Feature path — Common utils features

```bash
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container \
  --feature-file /path/to/opencode-container/tests/testdata/features-common-utils.json \
  tui
```

**Expected:**
- `npx` installs `@devcontainers/cli` and `node-jq` on first use (slow)
- `devcontainer up` builds container with common-utils feature installed
- TUI opens; `zsh --version` inside container should show installed zsh
- Subsequent runs are faster (cached)

### 3. Feature path — Multiple features merged

```bash
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container \
  --feature-file /path/to/opencode-container/tests/testdata/features-common-utils.json \
  --feature-file /path/to/opencode-container/tests/testdata/features-go.json \
  tui
```

**Expected:** Both common-utils (zsh) and Go installed inside container. Check with
`zsh --version` and `go version`.

### 4. Feature path — Merge collision, last file wins

`features-common-utils.json` enables zsh; `features-common-utils-override.json` disables it.
When both are passed, the last file's options for the same feature key should override
the first.

```bash
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container \
  --feature-file /path/to/opencode-container/tests/testdata/features-common-utils.json \
  --feature-file /path/to/opencode-container/tests/testdata/features-common-utils-override.json \
  tui
```

**Expected:** zsh is NOT installed (disabled by override). Also inspect the generated
`devcontainer.json` to confirm `.features["ghcr.io/devcontainers/features/common-utils:2"].installZsh`
is `false`.

### 5. Feature path — Web mode with features

```bash
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container \
  --feature-file /path/to/opencode-container/tests/testdata/features-common-utils.json \
  web --port 5000
```

**Expected:**
- Webserver starts on port 5000
- URL printed to stdout
- Ctrl+C stops container cleanly

### 6. Auto-detect hint — Project with .devcontainer directory

```bash
cd /path/to/opencode-container/tests/testdata/example-project
/path/to/opencode-container/bin/opencode-container tui
```

**Expected:** Stderr prints:
```
Hint: Found .devcontainer/devcontainer.json — use --feature-file .devcontainer/devcontainer.json to include its features
```

Then runs without features (fast path).

### 7. Feature path — Using project's own .devcontainer

```bash
cd /path/to/opencode-container/tests/testdata/example-project
/path/to/opencode-container/bin/opencode-container \
  --feature-file .devcontainer/devcontainer.json \
  tui
```

**Expected:** Rust feature installed inside container. Check with `rustc --version`.

### 8. Deduplication — Same project, second instance

```bash
# Terminal 1
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container tui

# Terminal 2 (same directory)
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container tui
```

**Expected:** Terminal 2 aborts with "already running for this project".

### 9. Rebuild — Force image rebuild

```bash
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container --build tui
```

**Expected:** Container image rebuilds from Containerfile.debian before starting.

### 10. Cache inspection — Generated devcontainer.json

```bash
cd /tmp/opencode-test-workspace
/path/to/opencode-container/bin/opencode-container \
  --feature-file /path/to/opencode-container/tests/testdata/features-common-utils.json \
  tui

# In another shell, inspect the generated config:
cat ~/.cache/opencode/$(echo -n /tmp/opencode-test-workspace | base64 | tr '+/' '-_' | tr -d '=')/devcontainer.json
```

**Expected:** Valid JSON with `.features` containing the common-utils feature, `.image`
pointing to `localhost/opencode-container`, and correct mount paths.

## Cleanup Between Tests

```bash
# Stop all running containers
podman ps -q | xargs -r podman stop -t 5

# Remove all containers
podman ps -aq | xargs -r podman rm -f

# Remove images (force rebuild on next run)
podman rmi localhost/opencode-container

# Clear generated devcontainer configs
rm -rf ~/.cache/opencode/
```

## Why no Node.js feature tests?

The base image (`node:22-slim`) already includes Node.js. Installing the
`ghcr.io/devcontainers/features/node` feature on top causes conflicts (the
feature installs Node via nvm, which can break the base image's Node setup
and trigger missing shared library errors like `libatomic.so.1`).

Always avoid devcontainer features that duplicate what the base image already
provides.
