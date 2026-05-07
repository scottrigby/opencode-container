<!-- SPDX-License-Identifier: Apache-2.0 -->
# NAME

opencode-container — run opencode in a Podman container with per-project isolation

# SYNOPSIS

```
opencode-container [global-options] [subcommand] [subcommand-options]
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

# GLOBAL OPTIONS

`-b`, `--build`
: Force rebuild the container image before running.

`--feature-file` *PATH*
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

`--no-git-root`
: Mount the current working directory instead of auto-detecting and
mounting the git repository root.

`--no-git-init`
: Do not auto-initialise an empty git repository in non-git directories.
By default the container runs `git init` when no `.git` is found so that
opencode treats the directory as a proper project root.

`-h`, `--help`
: Print usage information and exit. This is handled by the wrapper script;
it will never be forwarded to the opencode binary inside the container.

# COMMANDS

If no command is given, the default is `tui`.

`tui`
: Run opencode in terminal UI mode. The container attaches to your
terminal for interactive use. Any remaining arguments after the
subcommand are passed through to opencode.

`web`
: Run opencode in web UI mode. Auto-discovers a free port starting at
`4096`, prints the URL, and opens your default browser. The container
runs in the background; press Ctrl+C to stop.

`projects`
: List all project directories that have isolated session data under
`~/.local/share/opencode/`. Each line is the decoded (human-readable)
path of a project that has been opened at least once.

# COMMAND OPTIONS

## web

`-p`, `--port` *PORT*
: Override the default port (`4096`). If the port is in use, the next
available port is chosen automatically and a message is printed to
stderr.

`--no-open`
: Do not automatically open the browser. The URL is still printed to
stdout.

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

Run web mode on a custom port:

```bash
opencode-container web --port 5000
```

Force a rebuild and start web mode:

```bash
opencode-container --build web
```

Mount the current subdirectory (not the git root) and do not auto-init git:

```bash
opencode-container --no-git-root --no-git-init web --port 5000
```

Run with devcontainer features from a JSON file:

```bash
opencode-container --feature-file ./features.json tui
```

Run web mode with features on a custom port:

```bash
opencode-container --feature-file ./features.json web --port 5000
```

See `tests/testdata/README.md` for ready-made feature files and a full manual
E2E test checklist.

List all projects that have isolated data:

```bash
opencode-container projects
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
