<!-- SPDX-License-Identifier: Apache-2.0 -->
# Known Issues

Issues discovered while building the container wrapper. Some have local patches;
see [`patches/readme.md`](../patches/readme.md) for how to apply them upstream.

---

## glibc / musl on Alpine

**Symptom:** Native `.so` libraries loaded via Bun FFI fail with glibc symbol
errors:

```
Error relocating ...so: gnu_get_libc_version: symbol not found
```

**Root cause:** The upstream image is Alpine-based (musl libc). Native libraries
compiled against glibc cannot run on musl without a compatibility layer.

**Workaround in container:** The image layers `gcompat` and sets
`LD_PRELOAD=/lib/libgcompat.so.0` at runtime. This keeps us close to upstream
rather than maintaining a forked Debian image.

**Upstream:** [#9246](https://github.com/anomalyco/opencode/issues/9246) ·
[#9560](https://github.com/anomalyco/opencode/pull/9560)

**Rationale:** [`docs/design.md`](design.md#2-alpine--gcompat-instead-of-debian)

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
