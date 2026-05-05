# Design Rationale

This document captures the reasoning behind key decisions in `opencode-container`.

## Philosophy

The wrapper should be invisible to daily use: run `opencode-container` from any
directory and get a working opencode CLI/TUI/WebUI with correct project scoping,
per-project isolation, and no surprise side effects on the host filesystem.

---

## 1. Podman, not Docker

Podman is used because:
- Rootless containers work out of the box on macOS and Linux.
- No daemon model means `Ctrl+C` cleanup is straightforward.
- The target machine (macOS M4 via Podman Desktop) already uses it.

---

## 2. Alpine + `gcompat` instead of Debian

The upstream image is Alpine-based (`ghcr.io/anomalyco/opencode:latest`). Rather
than maintaining a forked Debian image, we layer on `gcompat` to resolve glibc
symbol errors at runtime:

```
Error relocating ...so: gnu_get_libc_version: symbol not found
```

Native `.so` libraries loaded via Bun FFI expect glibc. `LD_PRELOAD=/lib/libgcompat.so.0`
shims these calls. This keeps us close to upstream and avoids a full rebuild.

See upstream [#9246](https://github.com/anomalyco/opencode/issues/9246) and
[#9560](https://github.com/anomalyco/opencode/pull/9560) for ongoing discussion
of a Debian variant.

---

## 3. `localhost/opencode-container` image name

The image name matches the script name (`opencode-container`) so they form a
self-describing pair. Podman prepends `localhost/` to locally-built images
automatically. No registry credentials or tag management are needed.

---

## 4. Non-root user (`opencode`, UID 1000)

The upstream image runs as root, which gives the file browser write access to the
entire container filesystem. Running as a non-root user:
- Limits blast radius if the container is compromised.
- Makes system directories outside `/code` read-only.
- Prevents accidental writes to host-mounted paths that might have different UID
  mappings.

---

## 5. No shadowing of the `opencode` binary

The script is named `opencode-container` so it never shadows a native macOS
install. Users can alias it locally (e.g. `alias oc=opencode-container`) but the
binary name itself is explicit.

---

## 6. Per-project data/config isolation (not shared)

Each project directory gets its own `opencode.db`, `auth.json`, `node_modules`,
etc. under hashed paths:

```
~/.local/share/opencode/<hash>/
~/.config/opencode/<hash>/
```

This is **intentionally not shared** across projects to prevent:
- SQLite database lock conflicts when multiple containers run simultaneously.
- Plugin install races on shared `node_modules`.
- Auth cross-contamination between unrelated projects.

Users can manually copy `auth.json` between projects if they want to share keys.

---

## 7. No persistent cache volume

`${XDG_CACHE_HOME}` is intentionally **not** mounted. The only meaningful cached
asset is `models.json`, which is re-downloaded automatically on first run. This
eliminates concurrency issues when multiple team members (or CI jobs) run
containers simultaneously.

---

## 8. Web mode: `podman run -i` (no TTY) + host trap

Web mode runs `podman run -i --rm --init ... &` in the background with a host
shell `trap cleanup EXIT INT TERM`. This gives us:
- `Ctrl+C` caught by the host shell, which calls `podman stop`.
- `--init` ensures `SIGTERM` is forwarded correctly to the server process.
- `-t` is **not** used because it would allocate a PTY and prevent the trap from
  firing reliably in some terminal emulators.

---

## 9. Label-based deduplication instead of container names

Originally we truncated base64-encoded paths to create container names
(`opencode-<hash[:16]>`). This caused false positives when two directories shared
a prefix (e.g. two temp dirs under `/var/folders/...`).

We now:
- Let Podman auto-generate unique container names (e.g. `gracious_borg`).
- Store the **full** base64 project ID as a Podman label:
  `opencode.project.id=<full-hash>`.
- Use `podman ps --filter label=...` for deduplication, cleanup, and attach.

This is exact regardless of how long or similar paths are.

---

## 10. Git root detection on the host

opencode's server discovers the project root by searching for `.git`. If it
can't find one, it falls back to `/`, which causes:
- Sessions stored with `directory = "/"`.
- Web UI navigating to `/Lw` instead of the actual project.
- Broken sidebar, missing project icon, and terminal WebAssembly errors.

### Why not just `git init` inside the container?

If we only mount `$PWD` into the container, a subdirectory of a git repo has no
`.git` visible. The container would create an **empty** git repo at `/code`, which
is even worse (no commits → `ProjectID.global` fallback → half-initialized state).

### Solution

The wrapper runs `git rev-parse --show-toplevel` **on the host** before starting
the container. If inside a git repo, it mounts the repo root as `/code`. This
matches native opencode behavior: project boundaries are git roots.

For non-git directories, the container `entrypoint.sh` initialises an empty git
repo as a safe, reversible fallback. This is harmless and fully removable with
`rm -rf .git`.

Opt-out: `OPENCODE_NO_GIT_ROOT=1` forces subdirectory scoping (useful for large
monorepos where you only want to work on one package).

---

## 11. Upstream source patches

Two patches to the opencode source are included for completeness:

### `project.fromDirectory` non-git fallback

In `packages/opencode/src/project/project.ts`, when no `.git` is found, the
original code returns `worktree: "/"`. We changed it to:

```ts
worktree: directory,
sandbox: directory,
```

This prevents the server from collapsing all non-git directories into a single
global project. The patch is only relevant if you rebuild the container from
source; the `entrypoint.sh` + host git detection already handle the pre-built
image.

### TUI plugin runtime `vcs` inference

In `packages/opencode/src/cli/cmd/tui/plugin/runtime.ts`, the original code
infers `vcs: "git"` from `dir.worktree !== "/"`. We replaced this with a check
against the actual sync state so the plugin runtime doesn't misreport a non-git
directory as git after the worktree change.

---

## 12. Port auto-discovery

Web mode scans from `4096` upward using `lsof` (macOS) or `ss` (Linux) to find
an unused port. If the default is taken, it prints a message and continues. This
prevents "address already in use" crashes when multiple web containers run.

---

## 13. `Ctrl+C` and `podman stop`

The web mode wrapper runs the container in the background (`&`) and installs a
`trap cleanup EXIT INT TERM` in the **host shell**. On `Ctrl+C`:
1. Host shell trap fires.
2. `podman stop -t 5 <container>` is called.
3. Container receives `SIGTERM`, shuts down gracefully.
4. `wait` returns, shell exits.

This is more reliable than relying on `podman run -it` catching signals
inside the container.

---

## Open questions / future work

- **Dev Container variant:** Pre-installing project-specific dependencies (Node,
  Python, etc.) inside the container instead of relying on host tooling.
- **Build from source:** Once upstream adopts the non-git worktree fix, the
  `entrypoint.sh` `git init` workaround can be removed.
- **Cross-platform `lsof`/`ss`:** Windows/WSL support would need a different port
  scan mechanism.
