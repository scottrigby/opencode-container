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

## Installation

See [`docs/install.md`](docs/install.md) for requirements, install options (GitHub
Releases, Cargo, or build from source), shell completion setup, and data layout.

## Docs

- [`install`](docs/install.md) — installation, requirements, data layout
- [`commands`](docs/commands.md) — CLI reference
- [`design`](docs/design.md) — architecture rationale
- [`issues`](docs/issues.md) — known upstream issues and local patches

## What it does

1. **[Builds a glibc-compatible image](docs/issues.md#glibc--musl-on-alpine)** — layers `gcompat` onto the upstream Alpine image. Auto-builds on first run.
2. **Runs as non-root** (`node` user, UID 1000).
3. **Mounts your project** at `/code` — auto-detects git repo roots.
4. **Isolates data per project** under base64url-encoded paths, respecting the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest).
5. **Auto-discovers a free port** starting at `4096` (web mode).
6. **Prevents duplicate containers** for the same project using Podman labels.
7. **Handles Ctrl+C gracefully** in web mode.
8. **Native CLI** — written in Rust with [clap](https://crates.io/crates/clap) for robust argument parsing and auto-generated shell completions.

For the full rationale behind each decision, see [`docs/design.md`](docs/design.md).

## Development

```bash
git clone https://github.com/scottrigby/opencode-container
cd opencode-container
cargo build --release
cargo test
```

For a full development environment with Rust toolchain, docker-outside-of-docker,
and common utilities, use the provided devcontainer feature file:

```bash
opencode-container --feature-file dev/rust-devcontainer.json
```

See [`AGENTS.md`](AGENTS.md) for the full developer workflow and [`dev/`](dev/)
for additional feature file examples.
