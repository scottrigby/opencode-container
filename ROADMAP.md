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

### Cross-platform port scanning

**Idea:** The current port discovery uses `lsof` (macOS) or `ss` (Linux), which
are POSIX-specific. Windows/WSL support would need a different mechanism.

**Reference:** [`docs/design.md#12-web-mode-passthrough`](design.md#12-web-mode-passthrough)
