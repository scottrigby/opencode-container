<!-- SPDX-License-Identifier: Apache-2.0 -->
# Manual Podman Commands

If you prefer not to use the `bin/opencode-container` wrapper, run Podman
commands directly.

For the wrapper's usage, subcommands, and options, see [`docs/commands.md`](commands.md).

## One-time setup

Compute your project ID and create data directories:

```bash
PROJECT_ID=$(echo -n "$PWD" | base64 | tr -d '\n' | tr '+/' '-_' | tr -d '=')
DATA_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/opencode/$PROJECT_ID"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/opencode/$PROJECT_ID"
mkdir -p "$DATA_DIR" "$CONFIG_DIR"
```

## Build the image

```bash
podman build -t localhost/opencode-container /path/to/opencode-container/container
```

## Run TUI mode

```bash
podman run -it --rm \
  --label "opencode.project.id=${PROJECT_ID}" \
  -v "$DATA_DIR:/home/opencode/.local/share/opencode:Z" \
  -v "$CONFIG_DIR:/home/opencode/.config/opencode:Z" \
  -v "$PWD:/code:Z" \
  -e OPENCODE_DISABLE_DEFAULT_PLUGINS=true \
  localhost/opencode-container
```

## Run web mode

```bash
podman run -i --rm --init \
  --label "opencode.project.id=${PROJECT_ID}" \
  -p 4096:4096 \
  -v "$DATA_DIR:/home/opencode/.local/share/opencode:Z" \
  -v "$CONFIG_DIR:/home/opencode/.config/opencode:Z" \
  -v "$PWD:/code:Z" \
  -e OPENCODE_DISABLE_DEFAULT_PLUGINS=true \
  localhost/opencode-container web --hostname 0.0.0.0
```

## Find or stop a running container

Because container names are auto-generated, always use the label:

```bash
# Find the container name for this project
podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}'

# Attach to a running TUI session
podman attach $(podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}')

# Stop a running web container
podman stop $(podman ps --filter "label=opencode.project.id=${PROJECT_ID}" --format '{{.Names}}')
```

## Decode a project ID back to its path

```bash
echo "PROJECT_ID==" | tr '_-' '/+' | base64 --decode
```
