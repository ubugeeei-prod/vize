# Release tooling contract inputs

Tracks [#6863](https://github.com/ubugeeei-prod/vize/issues/6863).

## Decision

The 26 explicitly listed `tests/tooling/release/` contracts use
`toolingReleaseContracts` in the shared Vite+ input catalog. These tests
exercise standalone release and CI Rust scripts, JavaScript helpers, synthetic
package trees, checked-in release/workflow contracts, and temporary Git/API
fixtures. Their source inputs include `.github/**`, `.cargo/**`, Cargo
workspace metadata, `tools/**`, `tests/**`, `npm/**`, `editors/**`, `docs/**`,
`examples/**`, and `README.md`. The fresh-install runner can copy
`crates/vize/tests/fixtures/content_mapper_project/**`, so that specific
fixture tree remains an input. Each test's transitive literal local imports
are added by the existing selector.

Unrelated Rust implementation files do not affect these standalone contract
tests. A direct test change, global task/dependency input, unknown path, or
unresolved import restores broad selection. Two existing tests,
`release-smoke-init-fresh.test.ts` and
`release-smoke-init-typecheck.test.ts`, retain broad selection because their
import closures are unresolved. New release tests also remain broad until
their dependencies are audited and they are explicitly listed. The T1 merge
suite still enumerates and runs all 28 release tests.

## Evidence and limits

On the current 640-file inventory, a `crates/vize_l1/src/parser.rs` edit
selects 458 PR tooling files, including the two unscoped release tests;
without this scope it selects 484. Focused selector tests check all 26 import
closures and assert that workflow, release script, manifest, documentation,
copied fixture, global and unknown inputs restore the complete release group.
The existing full-suite and unresolved-import selector tests remain in place.

The historical [PR tooling job](https://github.com/ubugeeei-prod/vize/actions/runs/36314259215/job/108605910906)
lasted 455 seconds with 520 selected files and 315 seconds of Node test
execution. That PR changed shared inputs and therefore cannot establish a
duration saving for this narrower scope. TODO: measure exact-head PR and
merge-group durations after this change, then assess p50/p90 over subsequent
merged PRs. The T0 3-minute/6-minute targets remain unproven.
