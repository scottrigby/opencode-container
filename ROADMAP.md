# Future Enhancements

This file tracks feature ideas and known issues for post-MVP implementation.

## Ideas

### Embed Containerfiles into the binary

**Idea:** Use Rust's `include_str!` macro to embed `Containerfile.debian` and
`entrypoint.sh` into the binary at compile time. The binary could then
self-extract these files to a cache directory at runtime, making it a fully
self-contained single-file distribution.

**Benefit:** No need to ship the `container/` directory alongside the binary.

**Reference:** [`docs/design.md#15-cross-compilation-and-release-architecture`](design.md#15-cross-compilation-and-release-architecture)

---

### Config file support

**Idea:** Support a `.opencode-container.toml` or `.opencode-container.json` config
file in the project root for per-project defaults (e.g., default port, feature
files, env files).

---

### Custom mounts

**Idea:** Support additional bind mounts beyond the workspace, data, and config
directories. This would allow passing host directories like `~/.aws`,
`~/.kube/config`, or local package caches into the container.

**Use case:** Users working with cloud provider credentials, Kubernetes configs,
or monorepo shared dependencies that live outside the project root.

**Reference:** [`docs/design.md#7-no-persistent-cache-volume`](design.md#7-no-persistent-cache-volume)
for context on why persistent caches are not mounted by default.

---

### `exec` command for interactive shell

**Idea:** Add an `exec` subcommand to get an interactive shell into a running
opencode container for the current project. Useful for debugging, installing
system packages, or inspecting the container state without starting a new
opencode session.

**Example:**
```bash
opencode-container exec bash
```

**Implementation notes:**
- Find the running container using the project ID label (`opencode.project.id`)
- If no container is running, error with a helpful message
- Use `podman exec -it` or `devcontainer exec` depending on which path the
current project was started with (or detect from container labels)

---

### Data/config migration command

**Idea:** A command to migrate project-specific opencode data and config from an
old project directory to a new one. When a project is moved or renamed, the
base64-encoded isolation paths change, causing the new location to appear as a
fresh project (lost history, auth, etc.).

**Example:**
```bash
opencode-container migrate /old/path /new/path
```

**Implementation notes:**
- Compute the old and new project IDs from the given paths
- Move or copy directories under `~/.local/share/opencode/data/` and
`~/.config/opencode/config/`
- Optionally support `--dry-run` to preview what would be moved
- Warn if the new location already has data

---

### Cross-platform port scanning

**Idea:** The current port discovery uses `lsof` (macOS) or `ss` (Linux), which
are POSIX-specific. Windows/WSL support would need a different mechanism.

**Reference:** [`docs/design.md#12-web-mode-passthrough`](design.md#12-web-mode-passthrough)
