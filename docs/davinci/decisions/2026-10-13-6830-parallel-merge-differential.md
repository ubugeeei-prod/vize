# Merge-queue Rust differential tail starts beside the producer

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

In the merge queue, the `Required Rust differential corpora` job no longer
waits for the `PR Rust Clippy, tests, and fixtures` producer, and it no longer
downloads, verifies and extracts the producer's nextest archive. It starts at
the same time as the producer and runs the unchanged recipe in
[`test-rust-workspace-differential`](../../../.github/actions/test-rust-workspace-differential/action.yml),
followed by the unchanged fixture coverage check.

Every check still runs, and the `Rust source report` still requires both the
producer and the differential tail to succeed.

## Reason

Measured from the Actions job and step timings of the merge-queue candidates
of 2026-10-13:

- A candidate took 19.3 minutes at the median and 21.4 at the 90th percentile.
- The critical path was the producer (4.7 minutes), then the differential tail
  (11 minutes), then three report jobs (1.5 minutes).
- Inside the tail, 48 seconds went to downloading the 5.1 GB archive and 32
  seconds to extracting it.

The tail never executes that archive. Every command in the recipe is a
`cargo test`, `cargo check` or `cargo clippy` with its own feature set in the
dev or test profile, so Cargo builds separate binaries; the archive holds
`ci`-profile binaries. The coverage check builds `vize_test_runner` itself.
The dependency on the producer therefore only delayed the queue.

## Tradeoff

A candidate whose producer fails now also spends the runner minutes of the
tail. Queue candidates have already passed their PR checks, so that is the
uncommon case, and wall time, not runner capacity, limits merge throughput:
jobs waited a median of 6 seconds for a runner.

## Scope

This changes only the ordering and the archive transfer of the merge-queue
differential job. The archive, its receipt, and the four workspace shards that
do execute it are unchanged.
