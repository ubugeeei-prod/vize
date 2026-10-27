# Relative sibling SFC storage (#10934)

This minimized project preserves the graph reported by [dmnlk](https://github.com/dmnlk)
in [Issue #10934](https://github.com/ubugeeei-prod/vize/issues/10934): an app imports
a sibling SFC by a relative climb while a shared tsconfig explicitly includes a
hoisted Vue JSX declaration. The SFC imports the app's `LocalId` type in return.

The starting CLI regression is pinned at
`cb9c06d2b879da6b47a0a2f783d9a5f324776585`. This pack moves only the isolated JSX
stub into a pnpm-shaped installed path, so the corpus runner can retain its real
Vue provider without replacing that provider through its `node_modules/vue` link.
The `vue@3.4.23` directory is fixture spelling, not a runtime version claim.

`relative_sibling_imports_preserve_complete_package_diagnostics` executes both
cases through the production `BatchTypeChecker` rooted at `apps/app`, using that
package's tsconfig. The capture root remains the whole temporary workspace so
all input bytes and the complete authored diagnostic tuple retain safe paths.
The invalid case must return exactly TS2322 at `apps/app/src/entry.ts:2:31`; the
repaired case must return no diagnostics. Native adapter parity is unsupported.

The source CLI test
`check_package_external_import_cli::relative_sibling_sfc_keeps_package_storage_and_exact_repeated_diagnostics`
consumes these same input files, runs the invalid case twice and the repaired
case once, and additionally checks persistent package storage, stable build-info
paths, unchanged SFC/model mtimes and untouched installed declaration bytes.
Those repeated-run assertions are independent of any cold-check performance goal.
