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
3. Qualify the exact frozen source H with a full `Check` workflow using the
   official `release-qualification` profile. Its seven full-source report/build
   jobs and twelve crate semver checks must succeed before Release is dispatched.
   Release then builds and smokes the shipping packages and editor assets,
   validates the publish catalog, and runs preflight using that source evidence.
   Fuzz, Miri, Real Project Matrix and Docs build keep their own schedules or
   manual dispatches; they are not additional release gates.
4. Once Release is ready and the source PR's required checks pass, the official
   operator requests a squash merge of the metadata PR through the GitHub API.
   GitHub applies the repository's existing merge rules to that request. The
   runner authenticates the actual signed delivery, exact generated metadata, author
   permission and identical publish catalog before tagging H. The current driver
   has no separate protected full-workflow release wait.
5. Create the immutable tag at the frozen source head, publish the existing
   Release artifacts, and verify the public result. Changes merged after the
   cut are included in a subsequent release.

The source PR stays draft during qualification and closes after successful
external verification. The metadata PR carries the generated version changes;
the official pinned runner owns its admission, signed-delivery verification,
source tagging and publication. Preserve the source branch, source commit,
protocol markers and release run when resuming. A source that lacks the pinned workflow requires a
new prepared cut containing that implementation.

### Keep the source branch at its frozen head

The source PR is a receipt, so do not use GitHub's **Update branch**, merge
`main` into it, rebase it, or regenerate its version commit. Only the separate
metadata integration PR follows moving `main`. Even a merge that preserves the
release version changes the source head and breaks the receipt's identity.

If the branch moves, stop automated resume and preserve the changed commit and
its parents before attempting recovery. Record the original C/H, current PR and
branch heads, durable `release-pin/<tag>` marker, annotated tag, original Release
run and operator lease. A matching public tag or successful build does not make
the changed branch a qualified replacement. The existing
[source-head guard](../../tools/support/release/pr_pin_lock.rs) and
[recovery regression](../../tools/support/release/pr_pin_recovery_tests.rs)
deliberately refuse to rewrite the source during ordinary resume.

Review any proposed restoration as a separate operation: retain a backup ref,
name the exact expected old head and frozen target, and use an exact lease that
rejects intervening edits. Afterward, recheck the PR, source branch, durable pin,
tag and original run before resuming the same source. Preserve the tag, public
artifacts, receipt markers and original consumer policies throughout recovery.
Do not start a new version merely to bypass incomplete publication of this one.
The [unpublished-cut retirement procedure](./2026-10-10-retire-unpublished-pin.md)
does not apply once a tag, GitHub Release or package version exists.

Resume the same source PR with its protocol:

```sh
vp run release --resume SOURCE_PR --pin
```

When the local execution environment cannot reach the genuine registry or
GitHub, dispatch **Release Operator** on `main`. Choose `start` for the same
official minor command or `resume` with the original source PR number. Its
protected `release-operator` environment requires the dispatching maintainer's
existing user credential as `RELEASE_OPERATOR_TOKEN`, with an exact `main`
deployment branch policy. The entry installs the frozen JavaScript dependency
graph before invoking the public command.
If main advanced after the runner installed its graph, the official starter
refuses that cut before preparation; redispatch from fresh main. A hosted resume
installs the original H dependency graph in its isolated worktree.

The hosted operator has a four-hour cooperative watch budget inside its
six-hour runner limit. It returns normally on expiration so the original exact
operator lease cleanup runs; preserve the same source, pin and Release run and
resume that source PR. Do not cancel a live operator to bypass its ownership
receipt. Blocking child operations and ambiguous cleanup retain their existing
recovery semantics. All original qualification, protected metadata admission,
tag custody and public verification remain required. See the
[hosted operator decision](./2026-10-09-hosted-release-operator.md).

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

Release qualification follows the
[2026-10-15 scope decision](../davinci/decisions/2026-10-15-release-qualification-scope.md):
the release's own builds, preflight and required Check jobs must pass. Check
still includes its required tooling, JavaScript package and semver checks.
Fuzz replay, Miri, Real Project Matrix, Docs build and Benchmark retain their
independent schedules and manual entry points; release does not dispatch or
wait for them. This replaces the earlier five-workflow qualification contract.

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
For a pinned cut, let the official runner admit its separate metadata PR and
verify the actual signed delivery before tagging the frozen source.

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
