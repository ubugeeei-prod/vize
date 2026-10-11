# Protected source-build qualification and remaining critical path

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

[PR #8617](https://github.com/ubugeeei-prod/vize/pull/8617) actually merged at
`2026-10-11T13:05:52Z`, at
`fc551dc09faaa3187cd37a3a02cb94ebab0c3352`. Its protected
[Check 38140512895](https://github.com/ubugeeei-prod/vize/actions/runs/38140512895)
passed on that exact source. All four separate protected consumer workflows
also passed: [n8n](https://github.com/ubugeeei-prod/vize/actions/runs/38140512829),
[Nuxt 3](https://github.com/ubugeeei-prod/vize/actions/runs/38140512837),
[Nuxt scoped CSS](https://github.com/ubugeeei-prod/vize/actions/runs/38140512836),
and [Musea](https://github.com/ubugeeei-prod/vize/actions/runs/38140512871).
This supersedes only the pending qualification and merge status in the
[build inventory](./2026-10-11-6830-rust-build-inventory.md); its retained
historical failures, source identities, cache boundaries, and measurements
remain valid observations of their original runs.

## Comparable full Check workloads

These are completed `merge_group` executions of the original full suites.
Each baseline has 47 jobs: 30 successful and 17 intentionally skipped.
The final source has the same outcomes plus the one successful shared
producer, for 48 jobs: 31 successful and 17 intentionally skipped. The
pre-existing skipped scheduled/manual jobs are still unqualified here.

The Check envelope below is the last job's `completed_at` minus the earliest
job's `started_at`. Each job or step duration uses those same API fields.
These values include setup, transfer and scheduling within that boundary.

| Run                                                                           | Checked source                             | Check envelope |  JS job | JS test step | UI step |
| ----------------------------------------------------------------------------- | ------------------------------------------ | -------------: | ------: | -----------: | ------: |
| [38127019057](https://github.com/ubugeeei-prod/vize/actions/runs/38127019057) | `f440b7d7c1d39bec78268f45570d6499780a3e13` |        1,314 s | 1,216 s |      1,029 s |   140 s |
| [38127454609](https://github.com/ubugeeei-prod/vize/actions/runs/38127454609) | `78ac30f2f97b6f6ec681affdded5ad66cc44e61b` |        1,312 s | 1,216 s |      1,041 s |   142 s |
| [38140512895](https://github.com/ubugeeei-prod/vize/actions/runs/38140512895) | `fc551dc09faaa3187cd37a3a02cb94ebab0c3352` |        1,186 s | 1,077 s |        881 s |   145 s |

The observed complete Check is 126–128 seconds shorter. It is not an
isolated measurement of CLI sharing: the independently running JS test step
also changed by 148–160 seconds, and the checked source and native binaries
are different. The JS job remains the critical path. No CPU, RSS, p50/p90,
or causal whole-Check speedup is claimed from these three observations.

## Shared producer and transfer costs

Producer [114473394938](https://github.com/ubugeeei-prod/vize/actions/runs/38140512895/job/114473394938)
compiled the CLI in 36.71 seconds and the SFC oracle in 17.53 seconds, as
printed by the original Cargo commands. The complete build step took
63 seconds; packing took 6 seconds. The producer job took 87 seconds and
completed 94 seconds after planning. Tooling started 101 seconds after
planning, so all nested native hooks inherited that initialization delay.

All four consumers restored and authenticated the producer's original
receipt and actual bytes. Their complete restore steps took 16, 50, 10,
and 49 seconds for shards 1–4. In
[tooling shard 2](https://github.com/ubugeeei-prod/vize/actions/runs/38140512895/job/114473685732),
the Azure artifact download itself occupied 45.23 seconds; validation and
installation then took about 3 seconds. The uploaded archive was
53,184,612 bytes. This is materially more transfer wall than the retired
candidate's 4–5-second whole restorations. Provider latency remains a
variable cost; the current artifact already uses upload compression level 0
around its gzip archive.

The baseline's four CLI/oracle Cargo commands totalled 205.40 seconds of
aggregate command wall; the final producer's two total 54.24 seconds.
The 151.16-second difference is measured command wall across different
sources and workers, before the extra producer setup, packing, transfer and
validation costs. It is not saved CPU or complete CI time.

Tooling shard 1 took 735 seconds after its own initialization, compared with
707 and 684 seconds in the baselines. Shards 2–4 took 550/517/546 seconds,
compared with 562/574/466 and 557/567/485 seconds. Reduced repeated builds
therefore do not establish a faster tooling critical path in this run.

## Decision and next measurement

Retain the qualified producer and strict restoration. Preserve every full
suite, protected consumer, profile, recipe, original receipt and cache trust
boundary. Prioritize the JS critical path next: its original package tests,
real-project transport and UI conformance need separate measured boundaries
before choosing an optimization. Continue to distinguish aggregate Rust
build work from the longest parallel job.

Keep Bazel/Ninja as unevaluated alternatives. The current evidence exposes
test and transport execution plus artifact latency, rather than a missing
Rust build scheduler; Cargo and nextest already compile workspace tests once
and restore the archive into four shards. Changing build systems is not
justified by these measurements.

The pinned rust-script installation in
[PR #8632](https://github.com/ubugeeei-prod/vize/pull/8632) is a separate slice.
After its parent merged, GitHub rebased it to
`7de9e175dadae56d131d2006c4479a8c97e92ee5` on `main`. Both PRs retain native
Stack #8633, positions 1 and 2. Fresh ordinary Actions and protected delivery
for that child remain pending at this record's creation; its earlier green
head is not qualification of the rebased source.

Reproduce the complete job inventory and durations without another CI run:

```sh
gh api --paginate --slurp \
  repos/ubugeeei-prod/vize/actions/runs/38140512895/jobs | \
  jq '[.[] | .jobs[] | {id,name,conclusion,started_at,completed_at,steps}]'
```

Use the two baseline run IDs with the same command. Original Cargo command
durations and the artifact download boundary are in the complete linked
producer and consumer logs. TODO: qualify the rust-script child through its
fresh source and actual merge; measure the transport's own phase receipts
and preserve all refusal controls in any next overlap change.
