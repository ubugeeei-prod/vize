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

## Audited Rust-source PR smoke refinement

The earlier pure/full partition remains the fallback for every unsupported
input. For a PR whose changed paths are limited to `Cargo.lock`, `crates/**`,
`docs/davinci/**`, and the two audited structural contracts
`davinci-ssr-l4-bridge.test.ts` and `davinci-stage-dependencies.test.ts`, run
an independent early source-smoke job instead of the broad selected tooling
job. The job builds `target/ci/vize` from the candidate source, writes its
build receipt, invokes that exact binary, and runs either directly changed
structural contract with the source-binding requirement set. Rust, JS and
browser PR jobs retain their existing coverage for the changed source. An
empty comparison, unknown path, deleted audited contract, tooling helper edit,
new or other test file,
action edit, npm edit, or dependency input outside `Cargo.lock` restores the
prior broad PR tooling preparation. The required source report waits for both
the early smoke and the broad-or-no-op job. The protected merge queue retains
all tooling files in four full shards with the same receipt and report.

This is a deliberate T0-to-T1 movement of the unrelated tooling contracts
for the audited input class, not a cache-input assumption. The [#7132 exact
head run](https://github.com/ubugeeei-prod/vize/actions/runs/36457895487)
selected 503/651 tooling files and took 8m46s in the tooling job; the
required `test-report` took 9m55s from run creation. Its source CLI build
took about 59 seconds, NAPI build 45 seconds and serial Node suite 362
seconds. A [push-only exact-source diagnostic](https://github.com/ubugeeei-prod/vize/actions/runs/36459689901)
with the CLI receipt, binary smoke and both changed structural tests passed
in 103 seconds from run creation. An [integrated push-only
diagnostic](https://github.com/ubugeeei-prod/vize/actions/runs/36460619184)
fetched that exact base and head, selected the same two structural tests with
the new planner, built the source CLI and receipt, and ran them with the new
runner. It passed in 87 seconds from run creation. Neither diagnostic is an
ordinary PR `test-report`; the complete protected queue still runs. In the
same #7132 Actions run, the Rust source report completed 7m18s after run
creation, playground browser tests at 4m49s, and JS package tests at 3m39s.
Even with an 87-second tooling lane, the required aggregate remains above
two minutes. The next critical path is Rust source build/test/report, followed
by browser and JS work; that is separate #6830 work requiring its own measured
diagnostic and preserved T1 coverage.
