<!-- SPDX-License-Identifier: Apache-2.0 -->
# Design Rationale

This document captures the reasoning behind key decisions in `opencode-container`.

For a concise list of issues that led to these decisions, see [`docs/issues.md`](issues.md).

## Philosophy

The wrapper should be invisible to daily use: run `opencode-container` from any
directory and get a working opencode CLI/TUI/WebUI with correct project scoping,
per-project isolation, and no surprise side effects on the host filesystem.

---

## 1. Podman, not Docker

Podman is used because:
- Rootless containers work out of the box on macOS and Linux.
- No daemon model means `Ctrl+C` cleanup is straightforward.

---

## 2. Debian base image (`node:22-slim`)

The upstream image (`ghcr.io/anomalyco/opencode:latest`) is Alpine-based and
uses musl libc, which lacks glibc symbols required by native `.so` libraries:

```
Error relocating ...so: gnu_get_libc_version: symbol not found
```

Most [devcontainer features](https://containers.dev/features)
(community-maintained installation scripts for Node, Python, Go, etc.) assume a
glibc-based distribution and fail on Alpine.

The glibc issue also breaks the **web UI terminal**. The terminal WebAssembly
module (`ghostty-vt.wasm`) requires glibc and fails on Alpine even with `gcompat`:

```
WebAssembly.compile(): expected magic word 00 61 73 6d, found 3c 21 64 6f @+0
```

The magic word error is the browser reading HTML bytes (`0x3c21646f` = `<!do`)
as WASM — the WASM file was not found and the server returned the SPA shell.

Therefore, the base image is **Debian** (`node:22-slim`) with opencode installed
via npm (`opencode-ai`). The `node` user (UID 1000) provided by the base image
is reused rather than creating a separate user, keeping UID/GID consistent with
standard Node.js container conventions.

**Why not Alpine at all?** Alpine support was dropped entirely to simplify
maintenance. A single base image avoids platform-specific cache conflicts,
gcompat shim issues, and feature incompatibility. Users who need Alpine can use
the upstream image directly.

See upstream [#9246](https://github.com/anomalyco/opencode/issues/9246) and
[#9560](https://github.com/anomalyco/opencode/pull/9560) for upstream discussion
of a Debian variant.

---

## 3. `localhost/opencode-container` image name

The image name matches the script name (`opencode-container`) so they form a
self-describing pair. Podman prepends `localhost/` to locally-built images
automatically. No registry credentials or tag management are needed.

---

## 4. Non-root user (`node`, UID 1000)

The upstream image runs as root, which gives the file browser write access to the
entire container filesystem. Running as a non-root user:
- Limits blast radius if the container is compromised.
- Makes system directories outside `/code` read-only.
- Prevents accidental writes to host-mounted paths that might have different UID
  mappings.

The `node:22-slim` base image already provides a `node` user (UID 1000). We
reuse this user instead of creating a separate `opencode` user because:
- Avoids GID/UID collision with the host user's groups.
- Matches Node.js container conventions (standard practice in the Node ecosystem).
- Simplifies the Containerfile — no `useradd`/`groupadd` boilerplate needed.

---

## 5. No shadowing of the `opencode` binary

The binary is named `opencode-container` so it never shadows a native macOS
install. Users can alias it locally (e.g. `alias oc=opencode-container`) but the
binary name itself is explicit.

---

## 6. Per-project data/config isolation (not shared)

Each project directory gets its own `opencode.db`, `auth.json`, `node_modules`,
etc. under base64-encoded paths. Data, config, and cache are always stored in
separate subdirectories (`data/`, `config/`, `cache/`), even on platforms where
the base `dirs::data_dir()` and `dirs::config_dir()` return the same path
(e.g., macOS `~/Library/Application Support/`). This ensures the Linux container
receives distinct mount points for data and config, matching its XDG
expectations:

| Platform | Data directory | Config directory | Cache directory |
|----------|---------------|------------------|-----------------|
| Linux | `~/.local/share/opencode/data/<encoding>/` | `~/.config/opencode/config/<encoding>/` | `~/.cache/opencode/cache/<encoding>/` |
| macOS | `~/Library/Application Support/opencode/data/<encoding>/` | `~/Library/Application Support/opencode/config/<encoding>/` | `~/Library/Caches/opencode/cache/<encoding>/` |
| Windows | `%APPDATA%/opencode/data/<encoding>/` | `%APPDATA%/opencode/config/<encoding>/` | `%LOCALAPPDATA%/opencode/cache/<encoding>/` |

This is **intentionally not shared** across projects to prevent:
- SQLite database lock conflicts when multiple containers run simultaneously.
- Plugin install races on shared `node_modules`.
- Auth cross-contamination between unrelated projects.

Users can manually copy `auth.json` between projects if they want to share keys.

---

## 7. No persistent cache volume

`${XDG_CACHE_HOME}` is intentionally **not** mounted. The only meaningful cached
asset is `models.json`, which is re-downloaded automatically on first run. This
prevents race conditions when multiple `opencode-container` instances run
concurrently.

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

Container names are auto-generated by Podman (e.g. `gracious_borg`). The **full**
base64 project ID is stored as a Podman label:
`opencode.project.id=<full-hash>`.

Deduplication, cleanup, and attach all use `podman ps --filter label=...`. This
is exact regardless of how long or similar paths are.

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

Two patches are included in the [`patches/`](../patches) directory as a reminder
to contribute them upstream. They are **not required** for daily use; the
`entrypoint.sh` + host git detection handle the pre-built image.

See [`patches/readme.md`](../patches/readme.md) for patch details and how to
apply them. If accepted upstream, the `entrypoint.sh` `git init` workaround for
non-git directories could be removed.

---

## 12. Web mode passthrough via `run` subcommand

Web mode is detected when the first opencode argument is `web` (e.g.
`opencode-container run web --port 5000`). The `run` subcommand (which is the
default when no subcommand is given) passes all trailing arguments through to
opencode.

### Why passthrough as the primary mechanism?

- **Thinner wrapper:** No re-implementation of opencode's CLI surface. Users use
  opencode's native flags (`--port`, `--hostname`, `--pure`, etc.) directly.
- **No flag collisions:** A wrapper `--port` could conflict with future opencode
  flags or subcommands. Passthrough eliminates this entirely.
- **Natural UX:** `docker run` and `podman run` use this pattern
  (`podman run image <cmd> [args...]`). The wrapper follows the same convention.
- **Clean completions:** Separating the wrapper's global flags and subcommands
  from opencode's arguments fixes shell completion generation (clap_complete
  struggles with positional `allow_hyphen_values` args mixed with subcommands).

### Injected defaults

The CLI inspects opencode's `web` args and injects two defaults if missing:

- **`--hostname 0.0.0.0`**: Required for the host browser to reach the container.
  If the user sets a custom `--hostname`, the wrapper skips browser auto-open
  (the host likely cannot reach a non-default hostname).
- **`--port 4096`**: Provides a predictable default for port forwarding and URL
  generation. If the user sets `--port`, that value is used instead.

### Skipping web-mode infrastructure for non-server commands

If the opencode args include `--help`, `-h`, `--version`, `-v`, or `help`,
opencode will not start a web server. In this case the wrapper skips port
binding, health-check polling, and browser auto-open entirely. The container
starts normally and the command runs as a one-off (e.g. showing help text
and exiting). This avoids confusing port-scan messages and unnecessary
infrastructure when the user just wants to read help or check the version.

### Port auto-discovery

Web mode scans from `4096` upward using `lsof` (macOS) or `ss` (Linux) to find
an unused port. If the default is taken, it prints a message and continues. This
prevents "address already in use" crashes when multiple web containers run.

---

## 13. `Ctrl+C` and graceful shutdown

In web mode, the container runs as a background child process. The Rust binary
uses the `ctrlc` crate to install a `SIGINT` handler. On `Ctrl+C`:
1. The signal handler fires in the Rust process.
2. A cleanup flag is set atomically.
3. The main thread detects the flag and calls `podman stop -t 5 <container>`.
4. Container receives `SIGTERM`, shuts down gracefully.
5. The process exits cleanly.

For devcontainer mode, cleanup also calls `devcontainer stop` and `podman rm -f`
to ensure no orphaned containers remain. The `ctrlc` crate is cross-platform
(Linux, macOS, Windows), unlike shell traps which are POSIX-specific.

---

## 14. Devcontainer mode (`--feature-file`)

When `--feature-file` is passed, the CLI switches from a direct `podman run`
to the **devcontainer CLI**, which layers community devcontainer features onto
the base image at container startup.

### Why two paths?

- **Fast path** (no `--feature-file`): direct `podman run` is instant and has no
  dependencies beyond Podman.
- **Feature path**: the devcontainer CLI is the standard mechanism for installing
  and caching devcontainer features. Re-implementing feature installation in the
  CLI would be complex, fragile, and incompatible with the ecosystem.

### `--build` flag behavior

`--build` forces `podman build` to run instead of skipping it when the image
already exists. However, Podman still caches unchanged Dockerfile layers, so
you may see `--> Using cache` in the output even with `--build`.

To force a truly cache-less rebuild (e.g., after changing the Containerfile or
when debugging layer issues), remove the image first:

```bash
podman rmi localhost/opencode-container
opencode-container -b
```

This deletes all layers and rebuilds from scratch.

### TTY allocation in devcontainer TUI mode

`devcontainer exec` inherits TTY allocation from its parent process
automatically when stdin is a terminal. Key handling (arrow keys, Ctrl+C,
etc.) works correctly through the inherited terminal.

### Why `npx` for devcontainer CLI

The devcontainer CLI (`@devcontainers/cli`) is invoked via `npx --yes`, which
downloads it on first use and caches it for subsequent runs. If the devcontainer
CLI is already installed globally, the binary uses the global version and skips
the npx fallback.

### Why `serde_json` replaces `jq`

The bash implementation used `node-jq` (via `npx`) to merge `.features` objects
from multiple `--feature-file` arguments and generate `devcontainer.json`. The
Rust rewrite uses `serde_json` natively:

- **No Node.js dependency** for core functionality — only needed if using
  `--feature-file` with the devcontainer CLI.
- **Type-safe JSON manipulation** — the compiler guarantees valid JSON structure.
- **Testable** — JSON generation logic is unit-tested in `cargo test`.
- **Faster** — no process spawn overhead for `jq` invocations.

Feature files are read into `serde_json::Value`, the `.features` objects are
merged in memory, and the final `devcontainer.json` is written atomically to
the cache directory.

### Why `containerEnv` values must be strings

The devcontainer CLI v0.86.1 (and VS Code Dev Containers extension 0.396.0+)
has a bug where non-string values in `containerEnv` (booleans, numbers) cause
a `TypeError: [X].replace is not a function` during feature processing.
This is a known upstream issue
([microsoft/vscode-remote-release#10691](https://github.com/microsoft/vscode-remote-release/issues/10691)).

The CLI always generates string values (`"true"`, `"1"`) to avoid this.

### Why empty `features: {}` is omitted

Some devcontainer CLI versions fail when processing an empty features object
during dependency resolution. The CLI omits the `features` key entirely
when no `--feature-file` arguments are provided, generating it only when
features are actually present.

### Why `--progress=plain` is injected

BuildKit's animated progress renderer emits ANSI cursor-movement sequences
that corrupt the terminal after `devcontainer up` completes. Injecting
`build.options: ["--progress=plain"]` in the generated `devcontainer.json`
switches to plain line-by-line output, matching
[claudeman's approach](https://github.com/scottrigby/claudeman/pull/24).

### Why stderr is streamed live during `devcontainer up`

Feature installation (e.g., `common-utils:2`) can take a long time. Silently
capturing all output leaves the user staring at a blank screen with no
feedback. The CLI captures stdout silently (for potential JSON parsing)
while streaming stderr live to the terminal, so the user sees build progress
in real time.

### Why `ghcr.io/devcontainers/features/node:1` is incompatible

Our base image is `node:22-slim` which already includes Node.js. Installing
the `node` devcontainer feature on top causes a conflict: the feature installs
Node via nvm, which expects `libatomic.so.1` — a library not present in the
slim Debian base. This causes the feature build to fail with:

```
node: error while loading shared libraries: libatomic.so.1: cannot open shared object file
```

The CLI does not block this feature (the user controls `--feature-file`),
but test fixtures and documentation avoid it. This incompatibility is not
documented in the upstream feature README.

### Why Podman template uses `{{.ID}}` (uppercase)

Podman's Go template system uses `{{.ID}}` (uppercase), while Docker uses
`{{.Id}}` (mixed case). The CLI uses uppercase to match Podman's API.
This was discovered when `podman ps --format '{{.Id}}'` failed with:

```
Error: template: ps:1:13: executing "ps" at <.Id>: can't evaluate field Id in type containers.psReporter
```

### Why `devcontainer.json` is generated per-run

The JSON is assembled by merging `.features` objects from all `--feature-file`
arguments using `serde_json`, then layering opencode-specific mounts, labels, and
environment variables on top. Because the set of features may change between
runs, the JSON is regenerated each time into:

```
${XDG_CACHE_HOME}/opencode/<PROJECT_ID>/devcontainer.json
```

This location is disposable (cache, not config) but inspectable for debugging.

### Why no persistent cache volumes in feature path

Devcontainer features reinstall on every `devcontainer up` due to an upstream
limitation (`RUN --mount=type=bind` prevents layer caching). Platform-specific
package caches (npm, pip, Go modules) are tied to the base image architecture
and would be invalid if the image changes. Persistent caches add complexity
without meaningful speedup in this mode.

**Future TODO:** If `--feature-file` JSONs could define `cacheEnv` entries (as
claudeman profiles do), the CLI could generate `mounts` + `remoteEnv` entries
to persist caches per-project under `XDG_CACHE_HOME/opencode/<PROJECT_ID>/cache/`.
This would speed up subsequent rebuilds without cross-project races.

### Why Debian by default when using features

Most devcontainer features assume `apt` and glibc paths. Defaulting to Debian
removes the gcompat shim and aligns with ecosystem expectations. A single
base image also simplifies cache management and avoids cross-platform cache
invalidation.

### Why `.env` auto-detection

The CLI auto-detects `.env` in the project root (git repository root or
current directory) and passes it to the container via `--env-file`. This follows
the principle of least surprise — if a project has environment configuration, it
should be available in the container without explicit flags.

Manual `--env-file <path>` flags (repeatable) override and supplement the
auto-detected file. In devcontainer mode, `--env-file` entries are added to
`runArgs` in the generated `devcontainer.json`. In fast path, they are passed
directly to `podman run`.

---

## 15. Cross-compilation and release architecture

The Rust binary is compiled for the host architecture by default. This means:
- Building inside a Linux container produces a Linux binary.
- Building on macOS produces a Darwin binary.
- The two are not interchangeable.

### Release strategy

GitHub Actions builds release binaries for multiple targets:

| Target | Architecture | Notes |
|--------|-------------|-------|
| `x86_64-unknown-linux-gnu` | Linux AMD64 | Standard Linux servers |
| `aarch64-unknown-linux-gnu` | Linux ARM64 | Raspberry Pi, ARM servers, Apple Silicon VMs |
| `x86_64-apple-darwin` | macOS Intel | Older Macs |
| `aarch64-apple-darwin` | macOS Apple Silicon | M1/M2/M3 Macs |
| `x86_64-pc-windows-msvc` | Windows AMD64 | Windows 10/11 |

Users download the appropriate binary from GitHub Releases. No Rust toolchain
is required for end users.

### Development workflow

When working across architectures (e.g., developing in a Linux devcontainer on
an Apple Silicon Mac), the developer compiles for their host architecture
outside the container:

```bash
# On macOS host
cargo build --release
# Produces target/release/opencode-container for Darwin
```

Inside the Linux devcontainer, `cargo build --release` produces a Linux binary
(useful for testing the Linux path, but not for macOS distribution).

Cross-compilation from Linux to macOS requires the macOS SDK and is not
trivially available in a standard Linux container. GitHub Actions (macOS runners)
handle this natively.

---

## Open questions / future work

- **Build from source:** Once upstream adopts the non-git worktree fix, the
  `entrypoint.sh` `git init` workaround can be removed.
- **Cross-platform port scanning:** Windows/WSL support would need a different port
  scan mechanism (the current `lsof`/`ss` approach is POSIX-specific).
- **Custom mounts:** No mechanism exists for additional bind mounts (e.g.
  `~/.aws`, `~/.kube/config`).
- **Embed Containerfiles:** Use `include_str!` to embed `Containerfile.debian`
  and `entrypoint.sh` into the binary, making it fully self-contained.
