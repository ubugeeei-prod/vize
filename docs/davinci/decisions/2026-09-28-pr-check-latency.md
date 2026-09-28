# PR Check latency measurement (#6861)

The Rust archive split is live, so a successful Rust shard alone no longer
describes the time a maintainer waits for an ordinary PR. For the T0 target,
measure the top-level `Check` workflow from `created_at` to `updated_at` after
successful completion. Exclude failed, cancelled, pending and rerun attempts;
keep their separate failure and capacity evidence. Use linear interpolation at
rank `(n - 1) * p` for p50 and p90. This measures the complete required Check
workflow, including the slowest selected job and runner wait. It does not
include unrelated review or merge-queue time.

On 2026-09-28, the 28 successful first-attempt `pull_request` Check runs in
the latest 100 workflow runs (created 08:09–08:56 UTC) measured **p50 609 s
(10.15 min)** and **p90 707.1 s (11.79 min)**, range 531–1195 s. This is a
current burst sample, not a long-term service-level estimate. It misses the
roadmap's 3/6-minute T0 target and the maintainer's two-minute preference.
Keep #6830 open for the latency reduction and remeasure after input selection
and archive-transfer changes.

The source is GitHub Actions workflow `237041397`. Fetch
`/repos/ubugeeei-prod/vize/actions/workflows/237041397/runs?per_page=100`,
select `event == pull_request`, `conclusion == success`, `run_attempt == 1`,
and `created_at >= 2026-09-28T08:00:00Z`, then take the 28 eligible runs.
The exact run IDs are:

```
36400405339 36399850861 36399787431 36399672512 36399541366
36399323994 36399306964 36399268765 36399221309 36399183619
36399064538 36399061622 36398887003 36398676566 36397787421
36397764794 36397753445 36397388833 36397274182 36397218809
36396631879 36396389694 36396199926 36396178046 36396055394
36395936243 36395854562 36395845933
```

The Rust build/execution split is observable in the job steps. In ordinary
[Check 36399850861](https://github.com/ubugeeei-prod/vize/actions/runs/36399850861),
affected-crate Clippy took 42 s, archive build 76 s, doctests 13 s, and the
four archived test shards ran for 86–120 s each in parallel. Their artifact
downloads took 29–32 s. The whole Check still took 585 s because selected
tooling ran for 371 s after its CLI build. In
[Check 36399306964](https://github.com/ubugeeei-prod/vize/actions/runs/36399306964),
the same Rust steps took 42 s, 78 s and 13 s, but one of four downloads of the
2,931,270,455-byte archive took 580 s versus 28–29 s for siblings. Its actual
shard test then took 87 s; the whole Check took 1064 s. These are phase
observations from two source-bound runs, not p50/p90 claims for each phase.

Do not count a selected-out Rust job as a zero-second Rust build. For future
phase distributions, stratify by the affected-crate selection and retain
archive bytes, transfer time, build time, and test time separately. #6862
owns archive transfer/partition changes; #6863 owns tooling input selection.
