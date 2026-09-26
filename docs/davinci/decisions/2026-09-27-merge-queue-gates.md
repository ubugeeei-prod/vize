# #6865 — full source suites in the merge queue

The native merge queue is already enabled. T1 planning selects all four source
lanes on every `merge_group`, including docs-only, package-only and empty source
diffs. Pull-request selection remains unchanged; reducing T0 waits for successful
T1 evidence.

The required source report waits for Rust, JS package tests, the full tooling
suite and playground browser/VRT results. The required `test-report` waits for
that source report and every other existing required job. A failed, cancelled, skipped or
missing required result is unsuccessful.

The shared Rust test action preserves the full/manual Check recipe: workspace
first, then every explicit feature-enabled differential test. The merge queue
runs its existing workspace step (including the #6861 build/run measurement
instrumentation when stacked) before calling the same action for the feature
tail. Workspace tests run once per source Rust job. Pkl setup and the mandatory
TSGO environment are retained. Queue Rust gets the existing full Check limit of
45 minutes; PR Rust keeps its 35-minute limit. Feature tests must actually run.

The full tooling job retains its source-built CLI, fixture setup and actual CLI
receipt when the shared #6891 harness is stacked. Shared formatter tests stay in
that tooling job; their current unsupported native lane is not turned into
native acceptance by this orchestration change.

Manual/scheduled `test-scripts` also generates its own receipt immediately after
building the CLI. A receipt from the separate app-check job cannot serve this
job's source-built formatter tests; both tooling callers enforce that prerequisite.

Nightly/manual portability, allocation benchmarks, coverage, editor conformance
and real-project resource work keep their current scheduling. The #6868
instruction gate is a separate unfinished task. This change does not claim the
remaining Davinci native acceptance work is complete.

Local planning/orchestration tests are verification of dispatch and failure
propagation. #6865 remains open until a fresh synthetic merge group actually
executes the full planned suites and all required statuses succeed. Actions,
publication and T1 runtime evidence remain pending until the release hold is
lifted. The #6831 dependency guard remains a separate foundation change.
