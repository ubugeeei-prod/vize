# Name CI jobs after their actual work

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The maintainer requested useful job names instead of partition counters such
as `1/4`. The [Rust execution policy](./2026-10-16-6830-required-rust.md)
therefore groups tests by Compiler, Type checker, Language server, Lint and
format, and CLI and integrations. The first Actions run of these groups passed
in [4m19s](https://github.com/ubugeeei-prod/vize/actions/runs/38866126632).
Its 18,128 unique PR test identities exactly match the previous partitioned
run, with no missing, extra or duplicate cases.

Scheduled Real Project Matrix jobs list every project they execute. A small
Node TypeScript planner derives those names from the existing fixture registry
and preserves its modulo partitioning. Internal partition numbers, artifact
names, cache keys and every product command remain unchanged. Evidence readers
derive expected names from the selected run's exact source SHA, including its
workflow template. Old runs accept the old names; new runs require the new
names. Unknown source templates, missing or duplicate jobs, and mismatched
artifact/run/attempt identities still fail verification.

Scheduled tooling workers display their heaviest actual test file and the
number of additional files. Their duration-based packing and exhaustive test
inventory are unchanged. The aggregate retains the exact `test-scripts` name
and requires every planned worker; a worker with a friendly display name
cannot substitute for that aggregate.

Benchmark repetitions are labeled `measurement N`: those numbers identify
repeated measurements of the same workload rather than unnamed test groups.
Application E2E jobs already identify their application and suite.

Focused contracts verify exact partitions, historical/source-specific evidence,
failed/missing/duplicate aggregate evidence, and unchanged runtime commands.
PR #9198 records final Actions and merge-queue validation.
