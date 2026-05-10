<!-- SPDX-License-Identifier: Apache-2.0 -->
# Installation

## Requirements

- [Podman](https://podman.io) (macOS via Podman Desktop, or Linux) or Docker
- `git` (for auto-detecting repo roots)

**Optional:**
- Node.js + `npx` — only needed when using `--feature-file` with the devcontainer CLI

## Installing the binary

### Option 1: GitHub Releases (recommended for users)

Download the latest release for your platform from the [releases page](https://github.com/scottrigby/opencode-container/releases).

```bash
# macOS / Linux — extract and place on PATH
tar -xzf opencode-container-x86_64-unknown-linux-gnu.tar.gz
mv opencode-container ~/.local/bin/
```

### Option 2: Install from source with Cargo

Requires [Rust](https://rustup.rs/) 1.70+.

```bash
cargo install --git https://github.com/scottrigby/opencode-container
```

This compiles from source and installs the binary to `~/.cargo/bin/`.

### Option 3: Build locally from source

```bash
git clone https://github.com/scottrigby/opencode-container
cd opencode-container
cargo build --release
# Binary is at ./target/release/opencode-container
cp target/release/opencode-container ~/.local/bin/
```

## Tab completion

Shell completion scripts are generated on demand from the CLI definition (via [clap_complete](https://crates.io/crates/clap_complete)). They are always in sync with the binary — no manual maintenance needed.

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

### Fish

```bash
opencode-container completion --fish > ~/.config/fish/completions/opencode-container.fish
```

### PowerShell

```powershell
opencode-container completion --powershell > _opencode-container.ps1
. ./_opencode-container.ps1
```

## Data and config layout

Each project is fully isolated. Data, config, and cache are always stored in
separate subdirectories, even on platforms where `dirs::data_dir()` and
`dirs::config_dir()` return the same base path (e.g., macOS):

### Linux

```
~/.local/share/opencode/data/
├── <encoding-a>/          # project A session data (opencode.db, log/, etc.)
├── <encoding-b>/          # project B session data
└── ...

~/.config/opencode/config/
├── <encoding-a>/          # project A config (auth.json, node_modules/, etc.)
├── <encoding-b>/          # project B config
└── ...

~/.cache/opencode/cache/
├── <encoding-a>/          # generated devcontainer.json, etc.
├── <encoding-b>/
└── ...
```

### macOS

```
~/Library/Application Support/opencode/data/
├── <encoding-a>/          # project A session data
├── <encoding-b>/          # project B session data
└── ...

~/Library/Application Support/opencode/config/
├── <encoding-a>/          # project A config
├── <encoding-b>/          # project B config
└── ...

~/Library/Caches/opencode/cache/
├── <encoding-a>/          # generated devcontainer.json, etc.
├── <encoding-b>/
└── ...
```

### Windows

```
%APPDATA%\opencode\data\
├── <encoding-a>\
├── <encoding-b>\
└── ...

%APPDATA%\opencode\config\
├── <encoding-a>\
├── <encoding-b>\
└── ...

%LOCALAPPDATA%\opencode\cache\
├── <encoding-a>\
├── <encoding-b>\
└── ...
```

`<encoding>` is the base64url encoding of the absolute project path.

You will need to authenticate (add providers) once per project.

To share auth between projects, copy `auth.json` manually:

```bash
# Linux
cp ~/.config/opencode/config/<encoding-a>/auth.json ~/.config/opencode/config/<encoding-b>/

# macOS
cp ~/Library/Application\ Support/opencode/config/<encoding-a>/auth.json \
   ~/Library/Application\ Support/opencode/config/<encoding-b>/
```
