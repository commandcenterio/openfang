# Copilot Instructions

## Branch policy
- `main` is reserved for syncing with the upstream source repository.
- Do not implement feature work directly on `main`.
- `michaels-main` is the long-lived working branch for merged customizations and validated integration work.
- New feature branches should be created from `michaels-main`.

## Working rule
- Before making code changes, confirm you are not on `main`.
- If new feature work is needed, branch from `michaels-main` rather than from `main`.

## Dev runtime workflow
- During active development, do not trust `openfang` on `PATH` unless you have verified exactly where it resolves.
- Prefer running the fork's rebuilt binary directly from this repo, not an installed global executable.
- Use one canonical dev entrypoint consistently: `./scripts/dev-openfang start`.
- If you must use `openfang` from `PATH`, verify it first with `command -v openfang` and confirm it points at this fork's build output.
- After rebuilding, re-run the repo-built binary before investigating runtime behavior so stale executables do not masquerade as code bugs.
