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
exposing the full system instead of just your project.

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
4. Isolates **both session data and config** per-project using a base64-encoded
   `$PWD` hash
5. Prevents duplicate containers for the same project automatically

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
- **Prevents duplicate containers** for the same project — if you try to run a
  second instance from the same directory, it will print an error and the
  `podman attach` command to reconnect
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
  -v "$DATA_DIR:/home/opencode/.local/share/opencode:Z" \
  -v "$CONFIG_DIR:/home/opencode/.config/opencode:Z" \
  -v "$PWD:/code:Z" \
  -e OPENCODE_DISABLE_DEFAULT_PLUGINS=true \
  localhost/opencode-glibc web --hostname 0.0.0.0
```

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

## File browser scope

The web TUI file browser starts at `/code` but can still navigate to `/`.
Since the container runs as a non-root user (UID 1000), system files are
read-only and cannot be modified. The file browser does not have a built-in
option to restrict navigation to a single directory.

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
