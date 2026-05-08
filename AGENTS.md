# opencode-container — Agent Context

## Repo structure

```
opencode-container/
├── bin/opencode-container          # Main wrapper script (bash)
├── container/
│   ├── Containerfile.alpine        # Alpine + gcompat + non-root user
│   ├── Containerfile.debian        # Debian (node:22-slim) + non-root user
│   └── entrypoint.sh               # Auto-init git repo for non-git dirs
├── docs/                           # See docs/ for full reference
├── patches/                        # Upstream patches (see patches/readme.md)
├── README.md
└── AGENTS.md                       # This file
```

## Build and test

```bash
bash -n bin/opencode-container                                              # Syntax check
zsh test-completion.sh                                                      # Completion tests (requires bash + zsh)
./tests/integration/smoke.sh                                                # Dry-run smoke tests (no Podman required)
podman build -t localhost/opencode-container:debian -f container/Containerfile.debian container/   # Build Debian image
podman build -t localhost/opencode-container:alpine -f container/Containerfile.alpine container/       # Build Alpine image
```

## Manual E2E testing

See `tests/testdata/README.md` for ready-made feature files and a full manual
test checklist. Run from a sibling directory (e.g. `/tmp/test-workspace`) to
verify correct project scoping.

## Key constraints (read before changing)

| Decision | Rationale | Detail |
|----------|-----------|--------|
| No `--name` | Auto-generated names prevent collisions | [design.md](docs/design.md#9-label-based-deduplication) |
| Debian base (`node:22-slim`) | Devcontainer feature compatibility, glibc | [design.md](docs/design.md#2-debian-base-image-node22-slim) |
| No cache volume | Prevents races between concurrent containers | [design.md](docs/design.md#7-no-persistent-cache-volume) |
| Web mode: `-i` not `-t` | `-t` breaks `Ctrl+C` in some terminals | [design.md](docs/design.md#8-web-mode) |

## Coding conventions

- `#!/bin/bash` with `set -euo pipefail`
- `local` only inside functions (top-level `local` + `set -e` = immediate exit)
- `seq 1 N` not `{1..N}` for portability
- Two-phase parsing: global flags first, then subcommand-specific
- `--help` intercepted at **any position**

## Interface

Full reference: [docs/commands.md](docs/commands.md) · `opencode-container --help`

Quick summary: `[options] [--] [opencode-args...]` or `[options] <command> [command-options]`
- Options: `-b/--build`, `-f/--feature-file PATH`, `--env-file PATH`, `-e/--env VAR=value`, `--local-env VAR`, `-w/--web`, `-p/--port PORT`, `--no-open`, `--no-git-root`, `--no-git-init`, `-h/--help`
- Commands: `projects`, `completion`
- Default mode: TUI (use `-w/--web` for web mode)

## Dependencies

- **Podman** (or Docker) — required for all modes.
- **Node + npx** — required only when using `--feature-file`; auto-installs
  `@devcontainers/cli` and `node-jq` on first use.

## Gotchas

- **macOS `/var` → `/private/var`**: `resolve_path()` resolves symlinks before computing `PROJECT_ID`
- **Empty git repos show "Create Git repository"**: `0002` patch fixes upstream, not yet applied
- **Zsh completion**: `compadd` requires completion dispatch context — use `test-completion.sh` to verify
- **Bash completion**: self-contained, no `bash-completion` package dependency

## Upstream

- `https://github.com/anomalyco/opencode` · branch `dev`
- `/workspace/opencode-src` available in some sessions
