<!-- SPDX-License-Identifier: Apache-2.0 -->
# Upstream Patches

These patches were applied to the upstream opencode source tree but are **not
required** for daily use of `opencode-container`. The `entrypoint.sh` + host
git detection handle the pre-built image correctly.

They are included here as a reminder to contribute them upstream. If accepted,
the `entrypoint.sh` `git init` workaround for non-git directories could be
removed.

## Patches

| Patch | File | What it fixes |
|-------|------|---------------|
| [`0001-project-fromdirectory-use-directory-for-non-git-work.patch`](0001-project-fromdirectory-use-directory-for-non-git-work.patch) | `packages/opencode/src/project/project.ts` | When no `.git` is found, use the actual `directory` instead of `"/"` as the worktree. Prevents collapsing all non-git directories into a single global project. |
| [`0002-tui-plugin-runtime-fix-vcs-inference-for-non-git-dirs.patch`](0002-tui-plugin-runtime-fix-vcs-inference-for-non-git-dirs.patch) | `packages/opencode/src/cli/cmd/tui/plugin/runtime.ts` | Infer `vcs` from actual sync state instead of `worktree !== "/"`. Prevents misreporting non-git directories as git. |

## Contributing upstream

These patches should be contributed to the [opencode upstream repository](https://github.com/anomalyco/opencode) as pull requests. Track progress here:

- [ ] `project.fromDirectory` non-git fallback — `packages/opencode/src/project/project.ts`
- [ ] TUI plugin runtime `vcs` inference — `packages/opencode/src/cli/cmd/tui/plugin/runtime.ts`

To apply them locally for testing, run from the root of a cloned `opencode` repository:

```bash
git apply /path/to/opencode-container/patches/0001-project-fromdirectory-use-directory-for-non-git-work.patch
git apply /path/to/opencode-container/patches/0002-tui-plugin-runtime-fix-vcs-inference-for-non-git-dirs.patch
```
