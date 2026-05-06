<!-- SPDX-License-Identifier: Apache-2.0 -->
# opencode-container

Run the opencode CLI in a Podman container with glibc compatibility, non-root
user, and per-project session isolation.

## Quick start

```bash
# TUI mode — image builds automatically on first run
./bin/opencode-container

# Web mode
./bin/opencode-container web
# open http://localhost:PORT

# Force rebuild to pull latest upstream image
./bin/opencode-container --build web
```

For convenience:

```bash
ln -s "$PWD/bin/opencode-container" ~/.local/bin/opencode-container
alias oc='opencode-container'   # add to shell profile
```

## Requirements

- [Podman](https://podman.io) (macOS via Podman Desktop, or Linux)
- `base64`, `lsof` (macOS) or `ss` (Linux)
- `git` (for auto-detecting repo roots)

## What it does

1. **Builds a glibc-compatible image** by layering `gcompat` onto the upstream
   Alpine-based opencode image. The build happens automatically on first run.
2. **Runs as non-root** (`opencode` user, UID 1000) for safety.
3. **Mounts your project** at `/code` — if you're inside a git repo, it
   automatically uses the repo root so the file browser and session scoping
   work correctly.
4. **Isolates data per project** under base64-encoded paths, respecting the
   [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir/latest)
   (`$XDG_DATA_HOME/opencode/` and `$XDG_CONFIG_HOME/opencode/`).
5. **Auto-discovers a free port** starting at `4096`.
6. **Prevents duplicate containers** for the same project using Podman labels.
7. **Handles Ctrl+C gracefully** in web mode.

## Options

| Flag / Variable | Description |
|-----------------|-------------|
| `--build` | Force rebuild the container image (also pulls latest upstream) |
| `OPENCODE_PORT` | Override the default port (`4096`) |
| `OPENCODE_NO_GIT_ROOT=1` | Force mounting the current subdirectory instead of the git repo root |

## Manual Podman commands

If you prefer to run Podman directly instead of using the wrapper, see
[`docs/manual-commands.md`](docs/manual-commands.md) for the equivalent manual commands
(build, TUI, web, label-based container management).

## Data and config layout

```
~/.local/share/opencode/
├── <encoding-a>/          # project A session data (opencode.db, log/, etc.)
├── <encoding-b>/          # project B session data
└── ...

~/.config/opencode/
├── <encoding-a>/          # project A config (auth.json, node_modules/, etc.)
├── <encoding-b>/          # project B config
└── ...
```

Each project is fully isolated. You will need to authenticate (add providers)
once per project. To share auth between projects, copy `auth.json` manually:

```bash
cp ~/.config/opencode/<encoding-a>/auth.json ~/.config/opencode/<encoding-b>/
```

## Known issues

- **Non-git directories:** If you run this in a directory without `.git`, the
  container auto-initialises an empty git repo at `/code` so opencode treats it
  as a proper project root. Remove `.git` on the host afterward if you don't
  want it.
- **Alpine glibc:** Native `.so` libraries loaded at runtime need glibc symbols.
  The `gcompat` shim resolves this. See [`docs/design.md`](docs/design.md) for upstream
  context and why a Debian variant was not used.

## Design & rationale

For detailed reasoning behind every decision (Alpine+gcompat, label
deduplication, git root detection, port scanning, Ctrl+C handling, etc.), see
[`docs/design.md`](docs/design.md).

## Upstream

- [Official Dockerfile](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/Dockerfile)
- [Alpine/musl issue](https://github.com/anomalyco/opencode/issues/9246)
- [Debian variant PR](https://github.com/anomalyco/opencode/pull/9560)
