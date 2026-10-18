# Release qualification requires Check at the immutable source

The original 2026-10-15 statement that a release does not wait for Check is
superseded by the current exact-source qualification contract. The operator
starts or finds Check at frozen source H on its release branch before creating
publishing run R. R's artifact builds may overlap that pending qualification;
publication and official integration success require the latest matching Check
to finish with whole-workflow success and every source-bound required job
successful. Legacy sources retain seven full jobs plus twelve published-crate
SemVer checks. The current reusable full profile adds genuine Nuxt qualification
as an eighth full role. Failed, cancelled, skipped or pending required evidence
blocks publication and integration; a pending run is never qualified.
Check evidence from the cut C, integration merge M, or another source is not a
substitute for H. Existing-R resume preserves the original R and source identities.

The [release qualification profile decision](./2026-10-17-release-check-qualification-profile.md)
documents the current dispatch profile and strict parallel Rust aggregate. Only
explicit release qualification omits instrumented source coverage; ordinary
manual and scheduled Full Check retain it. Qualification still waits for the
whole Check workflow to succeed. Fuzz, Miri, Real Project Matrix and Docs build
keep their separate schedules and are not required release workflows. Protected
version integration and authenticated tag promotion remain required.

Docs build no longer walks the reviewed browser routes or Open Graph images.
The site build and its component-preview evidence remain. The route scripts
stay in the repository for local use. The wall-time benchmark can still time
navigation capture; that benchmark is not a release gate.

The release preflight command is
`node tools/support/compat/github/release-preflight.ts --verify-only`. A pinned candidate
passes `RELEASE_PR_NUMBER` and `RELEASE_TAG_NAME` on the release branch. The
release commit itself is not on main yet. Preflight checks that its parent is
still on main's first-parent history, that main still has that parent's
version, and that the tag does not exist yet. This publishing preflight rechecks
live qualified H evidence; it does not dispatch or wait for a cold Check. The
operator's source qualifier dispatches Check on `release/<tag>` before R exists,
because the tag ref does not exist until publication. The publishing preflight
joins that existing Q before its unchanged verify-only command; it cannot
bootstrap or rerun qualification.
The package build sequencer is `node tools/support/github/run-many.ts`.
Node's stable type stripping runs that TypeScript file. Package builds still
install `rust-script` because tasks such as `generate:rule-types` call the
remaining `tools/commands/**/*.rs` entrypoints.

The earlier five-workflow and browser-capture release contract remains historical;
the current required workflow is exact-H Check. The source handoff is implemented
in `tools/support/release/pr_pin_runs.rs`, and the verify-only publishing preflight
is wired by `.github/workflows/release-preflight.yml`.
