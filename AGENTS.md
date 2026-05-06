# opencode-container — Agent Context

## Repo structure

```
opencode-container/
├── bin/opencode-container          # Main wrapper script (bash)
├── container/
│   ├── Containerfile               # Alpine + gcompat + non-root user
│   └── entrypoint.sh               # Auto-init git repo for non-git dirs
├── docs/                           # See docs/ for full reference
├── patches/                        # Upstream patches (see patches/readme.md)
├── README.md
└── AGENTS.md                       # This file
```

## Build and test

```bash
bash -n bin/opencode-container          # Syntax check
zsh test-completion.sh                  # Completion tests (requires bash + zsh)
podman build -t localhost/opencode-container container/   # Build image
```

## Key constraints (read before changing)

| Decision | Rationale | Detail |
|----------|-----------|--------|
| No `--name` | Auto-generated names prevent collisions | [design.md](docs/design.md#9-label-based-deduplication) |
| Alpine + gcompat | Close to upstream, avoids fork | [design.md](docs/design.md#2-alpine--gcompat) |
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

Quick summary: `[global-options] [subcommand] [subcommand-options]`
- Global: `-b/--build`, `--no-git-root`, `--no-git-init`, `-h/--help`
- Subcommands: `tui` (default), `web`, `completion`, `projects`
- `web`: `-p/--port PORT`, `--no-open`

## Gotchas

- **macOS `/var` → `/private/var`**: `resolve_path()` resolves symlinks before computing `PROJECT_ID`
- **Empty git repos show "Create Git repository"**: `0002` patch fixes upstream, not yet applied
- **Zsh completion**: `compadd` requires completion dispatch context — use `test-completion.sh` to verify
- **Bash completion**: self-contained, no `bash-completion` package dependency

## Upstream

- `https://github.com/anomalyco/opencode` · branch `dev`
- `/workspace/opencode-src` available in some sessions
