# Development Feature Files

This directory contains example [devcontainer feature files](https://containers.dev/features) for use with `opencode-container --feature-file`.

## Files

- **`rust-devcontainer.json`** — Full development environment for working on
  `opencode-container` itself. Includes Rust toolchain, docker-outside-of-docker
  (for managing host containers), and common utilities.

- **`docker-outside-of-docker.json`** — Allows the container to manage the host's
  Podman/Docker daemon via `DOCKER_HOST`. Useful for container-related development
  tasks.

- **`common-utils.json`** — Basic shell utilities (zsh, git, curl, etc.). Useful for
  manually testing generated shell completions in a real shell (our automated tests
  in `tests/cli.rs` don't require it).

## Usage

```bash
# Use the Rust development environment
opencode-container --feature-file dev/rust-devcontainer.json

# Combine multiple feature files
opencode-container --feature-file dev/common-utils.json --feature-file dev/docker-outside-of-docker.json
```

## Notes

- The `node` devcontainer feature is **not included** in these examples because
  `opencode-container`'s base image (`node:22-slim`) already includes Node.js.
  Installing the `node` feature on top causes library conflicts
  ([see docs/issues.md](../docs/issues.md#devcontainer-node-feature-conflicts-with-node22-slim-base-image)).
- **Only the `.features` object is read** from these files. Other devcontainer
  properties (`postCreateCommand`, `customizations`, `remoteUser`, etc.) are
  ignored by `opencode-container` because the wrapper generates its own
  `devcontainer.json` with the required mounts, labels, and settings.
- These are starting points — copy and modify them for your project's needs.
