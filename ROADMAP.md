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

### 2. Zsh Completion Issues ✅

**Status:** Resolved by Rust rewrite.

**Resolution:** The CLI was rewritten in Rust using [clap](https://crates.io/crates/clap)
with [clap_complete](https://crates.io/crates/clap_complete). Shell completion
scripts for bash, zsh, fish, and PowerShell are generated automatically from the
single source of truth (the `Cli` struct definition). No hand-written completion
scripts, no drift between shells.

### 3. `--env` and `--local-env` Flags ✅

**Status:** Implemented.

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
- Fast path passes `-e VAR=value` and `-e VAR` directly to Podman

**Reference:** [VS Code Remote - Environment Variables](https://code.visualstudio.com/remote/advancedcontainers/environment-variables)

## CLI Framework Rewrite ✅

### 5. Formal CLI Specification and Completion Generation

**Status:** Resolved by Rust rewrite.

**What changed:**
- The bash script with hand-rolled arg parsing and hand-maintained completion
  scripts was replaced with a Rust CLI using `clap` derive macros.
- The CLI spec is now the Rust source code itself (`src/cli.rs`).
- `clap_complete` auto-generates completions for bash, zsh, fish, and PowerShell
  from the same struct definition.
- No more `cli-spec.json`, no more hand-written `completions/`, no more drift.

**New dependencies (build-time only for users):**
- Rust toolchain (for building from source)
- `cargo` / `cargo install`

**User impact:**
- Shell completions are always in sync with the binary.
- Type-safe argument parsing eliminates entire classes of CLI bugs.
- Single static binary distribution (no directory tree of scripts).

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

## Future Ideas

### 6. Embed Containerfiles into the binary

**Idea:** Use Rust's `include_str!` macro to embed `Containerfile.debian` and
`entrypoint.sh` into the binary at compile time. The binary could then
self-extract these files to a cache directory at runtime, making it a fully
self-contained single-file distribution.

**Benefit:** No need to ship the `container/` directory alongside the binary.

### 7. Cross-platform binary releases

**Idea:** Use GitHub Actions to build release binaries for:
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

**Benefit:** Users don't need a Rust toolchain.

### 8. Config file support

**Idea:** Support a `.opencode-container.toml` or `.opencode-container.json` config
file in the project root for per-project defaults (e.g., default port, feature
files, env files).

### 9. Plugin system for container runtimes

**Idea:** Abstract the container runtime so that Podman, Docker, and potentially
other runtimes (nerdctl, containerd) can be used interchangeably.
