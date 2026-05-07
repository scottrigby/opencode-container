<!-- SPDX-License-Identifier: Apache-2.0 -->
# Known Issues

Issues discovered while building the container wrapper. Some have local patches;
see [`patches/readme.md`](../patches/readme.md) for how to apply them upstream.

---

## glibc / musl incompatibility (resolved by Debian base)

**Symptom:** Native `.so` libraries loaded via Bun FFI fail with glibc symbol
errors:

```
Error relocating ...so: gnu_get_libc_version: symbol not found
```

**Root cause:** The upstream image is Alpine-based (musl libc). Native libraries
compiled against glibc cannot run on musl without a compatibility layer.

**Resolution:** The base image is Debian (`node:22-slim`), which uses glibc
natively. Devcontainer features also require glibc.

**Upstream:** [#9246](https://github.com/anomalyco/opencode/issues/9246) ·
[#9560](https://github.com/anomalyco/opencode/pull/9560)

**Rationale:** [`docs/design.md`](design.md#2-debian-base-image-node22-slim)

---

## Non-git directories collapse into a single global project

**Symptom:** Running opencode in multiple non-git directories all share the same
session data and URL path (`/`).

**Root cause:** `project.fromDirectory` returns `worktree: "/"` when no `.git`
is found, so every non-git directory is treated as the same project.

**Workaround in container:** `entrypoint.sh` auto-initialises an empty git repo
when none exists. This is reversible: `rm -rf .git` after the session.

**Patch:** [`patches/0001-project-fromdirectory-use-directory-for-non-git-work.patch`](../patches/0001-project-fromdirectory-use-directory-for-non-git-work.patch)

---

## TUI plugin misreports non-git directories as git

**Symptom:** The TUI shows "Create Git repository" even in directories that
are not git repositories.

**Root cause:** VCS inference uses `worktree !== "/"` as a proxy for "is git",
which is wrong after the non-git fallback patch.

**Patch:** [`patches/0002-tui-plugin-runtime-fix-vcs-inference-for-non-git-dirs.patch`](../patches/0002-tui-plugin-runtime-fix-vcs-inference-for-non-git-dirs.patch)

---

## `entrypoint.sh` shebang must be `#!/bin/bash`

**Symptom:** The container fails to start with syntax errors in `entrypoint.sh`:

```
/entrypoint.sh: 12: [[: not found
```

**Root cause:** `entrypoint.sh` was changed from `#!/bin/sh` to `#!/bin/bash`
because it uses bash-specific syntax (`[[ ]]`, `|| true`). The Debian base image
has `/bin/bash`, but the shebang must match.

**Fix:** Ensure `entrypoint.sh` starts with `#!/bin/bash`.

---

## Devcontainer `node` feature conflicts with `node:22-slim` base image

**Symptom:** When using `--feature-file` with `ghcr.io/devcontainers/features/node:1`,
the devcontainer CLI fails with:

```
node: error while loading shared libraries: libatomic.so.1: cannot open shared object file: No such file or directory
ERROR: Feature "Node.js (via nvm), yarn and pnpm." ... failed to install!
```

**Root cause:** The `node` feature installs Node.js via nvm on top of the base
image. Our base image is `node:22-slim`, which **already includes Node.js**.
Installing a second Node.js via nvm causes library conflicts — the nvm-installed
version expects `libatomic.so.1` which is not present in the slim Debian base.

**Workaround:** Do **not** use the `node` devcontainer feature with this base
image. Use features that add tools the base image *doesn't* already have,
e.g. `common-utils:2`, `go:1`, or `python:1`.

**Note:** This incompatibility is not documented in the upstream
[devcontainers/features node README](https://github.com/devcontainers/features/tree/main/src/node).
The feature assumes a clean Debian/Ubuntu base without Node.js pre-installed.

---

## Terminal WASM fails to load on page reload in web mode

**Symptom:** Opening a terminal in web mode works on first load, but reloading
the page (or opening a new session tab) shows:

```
WebAssembly.compile(): expected magic word 00 61 73 6d, found 3c 21 64 6f @+0
```

**Root cause:** `ghostty-web`'s `Ghostty.load()` uses a relative fallback path
`./ghostty-vt.wasm` when called without arguments. In a single-page app served
from non-root routes (e.g. `/L2NvZGU/session/...`), that relative path resolves
to a route that does not exist. The server returns the SPA shell HTML instead of
the WASM binary. The magic-word error is the browser reading HTML bytes
(`0x3c21646f` = `<!do`) as WASM.

**Fix:** Pass an absolute URL to `Ghostty.load()` by importing the `.wasm` asset
through Vite's `?url` mechanism. Vite emits the file to `dist/` with a hashed
absolute URL, bypassing all relative-path fallbacks.

**Patch:** [`patches/0003-ghostty-web-pass-absolute-wasm-url-for-non-root-spa-routes.patch`](../patches/0003-ghostty-web-pass-absolute-wasm-url-for-non-root-spa-routes.patch)

**Note:** This only manifests when the embedded UI is served through the
opencode backend from non-root URL paths. It does not affect the production
app hosted at the domain root.
