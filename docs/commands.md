<!-- SPDX-License-Identifier: Apache-2.0 -->
# NAME

opencode-container — run opencode in a Podman container with per-project isolation

# SYNOPSIS

```
opencode-container [--build] [subcommand]
```

# DESCRIPTION

A wrapper around `podman` that runs the opencode CLI in an Alpine-based
container with glibc compatibility, non-root user (UID 1000), and
per-project data isolation.

The container image builds automatically on first run. No registry or
Docker daemon is required — Podman handles everything rootlessly.

# COMMANDS

If no command is given, the default is `tui`.

`tui`
: Run opencode in terminal UI mode. This is the default when no subcommand
is provided. The container attaches to your terminal for interactive use.

`web`
: Run opencode in web UI mode. Auto-discovers a free port starting at
`4096`, prints the URL, and opens your default browser. The container runs
in the background; press Ctrl+C to stop.

`projects`
: List all project directories that have isolated session data under
`~/.local/share/opencode/`. Each line is the decoded (human-readable) path
of a project that has been opened at least once.

# OPTIONS

`--build`
: Force rebuild the container image before running. This also pulls the
latest upstream base image (`ghcr.io/anomalyco/opencode:latest`).

`--help`
: Print this documentation and exit.

# ENVIRONMENT

`OPENCODE_PORT`
: Override the default web-mode port (`4096`). Only used when the `web`
subcommand is given.

`OPENCODE_NO_GIT_ROOT=1`
: Force mounting the current working directory instead of auto-detecting
and mounting the git repository root. Affects all subcommands that start
a container.

`XDG_DATA_HOME`, `XDG_CONFIG_HOME`
: Base directories for per-project isolation. Defaults are
`~/.local/share/opencode/` and `~/.config/opencode/`.

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
OPENCODE_PORT=5000 opencode-container web
```

Force a rebuild and start web mode:

```bash
opencode-container --build web
```

List all projects that have isolated data:

```bash
opencode-container projects
```

# EXIT STATUS

`0`
: Success.

`1`
: General error (build failure, container start failure, etc.).

# SEE ALSO

[`docs/install.md`](install.md) — requirements, setup, data layout

[`docs/issues.md`](issues.md) — known upstream issues and local patches

[`docs/design.md`](design.md) — architecture rationale

[`docs/manual-commands.md`](manual-commands.md) — equivalent raw Podman commands
