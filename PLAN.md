# Plan: Devcontainer Features for opencode-container

## 1. Base Image

| Variant | Base | When Used |
|---|---|---|
| `debian` (default) | `node:22-slim` (Debian-based, Node pre-installed) | Default. Enables devcontainer features, solves musl/glibc issues. |

- Create `container/Containerfile.debian` based on `node:22-slim`
- Install opencode via `npm install -g`
- Install `git`, reuse existing `node` user (UID 1000), XDG dirs, `/code` workspace
- Re-use existing `entrypoint.sh`

## 2. New CLI Flag

| Flag | Description |
|---|---|
| `--feature-file <path>` | Extract `.features` from a JSON file and merge into generated `devcontainer.json`. Repeatable; later files override on key collision. |

**Auto-detect behavior:** If `.devcontainer/devcontainer.json` exists in the project root, print a hint: *"Use --feature-file .devcontainer/devcontainer.json to include its features"* -- but **do not auto-include**. No surprising magic.

## 3. Two Execution Paths

### Path A: Fast Path (no `--feature-file`)
Same as today. Direct `podman run` with image `localhost/opencode-container`.
- No `devcontainer` CLI or `jq` required.
- Label-based deduplication, port scanning, web mode backgrounding -- all unchanged.

### Path B: Devcontainer Path (`--feature-file` present)
1. **Requirements check:** `devcontainer` CLI and `jq` must be in `PATH`. Friendly error with install hints if missing.
2. **Build base image** if not present (`podman image exists` check).
3. **Merge features** from all `--feature-file` args via `jq` into a single `.features` object.
4. **Generate `devcontainer.json`** into:
   ```
   ${XDG_CACHE_HOME:-$HOME/.cache}/opencode/${PROJECT_ID}/devcontainer.json
   ```
    - `image: localhost/opencode-container`
    - `features: <merged>` (omitted when empty to avoid CLI bugs)
   - `containerEnv`: opencode env vars (OPENCODE_DISABLE_DEFAULT_PLUGINS, etc.)
   - `mounts`: data dir, config dir, code dir
   - `runArgs`: `--label=opencode.project.id=...`, port binds (web mode)
5. **Run:**
   ```bash
   devcontainer up --config <generated> --workspace-folder "${CODE_DIR}"
   devcontainer exec --config <generated> --workspace-folder "${CODE_DIR}" opencode <subcommand>
   ```
6. **Cleanup:** On `EXIT INT TERM`, `devcontainer stop` + `podman rm` by container ID or label.

**Why `XDG_CACHE_HOME`?** This is generated, per-command, disposable config -- not user-edited configuration. It also serves as a debugging artifact (last run's JSON is inspectable).

## 4. Cache Strategy

**No persistent cache volumes for devcontainer mode.**

Rationale:
- Devcontainer features reinstall every run anyway (upstream `RUN --mount=type=bind` limitation).
- Feature caches are platform-specific -- Alpine caches fail on Debian and vice versa.
- The benefit is marginal; the complexity is not worth it for v1.

**Note:** There is currently no mechanism for users to pass arbitrary environment variables or custom mounts to the container. This limits workaround options for persistent caches. See Future TODOs below.

## 5. Web Mode in Devcontainer Path

- Port auto-discovery (`lsof`/`ss`) runs on host before `devcontainer up`.
- Selected port injected into `devcontainer.json` via `forwardPorts` + `runArgs`.
- `devcontainer up` starts container (background).
- `devcontainer exec opencode web --hostname 0.0.0.0 --port PORT` runs in foreground.
- Cleanup trap stops container on `Ctrl+C`.

## 6. Deduplication

- **Fast path:** unchanged (`podman ps --filter label=...` -> abort if running).
- **Devcontainer path:** Same label check before `devcontainer up`. We also set `runArgs: ["--label=opencode.project.id=..."]` in generated JSON so label is visible to Podman.

## 7. File Changes

| File | Change |
|---|---|
| `container/Containerfile.debian` | **New** -- `node:22-slim` base with opencode, git, non-root user |
| `bin/opencode-container` | **Major update** -- `--feature-file` parsing; two-path logic; devcontainer JSON generation; `jq` merging |
| `docs/design.md` | Add section 14: Devcontainer mode rationale, Debian default, cache philosophy |
| `docs/commands.md` | Document `--feature-file` |
| `docs/issues.md` | Update Alpine/glibc section to note Debian default; add node feature conflict |
| `AGENTS.md` | Add build instructions for Debian image |
| `test-completion.sh` | Add completion for `--feature-file` |

## 8. Testing Plan

1. `bash -n bin/opencode-container`
2. `podman build -t localhost/opencode-container -f container/Containerfile.debian container/`
3. Fast path: `opencode-container tui`
4. Feature path: `opencode-container --feature-file <(echo '{"features":{"ghcr.io/devcontainers/features/common-utils:2":{}}}') tui`
5. Web + features: `opencode-container --feature-file ... web --port 5000`
6. Deduplication: run same project twice, verify second aborts
7. `zsh test-completion.sh`

**Note:** Do not use `ghcr.io/devcontainers/features/node:1` — our base image
is `node:22-slim` which already includes Node.js. The node feature installs via
nvm and causes `libatomic.so.1` missing library errors.

## 9. Future TODOs (Deferred to Post-MVP)

| Item | Rationale / Notes |
|---|---|
| **Support `.env` files and `--env` flag** | No mechanism exists today for passing arbitrary env vars into the container. Users may want `NODE_OPTIONS`, `OPENAI_API_KEY`, proxy settings, etc. `--env KEY=VALUE` (repeatable) or `--env-file .env` would forward to `podman run -e` and `devcontainer.json` `containerEnv`. |
| **Support additional mounts** | No mechanism exists for custom bind mounts (e.g., `~/.aws`, `~/.kube/config`, local package caches). A `--mount source=...,target=...` flag (repeatable) would be useful. |
| **Ctrl+C confirmation before closing** | Web mode: easy -- the host shell trap can `read -p "Quit? [y/N] "` before `podman stop`. TUI mode: harder -- the terminal is occupied by the TUI process, so a host-side prompt would compete for the same PTY. Would need to either (a) send a signal into the container and let the TUI handle it, or (b) use a separate notification mechanism. Community issue: [anomalyco/opencode#10975](https://github.com/anomalyco/opencode/issues/10975). |
