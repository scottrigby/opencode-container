<!-- SPDX-License-Identifier: Apache-2.0 -->
# Commands

## Usage

```
opencode-container [flags] [subcommand]
```

## Subcommands

| Subcommand | Description |
|------------|-------------|
| *(none)* | Run opencode in **TUI mode** (default) |
| `web` | Run opencode in **Web UI mode** — auto-discovers a free port starting at `4096` and opens the browser |
| `projects` | List all project directories that have isolated data in `~/.local/share/opencode/` |

## Flags

| Flag | Description |
|------|-------------|
| `--build` | Force rebuild the container image (also pulls the latest upstream image) |

## Environment variables

| Variable | Applies to | Description |
|----------|------------|-------------|
| `OPENCODE_PORT` | `web` | Override the default port (`4096`) |
| `OPENCODE_NO_GIT_ROOT=1` | all | Force mounting the current subdirectory instead of the git repo root |

## Examples

```bash
# TUI mode
opencode-container

# Web mode on custom port
OPENCODE_PORT=5000 opencode-container web

# Force rebuild, then web mode
opencode-container --build web

# List projects
opencode-container projects
```

For requirements and setup, see [`docs/install.md`](install.md).
For equivalent manual Podman commands, see [`docs/manual-commands.md`](manual-commands.md).
