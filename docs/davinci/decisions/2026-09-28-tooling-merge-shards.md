# Complete tooling suite in isolated merge shards

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

Keep the ordinary PR tooling selector and the scheduled/manual `test:scripts`
task as they are. In `merge_group`, enumerate the complete tooling inventory
and run it in four Actions matrix jobs with separate checkouts. The planner
produces the matrix IDs from the same shard count as the runner. Each runner
re-enumerates the current test files, rejects duplicate or missing formatter
evidence inputs, verifies an exact-once partition, then runs its shard with
`--test-concurrency=1`. This preserves serial access to shared fixture and
report paths within each checkout. The separate runner workspaces prevent the
four processes from mutating the same paths.

All four jobs retain the pinned tools, hydrated fixture submodules, native
test build, layout gate, source-built CLI and receipt, plugin isolation and
tsgo requirement. Rust cache target suffixes include the shard number on
merge groups, so concurrent jobs do not write the same cache identity. The
formatter API evidence test is assigned to shard 1; only that shard uploads
the existing artifact name. The `test-report` required context still waits
for the called `pr-source-checks` workflow, whose matrix fails if any shard
fails. The active ruleset requires `test-report`, `check-js`,
`check-vize-apps` and `fmt-rust`; none is removed or made optional.

## Evidence and limits

The protected #7118 Check ran all 651 tooling files: the tooling job took
940 seconds, its complete test step 849 seconds and Node reported 803.17
seconds. The #7113 queue job took 16m58s: 111 seconds of preparation and
15m02s of the complete test step, with Node reporting 847.70 seconds,
5,320 passes, zero failures and 12 skips. #7142 took 16m36s: 127 seconds of
preparation and 14m24s of complete tests, with Node reporting 810.74 seconds
and the same passing inventory. Its separate Rust differential corpus failed.
All other substantive #7113 jobs completed before tooling. The single
serial tooling step is the observed merge long pole. Four shards are a
bounded first split, not proof of a two-minute merge queue: even a perfectly
balanced quarter of the #7113 Node time is about 212 seconds before each
shard's setup. The #7113 job also spent 111 seconds before its test step and
the task rebuilt the native binding for about 55 seconds before Node tests
started. At those timings, even zero tests would exceed two minutes. Test
distribution and cross-job effects require an actual Actions run. Ordinary
PR latency remains a separate #6830 input/setup problem. This local change
is unmerged until Actions confirms complete coverage, evidence upload and
the required aggregate on an exact head.

## Next measured choice

If four shards leave an excessive queue tail, compare eight and sixteen
isolated shards on the same exact head. Their ideal shares of the observed
#7113 Node time are about 106 and 53 seconds, respectively, before setup,
native build, scheduling and imbalance; neither proves a two-minute total.
An independent setup slice must audit which selected tests need the native
binding, source-built CLI, rust-script, MoonBit and fixture hydration. Keep
each necessary receipt and fail-closed selection, but prepare a shared
artifact or separate CLI-independent partition only after measuring the
transfer and critical path in Actions. The complete merge inventory and
required aggregate remain conditions for either option.
