# Bounded concurrency for CI walltime (#6830)

The maintainer requested one PR for four optimizations and an actual before/after
measurement. These changes preserve the existing input sets, assertions and
publication barriers. They do not claim the roadmap's T0 latency target is met.

1. Qualify both pinned Oxlint hosts concurrently, at most two processes, after
   preparing the full n8n baseline once. Keep each host's complete native-call
   ledger isolated and aggregate only after both finish. Exact source identity,
   binary hashes, corpus inventories and all diagnostic projections remain
   mandatory. A failed host fails the qualification and retains both outcomes.
2. Render Docs with two workers, independent browser contexts and stable unique
   evidence names. Preserve every route/device job, full-page capture, layout
   assertion and capture control. The current manifest is 45 routes and 90 jobs;
   count from the manifest rather than preserving an outdated fixed count.
3. Run the full ordinary Rust workspace with the existing pinned nextest 0.9.146
   `full` profile, then explicitly run all workspace doctests. Keep its 16-test
   concurrency, serialized contract guest builders and unfiltered selection.
   Feature-enabled differential recipes and required TSGO remain unchanged.
4. Publish native platform packages with at most four asynchronous child
   publishers. Each keeps the existing provenance, retry, visibility and
   idempotency checks. On the first observed failure, stop queued launches and
   await every active child. Retain sorted results, skipped package identities
   and complete decoded UTF8 child diagnostics; reject any failed aggregate.
   Publish the parent package only after the platform step succeeds.

## Measurement

`CI walltime comparison` runs paired serial/concurrent modes on the same
checked-out SHA, the same runner class and identical input obligations. For a
PR, opening a `ci/optimize-*` branch or applying the `ci-walltime-benchmark` label
starts the measurement; ordinary subsequent pushes do not rerun it. After
registration, its manual input also allows retrying one component. No benchmark
has publication credentials or registry write authority.

- JS reuses one source-built frozen native binary for both complete original
  project qualifications; retain baseline and host phase timings and ledgers.
- Docs builds the site once, then verifies every manifest job with one and two
  workers. Retain both complete receipts, screenshots and per-phase timings.
- Rust compiles all ordinary binaries once, records the unfiltered nextest
  inventory, then compares `cargo test --locked --workspace` with full nextest
  plus doctests. Compilation preparation is excluded from both runner timings.
- Native publishing runs the actual MoonBit native parent with eight controlled
  publisher subprocesses. Three samples compare one and four workers. These
  measurements establish scheduling overlap; real registry visibility delays
  and total release improvement must be measured on the next authorized release.

Serial modes exist for reproducible comparisons. Docs and Rust use one pair per
Actions run; this is a measured sample, not a stable percentile estimate. Failed
comparisons remain failed evidence and cannot establish a speedup.

## Acceptance and follow-up

- Require focused failure/isolation tests and exact-head PR checks before queue
  entry. Keep complete differential, rendering and publication obligations.
- Record actual Actions links, phase timings and limits in this PR and #6830.
- Inspect the new critical path after measurement; the canonical corpus and
  full Rust source branch may limit total PR/merge walltime after JS improves.
- Confirm platform publication timings from the retained receipt on the next
  release before reporting a production publishing speedup.

Results are pending the first exact-source Actions comparison.

## First source correction

The original `bdc08716` source Actions failed on six lint warnings, the staging
paragraph location/output guard, two files crossing the existing 350-line
limit, and the Check workflow growing beyond its unchanged 690-line allowance. Keep those failures as evidence. Explicit promise handling and numeric
sorting fix lint; extraction preserves every page operation and seven unchanged
publisher laws. The staging law still checks the actual source paragraph,
exclusive native ownership, whole staged bytes, and exact additive output paths.

The original scheduler also launched remaining packages after a failed serial
publisher. Restore serial stop-first behavior and stop queued concurrent launches
before releasing the failed task's semaphore slot; active publishers still finish.
Independent barrier laws require one or four launched children, complete failure
results, all active process exits, seven or four skipped identities, and exact
CRLF/boundary-whitespace diagnostics. A receipt records complete decoded UTF8
texts, rather than claiming binary-byte custody. Wrapper publication still needs
all platform publishers and their registry visibility checks to succeed.

Public release acceptance recognizes exactly the old native invocation or the new
four-worker/provenance/quoted receipt invocation, retaining the original release
plan vectors and refusing missing, duplicate or hostile publication commands.
The full-workspace artifact upload uses a dedicated upload-only composite at its
original top-level Check position. Both the outer call and its sole upload step
keep `always()`, so preceding setup failures and runner/feature-tail failures all
retain the original upload attempt without running tests that previously skipped.
The SHA/run-attempt name, path, missing-file warning and retention remain exact.
The shared Rust recipe remains byte-identical. Only typed-equivalent flow
formatting of the global and Clippy environment maps recovers three lines; Check
retains its original 690-line allowance. The rejected inner-only extraction is
retained as failure-boundary evidence.

The two new benchmark Rust setup calls use their proven reachable provider parent
with explicit stable; the 86 inherited generated-provider pin findings are handled
by a separate security change. Preserve existing explicit versions, channels,
components and targets when that actual merged main is incorporated.

The first paired Docs run failed the unchanged Musea link checker before its
serial measurement. Its concurrent step completed all 90 original jobs, which is
standalone historical evidence only. The two inherited Musea links are repaired
separately in #8509 and must be incorporated after that actual merge.

Focused source laws are local evidence only. Fresh exact-head Actions, complete
paired measurements, protected merge and actual publication remain required.
