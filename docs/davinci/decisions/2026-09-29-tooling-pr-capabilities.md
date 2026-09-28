# Audited tooling preparation cohorts on pull requests

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
This decision is a local, unpublished child of the complete T1 tooling shard
change.

## Decision

Keep the existing input selector as the authority for which T0 tests must
run. Partition its selected files into a small explicit `pure` allowlist and
a `full` default. The initial audited pure tests inspect generated ledgers,
consumer migration, workflow contracts and simulated Rust cache actions.
They require Node, Git, Bash and installed JS dependencies, but do not use
the NAPI binding, source-built CLI, `rust-script`, MoonBit, fixture submodules
or plugin isolation. Unknown files, unreviewed requirements, dynamic imports
and edits to tests, tools, actions, scripts, npm or JS dependency inputs stay
in `full` until their capabilities are reviewed. Two otherwise audited
planner tests have an incomplete literal-import closure, so the planner
keeps them in `full`.

Run pure tests in an independent PR job with checkout, declared Node and JS
dependencies. Retain the existing full preparation, including source-built
CLI, receipt, native build, fixture hydration, tool layout and plugin setup,
for every selected full test. The full PR runner also requires the receipted
source CLI environment used by the merge runner. The two jobs run in
parallel, always instantiate even when their cohort is empty, and both
feed the required `source-report`. An empty job explains its no-op and
succeeds; skipped jobs would fail the report's existing contract. The
runner validates the plan, pure allowlist and exact-once partition before
executing either cohort. The T1 merge plan and all four full shards are
unchanged.

## Evidence and limit

The ordinary [#7136 Actions run](https://github.com/ubugeeei-prod/vize/actions/runs/36436934899)
selected 504/651 tooling files; its full tooling job lasted 9m43s, with
117 seconds before the test step, about 48 seconds for native preparation
and 412 seconds of Node tests. An independent import and subprocess audit
found nine pure candidates totaling 38.36 seconds of measured file time.
The planner currently admits seven of them; the other two fail its
conservative import-closure check. On the current branch, replaying the
`Cargo.lock` input selects 496 tests: 7 pure and 489 full. Their prior
Actions file times sum to 37.82 and 376.08 seconds. Thus this first cohort
proves a preparation boundary but does not materially shorten the full
job or meet the two-minute T0 target. The
[push-only exact-head diagnostic 36452518335](https://github.com/ubugeeei-prod/vize/actions/runs/36452518335)
validated the replay's 7/489 exact-once partition and ran the seven pure
files without Rust, native, CLI or fixture preparation: 48 tests passed,
none failed or skipped. Pure-job preparation took about 12 seconds, its
Node tests 38.14 seconds, the job 56 seconds and the diagnostic aggregate
80 seconds from run creation. The required PR `test-report` and protected
merge queue have not run on this unpublished change. Wider audited cohorts
and likely further full-cohort partitioning are needed before any overall
latency claim. The complete protected queue and required contexts remain
the release gate.
