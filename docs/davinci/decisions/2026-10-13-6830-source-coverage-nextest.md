# Source coverage runs its tests under nextest

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

The full Check's `source-coverage` job runs
`cargo llvm-cov nextest --workspace` instead of `cargo llvm-cov --workspace`,
and its limit rises from 40 to 50 minutes. The measured tests, the coverage
report and the minimums are unchanged.

## Reason

The job ran every instrumented workspace test binary one after another under
libtest. Its report step took 26 to 32 minutes on 2026-10-11 and 2026-10-12,
then 36 to 39 minutes on 2026-10-13 as tests were added. Two runs reached the
40-minute limit and were cancelled, including run 38433057234, which retired
the v0.449.0 release candidate.

nextest runs the binaries in parallel, with the `full` profile's test groups and eight threads. At the default 32 threads, instrumented real-TSGO LSP tests timed out and process spawns failed with `EAGAIN`. The PR and merge-queue Rust shards
already run the whole workspace under nextest, so every test is known to pass
there. The higher limit keeps headroom for the next growth in tests.
