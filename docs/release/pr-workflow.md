# Release through a PR

Run this from an installed workspace with `gh` authenticated as a repository
maintainer or administrator:

```sh
vp run release patch -y
```

The other supported bumps are `minor`, `major`, `alpha`, `beta`, `rc`, and
`release`. Omit `-y` for an interactive confirmation.

The command performs the whole release:

1. Fetch current `main`, prepare aligned versions in a dedicated worktree, and
   open `chore(release): vVERSION` from `release/vVERSION` into `main`.
2. Dispatch Release for that exact PR commit. Build CLI binaries, native and
   WASM packages, npm packages, and editor extensions; smoke the packages and
   validate the existing release evidence and crates.io publish plan.
3. Wait for release validation and every required PR check. Verify the author's
   current `maintain` or `admin` role again. If `main` advanced, regenerate the
   version commit on its new tip and repeat validation.
4. Fast-forward `main` to the validated commit and create its annotated tag in
   one **non-force atomic Git push**. A concurrent change to `main` rejects the
   whole transaction, including the tag. GitHub records the PR as merged when
   its head is included in `main`.
5. Publish the artifacts already built by that Release run. Wait until all
   publication jobs succeed and the GitHub Release is publicly available.

Release preflight requires successful Check, Fuzz replay, Miri, Real Project
Matrix, and Docs build evidence. For a version-only release commit it accepts
the parent's main push evidence, including Check's `test-scripts` job. The
Benchmark workflow remains available on demand, but is not dispatched or
waited on by release preflight.

Your original worktree stays untouched. Only the command's generated release
branch can be refreshed, using a lease to reject concurrent edits. Another
release changing `main`'s version stops this candidate instead of changing its
proposed version silently. Keep the terminal running, or resume the same PR.

## Ordinary PRs

Release has no `pull_request` or tag-push trigger. Ordinary PRs keep their usual
checks; the additional release builds and preflight run only when the release
command explicitly dispatches them. No global strict-status-check setting is
needed. The atomic promotion enforces the latest-main requirement for releases.

Use the release command to finish release PRs; manually squash-merging one does
not preserve the validated commit identity and will not trigger publication.

## Resume after a failure or interruption

```sh
vp run release --resume 1234
```

Before promotion, failed validation leaves an open PR and no remote tag. Resume
reuses the matching run, reruns failed jobs, and refreshes against current main
when needed. The failure message includes the preserved worktree and run URL.

Registry outages or credentials can still interrupt publication after promotion:
Git and external registries cannot share one atomic transaction. Resume reruns
failed jobs in the original run and reuses its artifacts. Already visible exact
package versions are handled by the existing idempotent publishers. Never move
or replace the tag, and do not start another version merely to retry publication.

For a first npm publication requiring owner configuration, follow
[Trusted Publishing Recovery](./trusted-publishing-recovery.md) and
[Supply Chain](./supply-chain.md), then resume the same PR. The existing manual
crate handoff remains available by dispatching Release with an empty `release_pr`
and the already-published GitHub Release tag.

The command needs permission to push `main` after required checks pass. It does
not bypass repository rules or force-update `main`. Publication continues to use
the existing `release.yml` trusted-publisher identity and protected environments.
