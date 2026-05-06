<!-- SPDX-License-Identifier: Apache-2.0 -->
# Installation

## Requirements

- [Podman](https://podman.io) (macOS via Podman Desktop, or Linux)
- `base64`, `lsof` (macOS) or `ss` (Linux)
- `git` (for auto-detecting repo roots)

## Convenience setup

```bash
ln -s "$PWD/bin/opencode-container" ~/.local/bin/opencode-container
alias oc='opencode-container'   # add to shell profile
```

## Data and config layout

Each project is fully isolated:

```
~/.local/share/opencode/
├── <encoding-a>/          # project A session data (opencode.db, log/, etc.)
├── <encoding-b>/          # project B session data
└── ...

~/.config/opencode/
├── <encoding-a>/          # project A config (auth.json, node_modules/, etc.)
├── <encoding-b>/          # project B config
└── ...
```

You will need to authenticate (add providers) once per project.

To share auth between projects, copy `auth.json` manually:

```bash
cp ~/.config/opencode/<encoding-a>/auth.json ~/.config/opencode/<encoding-b>/
```

## Known issues

- **Non-git directories:** If you run this in a directory without `.git`, the
  container auto-initialises an empty git repo at `/code` so opencode treats it
  as a proper project root. Remove `.git` on the host afterward if you don't
  want it. See [`docs/issues.md`](issues.md#non-git-directories-collapse-into-a-single-global-project)
  for the full upstream context.
- **Alpine glibc:** Native `.so` libraries loaded at runtime need glibc symbols.
  The `gcompat` shim resolves this. See [`docs/issues.md`](issues.md#glibc--musl-on-alpine)
  and [`docs/design.md`](design.md) for upstream context.
