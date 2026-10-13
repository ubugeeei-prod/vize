# A thirty-minute path from a third-party report to a release

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

A fix for a third-party report reaches a published release within thirty
minutes of the report. A longer delivery is handled as an incident. The
release itself therefore has a ten-minute budget from its start to the public
npm version, and it is restructured around four rules.

1. **A release promotes a green `main` commit.** The merge queue has already
   run the full Check suite on that exact commit. Release preflight reads that
   `merge_group` result as evidence and does not run Check again on the same
   bytes.
2. **Exhaustive suites do not sit in the release's serial path.** Real Project
   Matrix, Miri, Fuzz and the Docs build keep running on `main` pushes and on
   their schedules. Preflight requires only that the most recent completed run
   of each on `main` is green. A red run opens an issue immediately and is
   fixed on `main`.
3. **Artifacts are built before the release starts.** Every `main` push
   builds and retains the native binaries for every platform, the WASM
   package, the npm tarballs and the editor extensions. A release downloads
   the artifacts of its commit, smoke-installs them and publishes them.
4. **Public acceptance follows publication.** The `release-*-acceptance`
   workflows still run against every published version, but they verify it
   afterwards instead of gating it.

No check is removed. Each one moves to the place where it runs once per
commit instead of once more per release.

## Reason

The last successful release, v0.443.0
([run 38081199501](https://github.com/ubugeeei-prod/vize/actions/runs/38081199501)),
took 2 hours 37 minutes from dispatch to the GitHub Release:

| Phase                                 | Minutes | What it waits for                                         |
| ------------------------------------- | ------: | --------------------------------------------------------- |
| Exact-commit gates                    |      48 | Docs build 48, full Check 29, Matrix 22, Fuzz 4, Miri 3   |
| Builds (parallel with the gates)      |      22 | Windows native is the longest of 19 build jobs            |
| Version metadata PR and tag promotion |      87 | PR checks and the merge queue for a version-only commit   |
| Publication                           |      19 | A dependency chain of npm packages, crates and extensions |
| GitHub Release                        |       1 |                                                           |

Every gate in the first row re-validates bytes that the merge queue and the
`main` schedules already validated, plus a generated version-only commit. The
third row repeats PR checks and the merge queue for that same version-only
commit. Together they are 135 of the 157 minutes.

## Consequences

`main` carries the version of its next release, so the promoted commit needs
no generated version commit and no metadata pull request in the release path.
The version moves forward in an ordinary pull request after publication.

The pinned source cut, its metadata pull request and the exact-commit gate
dispatch remain available for a release that must be cut from a commit other
than a green `main` head.

## Order of work

Each step is a separate small change, and each is measured against the phase
it removes.

1. Preflight accepts the merge queue's Check result for the promoted commit
   and the latest completed green `main` runs of the exhaustive suites.
2. `main` pushes build and retain the release artifacts, keyed by commit.
3. The release publishes the retained artifacts, and the publication jobs run
   as parallel as their dependencies allow.
4. `main` carries the next version, and the version bump follows publication.
5. Acceptance workflows are dispatched after publication and report failures
   as issues.
