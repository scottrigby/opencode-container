# opencode container wrapper

Runs the official opencode CLI inside a Podman container with glibc
compatibility, non-root user, and per-project session + config isolation.

## Problem

### 1. Alpine/musl glibc incompatibility

The official opencode container image is built on Alpine (musl libc). Some native
`.so` libraries loaded at runtime via Bun's FFI layer require glibc symbols
(e.g. `gnu_get_libc_version`, `getauxval`), causing:

```
error: Failed to open library "...so": Error relocating ...so:
  gnu_get_libc_version: symbol not found
```

Setting `OPENCODE_DISABLE_DEFAULT_PLUGINS=true` does not prevent these core
dependencies from loading.

### 2. Security and file browser scope

The official image runs as root, giving full write access to the entire
container filesystem. The file browser starts from the container root (`/`),
exposing the full system instead of just your project. This wrapper fixes both
issues: a non-root user and `WORKDIR /code` so the file browser starts in your
project.

### 3. Session and config management

Running multiple opencode containers without isolation leads to:
- **Database lock conflicts** when multiple instances share the same session data
- **Missing auth** when config isn't persisted between runs
- **Plugin races** when multiple containers write to shared `node_modules`
- **Duplicate containers** accidentally launched from the same project directory

## Fix

This image and wrapper script:

