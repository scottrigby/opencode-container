<!-- SPDX-License-Identifier: Apache-2.0 -->
# opencode-container

Run the opencode CLI in a Podman container with per-project isolation.

## Quick start

```bash
# TUI mode — image builds automatically on first run
opencode-container

# Web mode
opencode-container web
```

## Docs

- [`install`](docs/install.md)
- [`commands`](docs/commands.md)
- [`design`](docs/design.md)
- [`issues`](docs/issues.md)

## What it does

1. **[Builds a glibc-compatible image](docs/issues.md#glibc--musl-on-alpine)** — layers `gcompat` onto the upstream Alpine image. Auto-builds on first run.
2. **Runs as non-root** (`opencode` user, UID 1000).
3. **Mounts your project** at `/code` — auto-detects git repo roots.
4. **Isolates data per project** under base64-encoded paths, respecting the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest).
5. **Auto-discovers a free port** starting at `4096` (web mode).
6. **Prevents duplicate containers** for the same project using Podman labels.
7. **Handles Ctrl+C gracefully** in web mode.

For the full rationale behind each decision, see [`docs/design.md`](docs/design.md).
