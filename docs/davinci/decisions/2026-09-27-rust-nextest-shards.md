# Rust PR archives and test shards

Tracked in [#6862](https://github.com/ubugeeei-prod/vize/issues/6862), under
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

PR Rust checks compile affected packages once into a nextest archive and run
four deterministic hash partitions on separate Linux runners. The affected
package plan is also used by Clippy and doctests. The full workspace gate stays
on merge groups; its required tsgo runtime and fixture coverage remain there.
The default workspace suite, doctests, and the feature-specific differential
commands remain required in the full validation recipe.

Nextest 0.9.146 is pinned on the builder and runners. Each runner schedules at
most 16 test processes. The `vize_extension_host` test group schedules one at a
time because its guest builds share `target/contract-guests`; its Rust OnceLocks
do not synchronize separate nextest processes. Tests using process-local
profiling counters and environment guards gain isolation. The audited CLI Unix
socket tests use unique temporary directories.

The archive uses the existing `ci` Cargo profile to avoid transferring debug
symbols for hundreds of test executables. This profile inherits `dev`, including
its debug assertions and overflow checks. Doctests run separately because
[nextest does not run them](https://nexte.st/docs/running/). No feature flags or
test bodies change.

T0 sets `VIZE_TEST_DISABLE_TSGO=1` and omits `VIZE_TEST_REQUIRE_TSGO`. Removing only
the requirement flag would still discover the runtime installed in
`node_modules`. The existing guards in `vize_canon`'s test runtime resolver,
`auto_import_project`, Maestro's Corsa tests, and the CLI's `corsa_requirement`
honor the explicit disable flag. These tests may return without checking their
runtime oracle in T0; this is an intentional tier exclusion, not complete tsgo
conformance evidence. T1 keeps required runtime execution.

## Archive identity and required checks

Many tests use compile-time `env!("CARGO_MANIFEST_DIR")` and
`env!("CARGO_BIN_EXE_...")`. Those paths cannot be remapped by nextest at runtime.
The builder records the source SHA and tree, workspace path, runner platform and
architecture, nextest version, and archive SHA-256. Every runner verifies this
receipt and extracts into the original workspace path before running. This
preserves the baked executable and fixture paths. Supporting arbitrary workspace
relocation requires converting those tests to runtime paths in a separate task;
the current workflow fails when paths differ.

Every partition has a separate JUnit artifact. Empty partitions are allowed
because an affected package can have fewer than four tests. The aggregate still
requires the archive/doctest job and the entire four-job matrix to succeed.
Failed, cancelled, absent, and skipped required jobs fail the Rust source gate.
Documentation-only PRs retain an explicit skip explanation.

## Verification and follow-up

Unit tests reject wrong archive content, source identity, baked paths, runner,
and nextest version, and reject incomplete required job results. A small local
archive smoke test removes the original build directory, restores the archive,
and checks both baked fixture and executable paths; it also checks that empty
partitions write JUnit results.

Actual workspace compilation, process concurrency, artifact transfer cost, and
the p50 three-minute / p90 six-minute targets still require Actions evidence.
The earlier measured workspace build was 170 seconds and execution 1,117 seconds
on one partially reused Linux cache; it does not establish cold-build latency or
the new archive runner's speed. Adjust shard count or resource groups only from
the collected job and JUnit evidence. Do not claim the targets are met before
measuring them.
