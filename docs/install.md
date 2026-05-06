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

## Tab completion

### Bash

```bash
# Current session:
source <(opencode-container completion --bash)

# Every new session:
opencode-container completion --bash > ~/.local/share/bash-completion/completions/opencode-container
```

### Zsh (macOS default)

```bash
# Current session:
source <(opencode-container completion --zsh)

# Every new session:
opencode-container completion --zsh > "${fpath[1]}/_opencode-container"
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
