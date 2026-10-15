# Release qualification stops at Check

A release publishes when the exact-SHA Check workflow is green. Fuzz, Miri,
Real Project Matrix, and Docs build keep running on their own schedules and on
main. They are not dispatched for a cut, and a red run of one of them does not
block the tag.

Check still has to finish its required jobs, including `test-scripts`, the
JavaScript package jobs, and `cargo-semver-checks`. A failed required Check
job still fails the cut.

Docs build no longer walks the reviewed browser routes or Open Graph images.
The site build and its component-preview evidence remain. The route scripts
stay in the repository for local use. The wall-time benchmark can still time
navigation capture; that benchmark is not a release gate.

The release preflight command is
`node tools/support/compat/github/release-preflight.mjs`. A pinned candidate
passes `RELEASE_PR_NUMBER` and `RELEASE_TAG_NAME` on the release branch. The
release commit itself is not on main yet. Preflight checks that its parent is
still on main's first-parent history, that main still has that parent's
version, and that the tag does not exist yet.
The package build sequencer is `node tools/support/github/run-many.ts`.
Node's stable type stripping runs that TypeScript file. Package builds still
install `rust-script` because tasks such as `generate:rule-types` call the
remaining `tools/commands/**/*.rs` entrypoints.

This replaces the earlier release contract that required all five workflows
and the browser docs captures before a tag could exist.
