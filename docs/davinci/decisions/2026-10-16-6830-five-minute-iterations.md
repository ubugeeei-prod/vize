# Five-minute development checks and warm release builds

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The maintainer requested one CI optimization change targeting at most five
minutes for ordinary PR checks and approximately ten minutes for releases.
These are elapsed-time targets, not permission to truncate tests, soften
assertions, accept stale binaries, or report an unmeasured improvement.

This decision supersedes #6830's original policy of running the full tooling
suite and broad ecosystem qualification on every merge candidate. Product
regression checks remain on affected source and its transitive consumers;
repository documentation, inventories, historical project campaigns, coverage,
Miri, fuzz campaigns and exhaustive tooling qualification run on schedules or
explicit dispatch. The Real Project Matrix already has weekly/manual triggers;
it remains outside the ordinary PR, queue and release dependency chains.

## Measured starting point

The [baseline receipt](./2026-10-16-ci-fast-baseline.json) records GitHub run IDs,
attempts, source identities, job timings and the live required-check rulesets.
At source `b4be3b70977b80a689c8e5ada62d05283e86b868`:

| Surface               | Recent measurements               | Dominant work                                                  |
| --------------------- | --------------------------------- | -------------------------------------------------------------- |
| Passing PR Check      | 12m 03s, 12m 32s, 12m 51s         | Whole original-project Oxlint qualification, then reports      |
| Passing main Check    | 13m 03s, 14m 34s, 14m 49s         | Full JS qualification and historical project transport         |
| Release first attempt | 40m 16s, 40m 44s, 45m 55s; failed | Native Windows builds and dependent publication/registry waits |
| Nuxt artifact         | 934 MB; 73s upload                | Fixture node_modules followed through symlinks                 |

The release preflight had already stopped requiring exhaustive workflow runs.
The current source/identity check took 21s and crate dry run 42s in the sampled
release. The remaining bottleneck is the build and publication graph, rather
than a reason to remove those safety checks.

## Development verification

The existing required contexts stay `check-js`, `check-vize-apps`, `fmt-rust`
and `test-report`. The protected merge queue remains enabled. A lightweight
Node planner selects product-app checks, dependency auditing and package engine
compatibility from exact event/checkout comparisons. Changed selector and task
definitions exercise their consumers. A failed comparison fails the plan and
therefore the aggregate; it never becomes an empty successful selection.

The fast JS task retains lint, formatting and a zero-warning budget. It does
not install a Rust script runtime or typecheck every private documentation and
tooling project. Product package checks follow the workspace reverse dependency
graph, including peer, optional and development dependencies and native addon
consumers. Full repository checks remain scheduled/manual.

Cargo's own offline metadata supplies the crate graph. The planner uses the
runner's existing Cargo to read it without downloading the pinned compilation
toolchain; actual builds still use the pinned toolchain. Changed crates and
transitive consumers run Clippy, tests and doctests. Unknown/shared inputs select
the workspace. Merge candidates retain applicable feature-enabled compiler,
DOM, SSR, Vapor, JSX and editor differential recipes, runtime witnesses and
required typechecker observations.

Independent runtime and differential lanes start in parallel. Broad tooling
test shards no longer sit between source compilation and those product checks.
The whole original-project Oxlint campaign remains in scheduled full JS tests.
Full historical n8n adoption also remains available nightly and manually.
The [native feedback decision](./2026-10-16-6830-native-feedback.md) moves the
16–18 minute historical phase campaign out of PRs while retaining its unique
native law, feature differential and actual source-built LSP product tests.
Nuxt and Musea integration checks select the inputs they actually exercise;
Nuxt's three runtime scenarios run independently. Artifact uploads exclude
node_modules while keeping candidate/public adapter snapshots and observations.

Instruction-count ceilings run in the queue and scheduled/manual qualification.
Their existing advisory policy from [#9183](https://github.com/ubugeeei-prod/vize/pull/9183)
and [#9195](https://github.com/ubugeeei-prod/vize/pull/9195) is preserved; this
performance change does not turn existing level/formatter ceiling failures into
a new merge requirement. Measurements still report failures and existing ceiling
values are unchanged. Required product checks continue to fail the aggregate
when they fail, cancel or unexpectedly skip.

## Build and publication reuse

Hot orchestration entry points use native Node TypeScript type stripping,
including warning/audit wrappers, source selection, cache policy and release
platform planning. Full scheduled tools can retain their separate runtimes
until their own migration; this change does not claim every script was ported.

Cargo target caches remain distinct from released artifacts. Trusted main runs
write warm snapshots; candidate jobs restore compatible snapshots without
committing to trusted storage. Cargo still checks the source, configuration and
toolchain before producing the candidate's outputs. Registry and target cache
names remain separated by platform and build role.

Release build recipes are shared with an asynchronous trusted cache warmer.
The validation entry point exercises those recipes without tagging or publishing.
GNU Linux builders retain the glibc baseline on Ubuntu 22.04. Windows builds use
the existing Blacksmith 32-vCPU runner class; ARM64 cross compilation selects
the appropriate MSVC target environment. Every target, ABI check, native token,
artifact identity and publication prerequisite remains required.

Independent npm platform packages publish concurrently. Their registry visibility
checks remain mandatory; only the wait overlap changes. Real package dependencies
still order native, CLI, Vite and framework publication. Registry propagation,
runner queues and cold caches can exceed the elapsed-time target, and must be
reported separately rather than hidden by timeouts.
Release version bumps also invalidate workspace crate fingerprints, so a warm
repeat at the same SHA does not establish the elapsed time of a real release.

The [distributed-build assessment](./2026-10-16-6830-distributed-builds.md)
compares the concrete Google and Cloudflare practices, Bazel remote cache and
remote execution, sccache limitations and a bounded future adoption experiment.
No external cache service, organization setting or paid infrastructure is
enabled by this change.

## Verification and remaining measurement

Use `node tools/commands/ci/workflow-timings.ts --attempt 1 RUN_ID...` for
comparable elapsed time, executed versus reused jobs, runner minutes and the
critical job tail. The same command without `--attempt` reports the latest
attempt and separates reused jobs from that attempt's duration.

The implementation must pass focused selector/cache/aggregation tests, workflow
validation and exact-head PR Actions. The opt-in `release-builds` component of
CI walltime comparison validates release recipes without publication. Record
observed PR, queue, cold build and warm build durations in the PR before claiming
the targets are achieved. Publication time remains unverified until a real
release runs; no production release is created solely to benchmark this change.
