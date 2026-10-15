# Release qualification stops at Check

A release publishes when its own builds are green. It does not dispatch or
wait for Check, Fuzz, Miri, Real Project Matrix, or Docs build. Those keep
their own schedules. After the release artifacts are ready, the operator
enables auto-merge on the version integration. The promotion job waits at
most 15 minutes for that merge and the tag. It does not wait for a second
merge-queue run of Check, n8n, Musea, or Nuxt.

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
version, and that the tag does not exist yet. Check is dispatched on
`release/<tag>`, because that tag ref does not exist until publication.
The package build sequencer is `node tools/support/github/run-many.ts`.
Node's stable type stripping runs that TypeScript file. Package builds still
install `rust-script` because tasks such as `generate:rule-types` call the
remaining `tools/commands/**/*.rs` entrypoints.

This replaces the earlier release contract that required all five workflows
and the browser docs captures before a tag could exist.
