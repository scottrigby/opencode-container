# opencode-container

Run the opencode CLI in a Podman container with glibc compatibility, non-root
user, and per-project session isolation.

## Quick start

```bash
# TUI mode — image builds automatically on first run
./opencode-container

# Web mode
./opencode-container web
# open http://localhost:PORT

# Force rebuild to pull latest upstream image
./opencode-container --build web
```

For convenience:

```bash
ln -s "$PWD/opencode-container" ~/.local/bin/opencode-container
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
4. **Isolates data per project** under hashed paths, respecting the
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

## Manual `podman` commands

If you prefer not to use the wrapper:

```bash
PROJECT_ID=$(echo -n "$PWD" | base64 | tr -d '\n' | tr '+/' '-_' | tr -d '=')
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/opencode/$PROJECT_ID"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/opencode/$PROJECT_ID"
mkdir -p "$DATA_DIR" "$CONFIG_DIR"

# Build image (only needed once, or to update)
podman build -t localhost/opencode-container /path/to/opencode-container

# TUI
podman run -it --rm \
  --label "opencode.project.id=${PROJECT_ID}" \
  -v "$DATA_DIR:/home/opencode/.local/share/opencode:Z" \
  -v "$CONFIG_DIR:/home/opencode/.config/opencode:Z" \
  -v "$PWD:/code:Z" \
  -e OPENCODE_DISABLE_DEFAULT_PLUGINS=true \
  localhost/opencode-container

# Web
podman run -i --rm --init \
  --label "opencode.project.id=${PROJECT_ID}" \
  -p 4096:4096 \
  -v "$DATA_DIR:/home/opencode/.local/share/opencode:Z" \
  -v "$CONFIG_DIR:/home/opencode/.config/opencode:Z" \
  -v "$PWD:/code:Z" \
  -e OPENCODE_DISABLE_DEFAULT_PLUGINS=true \
  localhost/opencode-container web --hostname 0.0.0.0
```

### Find or stop a running container by project

Because names are auto-generated, use the label:

```bash
# Find the container name for this directory
podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}'

# Attach to a TUI session
podman attach $(podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}')

# Stop a web container
podman stop $(podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}')
```

### Decode a project ID back to its path

```bash
echo "PROJECT_ID==" | tr '_-' '/+' | base64 --decode
```

## Data layout

```
~/.local/share/opencode/
├── <hash-a>/              # project A session data (opencode.db, log/, etc.)
├── <hash-b>/              # project B session data
└── ...

~/.config/opencode/
├── <hash-a>/              # project A config (auth.json, node_modules/, etc.)
├── <hash-b>/              # project B config
└── ...
```

Each project is fully isolated. You will need to authenticate (add providers)
once per project. To share auth between projects, copy `auth.json` manually:

```bash
cp ~/.config/opencode/<hash-a>/auth.json ~/.config/opencode/<hash-b>/
```

## Known issues

- **Non-git directories:** If you run this in a directory without `.git`, the
  container auto-initialises an empty git repo at `/code` so opencode treats it
  as a proper project root. Remove `.git` on the host afterward if you don't
  want it.
- **Alpine glibc:** Native `.so` libraries loaded at runtime need glibc symbols.
  The `gcompat` shim resolves this. See [`DESIGN.md`](DESIGN.md) for upstream
  context and why a Debian variant was not used.

## Design & rationale

For detailed reasoning behind every decision (Alpine+gcompat, label
deduplication, git root detection, port scanning, Ctrl+C handling, etc.), see
[`DESIGN.md`](DESIGN.md).

## Upstream

- [Official Dockerfile](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/Dockerfile)
- [Alpine/musl issue](https://github.com/anomalyco/opencode/issues/9246)
- [Debian variant PR](https://github.com/anomalyco/opencode/pull/9560)
