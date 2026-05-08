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

## CLI Spec and Completion Generation

### 5. Formal CLI Specification

**Status:** `cli-spec.json` created as documentation. Generator not yet implemented.

**Idea:** Represent the CLI structure (commands, options, types, descriptions) in a
machine-readable format (JSON/YAML) and use it to auto-generate shell completion
scripts, man pages, and help text.

**Why:** Currently the CLI structure is defined ad-hoc in `bin/opencode-container` with
hand-written completion scripts for bash and zsh. This leads to:
- Completions getting out of sync with the actual parser
- Bugs in one shell's completion that don't exist in the other
- No single source of truth for the CLI structure

**Approaches evaluated:**

1. **Hand-written generator script** — Parse `cli-spec.json` with `jq`, emit bash/zsh
   completion scripts. Doable but error-prone (as demonstrated by initial attempt).
   Would need proper testing.

2. **`argbash`** — Bash script generator that takes annotated templates and produces
   scripts with proper `getopts` parsing + completions. Would require rewriting
   `bin/opencode-container` as an argbash template. Adds build step and dependency.

3. **Switch to language with CLI framework** — Rewrite wrapper in Go (cobra), Rust
   (clap), or Python (click) which all auto-generate completions from struct/decorator
   definitions. Major architectural change, not justified for a thin wrapper.

**Recommendation:** For now, keep hand-written completions but use `cli-spec.json` as
the reference when adding new flags. In the future, evaluate `argbash` or a custom
`jq`-based generator that produces both bash and zsh scripts from the spec.

**Related:** See §2 (Zsh Completion Issues) for immediate bugs to fix.

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
