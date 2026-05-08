<!-- SPDX-License-Identifier: Apache-2.0 -->
# NAME

opencode-container — run opencode in a Podman container with per-project isolation

# SYNOPSIS

```
opencode-container [options] [--] [opencode-args...]
opencode-container <command> [command-options]
```

# DESCRIPTION

A wrapper around `podman` (or the devcontainer CLI) that runs the opencode
CLI in a Debian-based (`node:22-slim`) container with per-project data
isolation.

When `--feature-file` is given, the wrapper uses the devcontainer CLI to layer
devcontainer features (Node, Python, Go, etc.) on top of the base image.
Without `--feature-file`, the container starts directly via Podman for a fast
path.

The container image builds automatically on first run. No registry or
Docker daemon is required — Podman handles everything rootlessly.

# OPTIONS

`-h`, `--help`
: Print usage information and exit. This is handled by the wrapper script;
it will never be forwarded to the opencode binary inside the container.

`-b`, `--build`
: Force rebuild the container image before running.

`-f`, `--feature-file` *PATH*
: Merge the `.features` object from a JSON file into the generated
`devcontainer.json`. Repeatable; later files override on key collision.
Requires `npx` (which will auto-install `@devcontainers/cli` and
`node-jq` on first use). When this flag is used, the wrapper uses the
devcontainer CLI instead of direct `podman run`.

`--env-file` *PATH*
: Pass an environment file to the container. Repeatable. The wrapper
auto-detects `.env` in the project root (git repository root or current
directory) and passes it automatically. Additional files can be specified
manually. In devcontainer mode, adds `--env-file` to `runArgs`. In fast
path, passes `--env-file` directly to Podman.

`-e`, `--env` *VAR=value*
: Set an environment variable in the container with a literal value.
Repeatable. In devcontainer mode, adds the variable to `containerEnv`.
In fast path, passes `-e VAR=value` directly to Podman.

`--local-env` *VAR*
: Pass an environment variable from the host environment into the container.
Repeatable. In devcontainer mode, uses `${localEnv:VAR}` syntax so the
devcontainer CLI resolves the value from the host at container start time.
In fast path, passes `-e VAR` directly to Podman, which inherits the
host's current value.

`--no-open`
: Do not automatically open the browser when running the web UI. Only
applies when the first opencode argument is `web` and no custom
`--hostname` is set by the user. The URL is still printed to stdout.

# WEB UI MODE

The wrapper does not have a dedicated `--web` flag. Instead, it detects
web mode when the first argument after `--` (or the first non-option
argument) is `web`:

```bash
opencode-container -- web --port 5000 --pure
```

When web mode is detected, the wrapper:

1. Injects `--hostname 0.0.0.0` if the user did not set `--hostname`
   (required for the host to reach the container).
2. Injects `--port 4096` if the user did not set `--port`.
3. Forwards the port from the container to the host.
4. Waits for the HTTP server to respond.
5. Prints the URL and auto-opens the browser (unless `--no-open` was
   passed or a custom `--hostname` was set).

If the user sets a custom `--hostname`, the wrapper skips browser
auto-open because the host likely cannot reach a non-default hostname.

If opencode web subcommands or flags that do not start a server are
detected (e.g. `help`, `--help`, `--version`), the wrapper skips the
port wait and browser open entirely.
: Mount the current working directory instead of auto-detecting and
mounting the git repository root.

`--no-git-init`
: Do not auto-initialise an empty git repository in non-git directories.
By default the container runs `git init` when no `.git` is found so that
opencode treats the directory as a proper project root.

# COMMANDS

`projects`
: List all project directories that have isolated session data under
`~/.local/share/opencode/`. Each line is the decoded (human-readable)
path of a project that has been opened at least once.

`completion`
: Generate shell completion scripts. Use `--bash` or `--zsh` to select
a shell.

# ENVIRONMENT

`XDG_DATA_HOME`, `XDG_CONFIG_HOME`
: Base directories for per-project isolation. Defaults are
`~/.local/share/opencode/` and `~/.config/opencode/`.

`XDG_CACHE_HOME`
: Base directory for generated devcontainer configs when using `--feature-file`.
Defaults to `~/.cache/opencode/`.

# FILES

```
~/.local/share/opencode/<encoding>/
    Session data: opencode.db, log/, etc.

~/.config/opencode/<encoding>/
    Project config: auth.json, node_modules/, etc.
```

`<encoding>` is the base64url encoding of the absolute project path.

# EXAMPLES

Run TUI mode in the current directory:

```bash
opencode-container
```

Run web UI mode (auto-opens browser, forwards port 4096):

```bash
opencode-container -- web
```

Run web UI on a custom port:

```bash
opencode-container -- web --port 5000
```

Run web UI with a custom hostname (skips browser auto-open):

```bash
opencode-container -- web --hostname 127.0.0.1 --port 5000
```

Force a rebuild and start web UI:

```bash
opencode-container -b -- web
```

Mount the current subdirectory (not the git root) and do not auto-init git:

```bash
opencode-container --no-git-root --no-git-init -- web
```

Run with devcontainer features from a JSON file:

```bash
opencode-container -f ./features.json
```

Run web UI with features on a custom port:

```bash
opencode-container -f ./features.json -- web --port 5000
```

See `tests/testdata/README.md` for ready-made feature files and a full manual
E2E test checklist.

List all projects that have isolated data:

```bash
opencode-container projects
```

Generate bash completion script:

```bash
opencode-container completion --bash > /etc/bash_completion.d/opencode-container
```

# EXIT STATUS

`0`
: Success.

`1`
: General error (unknown option, build failure, container start failure,
etc.).

# SEE ALSO

[`docs/install.md`](install.md) — requirements, setup, data layout

[`docs/issues.md`](issues.md) — known upstream issues and local patches

[`docs/design.md`](design.md) — architecture rationale

[`docs/manual-process.md`](manual-process.md) — equivalent raw Podman commands
