# Release through a PR

Run this from an installed workspace with `gh` authenticated as a repository
maintainer or administrator:

```sh
vp run release minor -y
```

Vize increments the minor version for every `0.x` release, as specified in the
[redundancy guide](../../ubugeeei-redundancy.md#versioning-before-10). The command
also accepts `patch`, `major`, `alpha`, `beta`, `rc`, and `release`. Omit `-y` for
an interactive confirmation.

Choose the protocol before starting:

| Protocol               | Start                           | Source and delivery                                                                                                                                               |
| ---------------------- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Current-main promotion | `vp run release minor -y`       | Refresh the release PR when `main` advances, then atomically promote its validated head and tag.                                                                  |
| Immutable source cut   | `vp run release minor -y --pin` | Freeze the source, reuse successful build artifacts, and deliver generated version metadata through a separate protected PR while ordinary development continues. |

## Immutable source cut

Use `--pin` when the release should retain a fixed source while other PRs keep
merging. The official command performs these steps:

1. Fetch current `main`, prepare the minor version in an isolated worktree, and
   open a draft source PR named `chore(release): vVERSION immutable source cut`.
   Its base commit and generated version-only head become the frozen source.
2. Open the associated `chore(release): integrate vVERSION version metadata`
   PR. Its body identifies the source PR and exact source commit. Keep that
   association intact throughout recovery.
3. Dispatch Release for the frozen source. It builds and smokes the packages
   and editor assets, validates the publish catalog, and runs preflight.
   Full Check, Fuzz replay, Miri, Real Project Matrix and Docs build must all
   succeed at the frozen head. The pinned protocol disables parent evidence
   reuse; required source PR checks also remain mandatory.
4. After required source checks, all five full source gates, every build and
   full preflight succeed, the release operator admits the generated metadata
   PR to the protected merge queue after its own checks pass. The command
   authenticates the actual signed delivery, exact generated metadata, author
   permission, required protected workflows and identical publish catalog.
5. Create the immutable tag at the frozen source head, publish the existing
   Release artifacts, and verify the public result. Changes merged after the
   cut are included in a subsequent release.

The source PR stays draft during qualification and closes after successful
external verification. The metadata PR receives the
ordinary protected merge; the official pinned runner owns source tagging and
publication. Preserve the source branch, source commit, protocol markers and
release run when resuming. A source that lacks the pinned workflow requires a
new prepared cut containing that implementation.

Resume the same source PR with its protocol:

```sh
vp run release --resume SOURCE_PR --pin
```

Resume reuses the authenticated source run and its artifacts and requests
failed-job reruns on that same run. When recovering a cancelled run, verify that
its cancelled promotion and dependent jobs resume successfully. Registry recovery
keeps the same version and tag. The runner uses an operator lock to prevent
concurrent publication of that source.

When a delivered workflow repair must requalify an unchanged source PR, first
verify that its tested merge snapshot contains that repair. GitHub computes
these snapshots asynchronously; a new event timestamp alone cannot establish
which workflow bytes ran. Coordinate any temporary close/reopen with the release
operator while its promotion watchers are stopped, preserving the exact source,
draft state, body, pin, run and artifact catalog. Recover that same Release run
and verify cancelled dependent jobs before resuming publication.

Track completion separately for the signed metadata merge, full source
qualification, tag/source identity, successful publication jobs, public GitHub
Release and assets, every planned npm and crates.io version, and the VS Code
Marketplace extension. Open VSX uses a separate workflow. Issue-specific
installed-package reproductions remain required wherever the issue's acceptance
criteria call for them.

## Current-main promotion

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
Matrix, and Docs build evidence. Main pushes run the fast Check lane. Preflight
dispatches full Check, Miri, and Docs build at the exact release tag SHA and
verifies Check's `test-scripts` job; fast push and PR runs cannot stand in for
them. For a version-only release commit, Fuzz replay and Real Project Matrix
may reuse their parent's evidence. The Benchmark workflow remains available on
demand, but is not dispatched or waited on by release preflight.

Your original worktree stays untouched. Only the command's generated release
branch can be refreshed, using a lease to reject concurrent edits. Another
release changing `main`'s version stops this candidate instead of changing its
proposed version silently. Keep the terminal running, or resume the same PR.

## Ordinary PRs

Release has no `pull_request` or tag-push trigger. Ordinary PRs and main pushes
run the fast Check lane; daily and explicitly dispatched Check runs keep the
full compiler, package, coverage, and integration suites. The additional
release builds and preflight run only when the release command explicitly
dispatches them. No global strict-status-check setting is needed. The atomic
promotion enforces the latest-main requirement for releases.

Docs and playground content changes still build and deploy Pages immediately
after merging to `main`. Code-only changes use the daily Docs build or the
release's exact-SHA build instead of rebuilding the site on every push.

Use the release command to finish source PRs. For current-main promotion, the
runner preserves the validated commit identity through its atomic promotion.
For a pinned cut, deliver its separate metadata PR through the merge queue as
described above.

## Resume after a failure or interruption

```sh
vp run release --resume 1234
```

For an immutable source cut, retain `--pin` when resuming its source PR.

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

Current-main promotion needs permission to push `main` after required checks
pass. Pinned publication uses the protected metadata merge and tags the frozen
source. Both protocols retain repository rules, the existing `release.yml`
trusted-publisher identity and protected environments.