1. Adds [gcompat](https://git.adelielinux.org/adelie/gcompat), a glibc-compatible
   shim layer for Alpine, and sets `LD_PRELOAD` so that dlopen'd libraries can
   resolve the missing symbols
2. Runs as a non-root user (`opencode`) for safety
3. Sets `/code` as the working directory so the file browser starts there
4. Auto-initialises an empty git repo for non-git directories so opencode treats
   the project root as `/code` instead of collapsing it to `/`
5. Isolates **both session data and config** per-project using a base64-encoded
    `$PWD` hash
5. Prevents duplicate containers for the same project automatically using the full
   base64 project ID stored as a Podman label (avoids false positives from
   truncated container names)

## Build

```bash
podman build -t opencode-glibc .
```

This creates a `localhost/opencode-glibc` image in your local Podman registry.

## Usage

### Wrapper script

Use the included `opencode-container` wrapper script for both TUI and web modes:

```bash
# TUI mode
./opencode-container/opencode-container

# Web mode (http://localhost:4096)
./opencode-container/opencode-container web

# Custom port
OPENCODE_PORT=8080 ./opencode-container/opencode-container web
```

For convenience, install globally and create a short alias:

```bash
# Install to PATH
ln -s "$PWD/opencode-container/opencode-container" ~/.local/bin/opencode-container

# Optional short alias (add to shell profile)
alias oc='opencode-container'
```

The wrapper automatically:
- Computes a per-project hash from `$PWD`
- Mounts session data, config, and the current directory
- **Auto-discovers a free port** for web mode (starts at 4096, increments if
  in use)
- **Prevents duplicate containers** for the same project using the full base64
  project ID as a Podman label — if you try to run a second instance from the
  same directory, it will print an error. Because the container name is
  auto-generated, use the label (not the name) to find or attach to running
  containers
- **Handles Ctrl+C gracefully** — stops the container via host shell trap

### Manual commands

If you prefer to run podman directly:

**TUI mode:**

```bash
PROJECT_ID=$(echo -n "$PWD" | base64 | tr -d '\n' | tr '+/' '-_' | tr -d '=')
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/opencode/$PROJECT_ID"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/opencode/$PROJECT_ID"
mkdir -p "$DATA_DIR" "$CONFIG_DIR"

podman run -it --rm \
  --label "opencode.project.id=${PROJECT_ID}" \
  -v "$DATA_DIR:/home/opencode/.local/share/opencode:Z" \
  -v "$CONFIG_DIR:/home/opencode/.config/opencode:Z" \
  -v "$PWD:/code:Z" \
  -e OPENCODE_DISABLE_DEFAULT_PLUGINS=true \
  localhost/opencode-glibc
```

**Web mode:**

```bash
PROJECT_ID=$(echo -n "$PWD" | base64 | tr -d '\n' | tr '+/' '-_' | tr -d '=')
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/opencode/$PROJECT_ID"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/opencode/$PROJECT_ID"
mkdir -p "$DATA_DIR" "$CONFIG_DIR"

podman run -i --rm \
  -p 4096:4096 \
  --label "opencode.project.id=${PROJECT_ID}" \
  -v "$DATA_DIR:/home/opencode/.local/share/opencode:Z" \
  -v "$CONFIG_DIR:/home/opencode/.config/opencode:Z" \
  -v "$PWD:/code:Z" \
  -e OPENCODE_DISABLE_DEFAULT_PLUGINS=true \
  localhost/opencode-glibc web --hostname 0.0.0.0
```

> **Tip:** The `--label "opencode.project.id=${PROJECT_ID}"` flag stores the full
> project hash on the container. Because names are auto-generated, always use the
> label to find or interact with running containers:
> ```bash
> # Find the container name for this project
> podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}'
>
> # Attach to a running TUI session
> podman attach $(podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}')
>
> # Stop a running web container
> podman stop $(podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}')
> ```

### Decode a project ID back to its path

```bash
echo "PROJECT_ID==" | tr '_-' '/+' | base64 --decode
```

## Data layout

```
${XDG_DATA_HOME:-$HOME/.local/share}/opencode/
├── <hash-a>/              # project A session data (opencode.db, log/, etc.)
├── <hash-b>/              # project B session data
└── ...

${XDG_CONFIG_HOME:-$HOME/.config}/opencode/
├── <hash-a>/              # project A config (auth.json, node_modules/, etc.)
├── <hash-b>/              # project B config
└── ...
```

Each project is fully isolated. You will need to authenticate (add providers)
once per project.

### Sharing auth across projects (optional)

If you want to share API keys across projects without re-authenticating:

```bash
# After authenticating in project A, copy auth.json to project B
PROJECT_A=$(echo -n "/path/to/project-a" | base64 | tr -d '\n' | tr '+/' '-_' | tr -d '=')
PROJECT_B=$(echo -n "/path/to/project-b" | base64 | tr -d '\n' | tr '+/' '-_' | tr -d '=')
cp "${XDG_CONFIG_HOME:-$HOME/.config}/opencode/${PROJECT_A}/auth.json" \
   "${XDG_CONFIG_HOME:-$HOME/.config}/opencode/${PROJECT_B}/"
```

## Native compatibility

This setup is safe to use alongside a native macOS opencode install (via the
shell installer). Native opencode stores data directly in
`${XDG_DATA_HOME:-$HOME/.local/share}/opencode/`, while containers use
`${XDG_DATA_HOME:-$HOME/.local/share}/opencode/<hash>/` (per-project
subdirectories). They will not conflict.

## Cache

The container cache (`${XDG_CACHE_HOME:-$HOME/.cache}`) is intentionally
**not** mounted. Each container starts with a fresh cache. This avoids concurrency
issues when multiple team members run containers simultaneously. The only cached
asset (`models.json`) is re-downloaded automatically on first run — a minor
one-time cost per container.

## Non-git directories

opencode's server discovers the project root by searching for `.git`. If none is
found, it defaults the project root to `/`, which causes sessions to be stored
with `directory = "/"` and the web UI to navigate to `/Lw` instead of your
actual project path.

The container entrypoint automatically initialises an empty git repository in
`/code` **only when** the directory is not already inside a git repo. This is
harmless and fully reversible — delete `.git` on the host if you don't want
it. If you already have a git repo (or are inside one), nothing is changed.

## File browser scope

The web UI file browser starts at `/code`. Since the container runs as a non-root
user (UID 1000), system files outside `/code` are read-only.

## Future: Dev Container variant

A devcontainer-based approach may be explored later. The devcontainer ecosystem
(1000+ features) would allow pre-installing project-specific dependencies,
avoiding the "empty container" problem where every new run starts without host
tools. This is deferred until after validating the simple path.

## Upstream

- **Official Dockerfile:** [`packages/opencode/Dockerfile`](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/Dockerfile)
- **Publish workflow:** [`.github/workflows/publish.yml`](https://github.com/anomalyco/opencode/blob/dev/.github/workflows/publish.yml)
- **Related issue (Alpine/musl):** [#9246](https://github.com/anomalyco/opencode/issues/9246)
- **Related PR (Debian variant):** [#9560](https://github.com/anomalyco/opencode/pull/9560)
