# Future Enhancements

This file tracks feature ideas and known issues for post-MVP implementation.

## Environment Variable Support

### 1. `.env` File Support ✅

**Status:** Implemented.

**Feature:** Auto-detect `.env` files and support `--env-file` flag for custom locations.

**Implementation:**
- Auto-detect `.env` in project root (git repository root or current directory)
- Support `--env-file <path>` flag (repeatable)
- For each env file, add to generated `devcontainer.json`:
  ```json
  "runArgs": ["--env-file", "/path/to/.env"]
  ```
- For Podman fast path, pass `--env-file` directly (supports multiple)

### 2. Zsh Completion Issues

**Status:** Known issue. Zsh completion has several bugs:

- `opencode-container completion` only completes `--zsh`, not `--bash`
- After choosing a subcommand (e.g., `tui`), global options are not offered
- `opencode-container web` should complete `--port` and global options

**Root cause:** The `compadd` logic and context detection in the zsh completion
function need review. The bash completion works correctly; zsh needs similar
filtering logic.

### 3. `--env` and `--local-env` Flags

**Feature:** Pass environment variables directly or from host environment.

**Implementation:**
- `--env VAR=value` → sets literal value in `containerEnv`:
  ```json
  "containerEnv": {
    "VAR_ONE": "value-one",
    "VAR_TWO": "value-two"
  }
  ```
- `--local-env VAR` → sets value from host environment using `${localEnv:VAR}`:
  ```json
  "containerEnv": {
    "VAR": "${localEnv:VAR}"
  }
  ```
- Merge all into `containerEnv` key in generated `devcontainer.json`

**Reference:** [VS Code Remote - Environment Variables](https://code.visualstudio.com/remote/advancedcontainers/environment-variables)

## Ctrl+C / Ctrl+D Confirmation Prompt

### 4. Session Close Confirmation

**Feature:** Capture Ctrl+C or Ctrl+D and prompt user before closing session.

**Problem:** This is a major known annoyance with opencode — accidental keypresses terminate sessions unexpectedly.

**Implementation ideas:**
- Web mode: Host shell trap can `read -p "Quit? [y/N] "` before `podman stop`
- TUI mode: Harder — terminal is occupied by TUI process, so a host-side prompt would compete for the same PTY. Would need to either:
  - Send a signal into the container and let the TUI handle it
  - Use a separate notification mechanism

**Upstream issue:** [anomalyco/opencode#10975](https://github.com/anomalyco/opencode/issues/10975)

**Status:** Tracked upstream. Add to `docs/issues.md` when implemented.
