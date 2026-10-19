# Timed tooling shards

This [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) decision
packs the planned tooling test files onto up to 16 runners by measured
duration, instead of dealing them round-robin onto 4. The planner still selects
exactly the same files, every selected file still runs exactly once, and each
runner still runs its files serially with `--test-concurrency=1`. Only the
placement of files and the number of runners change.

## Why

The `PR tooling scripts (N/4)` jobs were the long pole of a PR `Check`. In
[PR run 38355807070](https://github.com/ubugeeei-prod/vize/actions/runs/38355807070)
the four shards spent 338-421 s in their test step for 1,104 selected files,
about 1,480 s of serial work in total. In
[merge run 38371147974](https://github.com/ubugeeei-prod/vize/actions/runs/38371147974)
the full suite of 1,227 files took 442-637 s per shard, and shard 1 also spent
about 246 s on the native captures that only matrix entry 1 runs.

No single file dominates: the slowest takes about 43 s and the median about
0.1 s. A fixed count of 4 cannot get below about 370 s however the files are
placed, so the count has to rise. Round-robin placement by alphabetical index
also ignores how long a file takes.

## Decision

`tooling-test-shards.ts` estimates each planned file from the committed table
`tests/tooling/fixtures/tooling-test-timings.json`. It packs them with
longest-processing-time-first: the heaviest file goes to the least-loaded
runner, ties go to plan order and then to the lowest runner index. Every
runner regenerates the same plan and table, so all of them derive the same
partition. Inside a runner, files keep plan order.

The matrix uses the fewest runners, up to `MAX_TOOLING_SHARDS = 16`, whose
estimated test time stays within `TOOLING_SHARD_TARGET_MS` (180 s for the PR tier,
which starts after the source CLI beside the Rust cohorts; 100 s for Full). The
target never drops below the single heaviest file, which no partition can split.
Small selections therefore still get one runner.

A file missing from its tier's table uses the other tier's value. A file
missing from both, normally a new file, gets `defaultMs` (10 s). That is above
the 95th percentile on purpose: an overestimate costs one more runner, while an
underestimate lengthens the critical path.

### Runner 1's captures

Matrix entry 1 alone also runs the native SSR, scoped CSS, JS setup, Vapor and
attribute-value captures, the selected-template modules and the historical
Docs join. The written plan now carries `leadSteps`, the IDs of the steps that
will run. A PR includes a capture only when the planner output that gates its
workflow step is true; the merge queue includes all of them; the historical
join is always included. Packing starts runner 1 at the measured time of those
steps, so runner 1 receives files only once the other runners catch up. A
contract test checks that the workflow's `matrix.index == 1` steps, their gates
and `TOOLING_LEAD_STEPS` agree. These steps still run exactly once, on runner 1.

### Equivalence

- `toolingShards` checks that the shards together hold every planned file
  exactly once, with nothing added, before it returns a partition.
- `selectToolingShard` accepts only `index/total` coordinates whose `total`
  equals the recomputed matrix size, as before.
- `toolingTestCommand` still rejects duplicate or unknown files, and a merge
  plan that does not hold every discovered file.
- The merge queue uses the same code with `test:scripts:planned`. Its plan
  already holds every file; only the runner count and placement change.

The tests in `tooling-test-shards.test.mjs` check the partition for synthetic
and real plans in both tiers, with and without lead steps. They also check the
minimal runner count, the bound for longest-first packing (the longest runner
is at most the average plus the heaviest file), and the per-runner Chromium
selection.

## Simulated result

With the committed table at the time of writing:

| Plan                              | Runners | Estimated test time per runner       |
| --------------------------------- | ------- | ------------------------------------ |
| Broad PR, 1,114 files, no capture | 16      | 90-101 s                             |
| Broad PR, every capture           | 16      | runner 1 captures only; others 107 s |
| Merge queue, 1,227 files          | 16      | runner 1 captures only; others 144 s |

The broad PR estimate includes 21 unmeasured files at the 10 s default. With
measured values only it is about 93 s.

## Timing table

The first table was built from the runs above. Their spec logs do not name
files, so each file's time was attributed by matching top-level test titles in
the log to the titles in that file's source. From now on, `run-tooling-tests.mjs`
adds a second reporter, `tooling-test-timing-reporter.ts`. The spec output on
stdout is unchanged. After the tests, the runner logs one line,
`vize-tooling-test-timings {"tier","shard","files"}`, with each file's exact
process wall time, imports included. That line never changes the exit status.

To refresh the table, run:

```sh
node tools/support/compat/github/update-tooling-test-timings.ts <run-id>...
```

The script reads that line from every successful tooling job of each run, and
reads the lead-step durations from runner 1's step timings. It overwrites only
what those runs measured and drops files that no longer exist.

The table names every tooling test, including the release-input roots whose
mentions under `tools/` must belong to the reviewed cohort in
`typechecker-native-release-inputs.json`. The table only orders those tests and
never runs one itself, so it lives beside them in `tests/tooling/fixtures/`, and
`tooling-test-shards.ts` reads it at run time instead of importing it. A refresh
therefore needs no new review. The changed shard and planner code is repinned
as the `reviewed-timed-tooling-shards` cohort.

## The source CLI dependency

`pr-tooling-scripts` keeps `needs: [pr-source-plan, pr-source-cli]`. The
consumer validates the restored CLI against the producer's job outputs
(`sha256`, `receipt-sha256`, `producer-attempt`), passed through
`needs.pr-source-cli.outputs` and `VIZE_SOURCE_CLI_RECEIPT_SHA256`. GitHub
exposes job outputs only through `needs`; the REST API does not return them.
Starting the job early would mean reading the digest from the artifact it is
meant to check, or from another channel. Either would change the receipt
custody that `restore-source-cli` and its contract tests enforce. Polling inside
the job would also hold every runner idle for the producer's 90 s. The
dependency therefore stays, and the producer's own build time is the place to
shorten that wait.

## Tradeoffs and unfinished work

A broad PR now starts 16 runners instead of 4. Each of the 12 extra runners
pays the fixed setup (about 37 s of checkout, toolchain, install and fixtures,
plus the native build), in exchange for about 5 minutes less wall time. The shard that owns the Pkl and Chromium
preparation still pays about 25 s more than the others, because that cost is
not yet in the table. The 16-runner cap and the 100 s target are constants;
raising either only moves this tradeoff.

The table should be refreshed from the first PR and merge runs after this
merges, so that its values come from exact per-file logs.
