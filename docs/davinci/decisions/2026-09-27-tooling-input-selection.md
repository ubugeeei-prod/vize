# Tooling test input selection

Issues: [#6863](https://github.com/ubugeeei-prod/vize/issues/6863) and
[#6864](https://github.com/ubugeeei-prod/vize/issues/6864).

## Decision

T0 uses `plan-tooling-tests.mjs` with `--tier pr`. The input groups live in
the Vite+ `cacheInputs` catalog, and `tooling-test-scopes.ts` associates
audited test groups with those inputs. Each test's local import closure is
included, so changing a shared helper or a transitive implementation selects
its callers. Package locks and task configuration select the broad PR suite.
Deleted and moved paths are included by the existing NUL-delimited git diff.
Unknown paths, unavailable dependency closure and an empty change list restore
broad selection. Unreviewed tests retain broad inputs; names alone do not prove
that a source or fixture is unrelated.

Plan status contracts read `docs/**`, including record files and local-link
targets. Storage summary contracts also include their imported inventory and
render helpers. The Phase 2 ledger additionally reads compatibility fixture
inputs. Synthetic typecheck baseline unit tests include their fixture inputs,
package sources and transitive resolver helpers. Their scope is separate from
real checker scenarios; a `typecheck-` prefix does not exclude a test.

The explicit T1 inventory contains 105 current scenarios using the real tsgo
requirement helper or the `LspSession` subprocess launcher. The pure dependency
gate tests remain in T0, including checks that missing required tools fail.
The runner preserves `VIZE_TEST_REQUIRE_TSGO=1`: a newly introduced requirement
cannot turn into an unnoticed optional skip. The inventory is reviewed when new
runtime scenarios are added.

T1 runs `--tier merge`, which enumerates every current test file. Existing
`test:scripts` keeps the full suite and its native build, typecheck requirement
and layout gate. T0 uses `test:scripts:pr` with the same native build and layout
gate followed by the selected plan. Both remain `--test-concurrency=1`, because
tests use shared fixture and report paths. This change does not assume that an
unreviewed selected test can run without the native binding, CLI, fixture
submodules or plugin isolation runtime.

## Validation and limits

Selector tests cover unrelated source changes, shared and transitive imports,
deleted inputs, dynamic imports, unknown paths, full T1 restoration, preserving
pure helper tests and rejecting duplicate or unrecognized execution plans.
At the initial selector baseline (`2b9947139`), the inventory was 605 files including the selector test; the issue's
577 count predates additional tests. A single L1 source edit selected 453 T0
files, with 105 real runtime scenarios deferred to T1. A plan edit selected 460.
These are selection counts, not measured runtime improvements.

CI must measure the resulting T0 duration before claiming the p50/p90 target.
Further narrowing needs an audited input declaration; broad fallback is
deliberate. Native setup and other integration scenarios remain possible T0
costs and must be measured before changing their capabilities.
