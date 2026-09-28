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
started. At those timings, even zero tests would exceed two minutes. The
diagnostic below measures the actual distribution and cross-job effects.
Ordinary PR latency remains a separate #6830 input/setup problem. This local
change is unmerged; the diagnostic result is not a protected queue run.

The [exact diagnostic branch run 36444784514](https://github.com/ubugeeei-prod/vize/actions/runs/36444784514)
finished successfully. Its four isolated shards ran 163/163/163/162 files,
with 5,322 passes, zero failures and 12 skips combined. The four Node times
were 299.52, 228.28, 146.68 and 177.60 seconds, totaling 852.07 seconds;
the serial #7113 Node time was 847.70 seconds. The tooling steps took 348,
274, 192 and 223 seconds, and the slowest job took 7m47s from runner start.
The diagnostic aggregate succeeded after all four jobs, and shard 1 alone
uploaded the unchanged formatter API evidence artifact. The run took 8m37s
from creation to final status, still far above two minutes. An earlier
diagnostic run failed two workflow contract tests tied to the former command
name; those contracts now assert the per-shard CLI receipt and matrix wiring.
The actual required `test-report` context and protected merge queue still
need an exact-head PR run before this change can merge.

The [second successful diagnostic run 36445933394](https://github.com/ubugeeei-prod/vize/actions/runs/36445933394)
measured all 651 unique file durations while retaining the same complete
suite. The four equal-count shards consumed 296.05, 217.00, 135.25 and
165.69 seconds of measured file time, so round-robin file counts hid a
large cost skew. Pin rounded durations for the 50 files that took at least
four seconds in this run. Assign the formatter evidence test to shard 1,
then place descending measured-cost files on the lightest estimated shard;
new and faster files use a one-second estimate. This stable planning hint
does not filter any file. Replaying those 651 measured durations through the
new partition projects 209.45, 190.82, 209.18 and 204.52 seconds per shard,
an 86.60-second reduction in the slowest file-time shard. The projection
comes from one run and must be validated by another Actions diagnostic.

The [weighted exact-head diagnostic run 36447504215](https://github.com/ubugeeei-prod/vize/actions/runs/36447504215)
then succeeded with all four jobs and their aggregate. Its fresh timing
artifacts contain 651 distinct files, split 163/162/163/163, with file-time
sums of 206.73, 192.15, 202.33 and 201.27 seconds. Node reported 218.62,
202.09, 211.17 and 208.99 seconds, totaling 840.86 seconds; 5,322 tests
passed, none failed and 12 were skipped. The slowest Node shard was 80.90
seconds faster than the 299.52-second unweighted shard; the test step fell
from 348 to 263 seconds, and the run from 8m37s to 6m25s. The formatter
evidence artifact remained on shard 1. The diagnostic is still not a
protected queue or required `test-report` run. The measured setup and native
build keep the queue far above two minutes, so further setup reduction is
needed separately.

## Next measured choice

The [ordinary #7136 PR run 36436934899](https://github.com/ubugeeei-prod/vize/actions/runs/36436934899)
selected 504 of 651 tooling files, with 107 explicit T1 deferrals. Its
tooling job took 9m43s. The test step started after 117 seconds of setup,
including 27 seconds to install `rust-script`, 10 seconds to hydrate fixtures
and dependencies, and 56 seconds to build and receipt the source CLI. The
462-second test step included a 47.56-second native build, a subsecond tool
layout check and 412.16 seconds of Node tests. Even ideal eight- or
sixteen-way Node shares would be about 52 or 26 seconds; the current 117
seconds of setup plus 48 seconds of native preparation already exceed two
minutes before any test runs.

The smallest setup experiment is to let the tooling-only native wrapper use
Cargo `--profile ci`, as the source CLI already does, while preserving its
no-JavaScript package loading behavior. The current native `build:debug`
uses the separate `dev` profile. Compare exact-head Actions times and tests
before claiming any artifact reuse or speedup; leave manual and JS-package
builds on their current task. This profile experiment alone cannot establish
a two-minute gate.

The [same-profile exact-head diagnostic 36449697411](https://github.com/ubugeeei-prod/vize/actions/runs/36449697411)
passed all 651 files and its aggregate, with 5,322 passes, zero failures
and 12 skips. Its native `ci` builds took 39.16, 42.40, 39.58 and 40.95
seconds, versus 41.36, 45.75, 44.30 and 44.14 seconds for the weighted
`dev` run. The two-profile savings were only about two to five seconds per
shard; the complete run took 7m14s versus the previous 6m25s amid other
runner variation. The separate profile code remains diagnostic-only and is
not part of this proposed merge change. Prioritize prerequisite cohorts
over this profile variation.

Next broader experiment: audit selected tests' actual preparation needs and
declare capability cohorts for pure scripts, native binding, source-built CLI
and their overlap. Unknown or dynamic requirements must retain the complete
setup. Keep exact-once selected input coverage, move the tool layout check
to a separate required parallel job only if its receipt remains enforced,
and run each cohort in an isolated job with just its proven prerequisites.
The CLI cohort keeps the source-built receipt, the native cohort keeps its
binding build, and the required aggregate waits for every cohort and the
layout check. Merge groups still cover the complete tooling inventory.
Skipping the repeated `rust-script` install on tooling shards would save
about 25–27 seconds but cannot meet two minutes by itself. Compare eight
and sixteen shards only after the setup split is measured in Actions; a
prepared runner or cache improvement may still be required. This is a
proposal for a separate unpublished slice, not a demonstrated two-minute
gate.
