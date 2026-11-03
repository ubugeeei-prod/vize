# Davinci completion metrics — review input

Snapshot: 2026-09-26. Repository inventory base:
[`aac2e1bf8bf2796870ac5145067a981e6cacf588`](https://github.com/ubugeeei-prod/vize/tree/aac2e1bf8bf2796870ac5145067a981e6cacf588).
Measurements below name their own revisions; they do not describe an unselected
release candidate. P6-10 remains open: its P5-14 start gate and the finished-substrate
review have not passed. This is the collected input to that review.

## Target authority

[Charter #35](./README.md#decided-positions) requires improved compile throughput
and peak memory, halved keystroke-to-diagnostics p95, fact adoption beyond the
neutral-core majority, source maps on every backend, and matrices without orphans.
[Budgets](./plan/budgets.toml) own the pinned measurements and ratchets.
A passing regression ceiling alone does not establish an improvement target.
Unrecorded measurements remain unrecorded; no target is changed here.

## Reconciliation

| Metric                             | Pinned target or baseline                                                                                                                   | Recorded achievement and scope                                                                                                                                                                                                                                                                                                               | Review status / blocker                                                                                                                                                                                                                                                                                       |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Compile throughput                 | Improve the phase-0 baseline; compile envelope `329.2 ms`, cold median, five runs, 15k SFCs, Blacksmith 32 vCPU, 5% regression tolerance    | [Envelope definition](./plan/budgets.toml); the committed [tool report](../../tools/benchmarks/results/tool-benchmark-latest.json) instead measures 3,000 SFCs at `f9cc50ee2e2ff14cd120241cd87a0162446df65b` ([run 35072615669](https://github.com/ubugeeei-prod/vize/actions/runs/35072615669)), so it is not a replacement 15k measurement | **Unmeasured on the completed substrate.** P3-6 needs repeatable reference-runner native/retained comparisons and a pinned promotion threshold; the [benchmark record](./plan/phase-3-records/p3-6-benchmark.md) explains remaining comparisons. `wall_p50_ns = 0` entries remain report-only, not successes. |
| Compiler peak memory               | Improve peak memory; `[allocation_peak]` pins deterministic stage-window bytes while per-bench `rss_peak_bytes` remains report-only         | [Budget definitions](./plan/budgets.toml) distinguish live allocator bytes from process RSS; [Vapor allocation recovery](./plan/phase-3-records/p3-6-emission-allocations.md) records allocation calls, not a full compile peak-RSS result                                                                                                   | **Final improvement unmeasured.** Preserve the original windows and obtain a comparable compiler peak-memory result on the finished substrate. Resident RSS and allocation-call reductions cannot substitute for this metric.                                                                                 |
| Peak resident memory               | Improve the baseline; nine-file TS-44 baseline `292.3 MiB`, ceiling `337 MiB`                                                               | Observed `286.4 MiB` at the R4 head maximum of three runs; same nine-file LSP/Corsa fixture and sampler, [R4 below](#r4--frozen-production-runtime-refresh)                                                                                                                                                                                  | **Nine-file ceiling met; broader completion unproven.** P5-11b still needs both largest corpus projects and its agreed comparable before/after evidence.                                                                                                                                                      |
| Keystroke → diagnostics p95        | Halve the P5-11a baseline `30.1 ms`; the existing regression ceiling is `46 ms`                                                             | Observed `29.9 ms` at the R4 head, nearest-rank p95 of 60 exact-version TS 2322 edits, [R4](#r4--frozen-production-runtime-refresh)                                                                                                                                                                                                          | **Halving target missed in this fixture.** Half of the recorded baseline is `15.05 ms`; this arithmetic is the target, not an estimated result. P5-11b large-project measurements and latency remediation remain open.                                                                                        |
| Linter fact adoption               | Charter starting point `26/345`; neutral-core majority required                                                                             | Base [rule-parity inventory](./plan/rule-parity.md): 249 rules, 91 neutral-core candidates, 23 Croquis readers, 40 markup-facade rules                                                                                                                                                                                                       | **Majority not established.** Croquis reads and markup-facade membership are different from declared fact consumption; the old 345-rule denominator cannot be silently replaced. P4-3/P4-7/P4-15 and TS-35 need a reconciled fact-consumer numerator and neutral-core denominator.                            |
| Source maps                        | All three backends; static-template coverage ≥95%, rewritten identifiers / boundaries 100%, accuracy 100%; frozen legacy SFC floor retained | Committed [TS-31 report](./plan/ts31-sourcemap-coverage.json): DOM and Vapor `35/35`, `13/13`, `11/11`; SSR `35/35`, `11/11`, `11/11`; SFC recovery `11/11`, rewritten statements `19/19`, all exact                                                                                                                                         | **Recorded battery meets every pinned row.** This is the committed battery, not all authored tokens in the corpus. Run TS-31 again on the final candidate; new native shapes need their own exact mapping proofs. [P3-9 record](./plan/phase-3-records/p3-9.md).                                              |
| Consumption / rule-parity matrices | Every computed group has ≥1 consumer or an explicit gate; no orphan rule or undeclared access                                               | [Croquis shards](./plan/croquis-consumption.md), [rule matrix](./plan/rule-parity.md), [migration shards](./plan/consumer-migration-surfaces.md); rule registry reports `0 unregistered`, with six Musea rules explicitly outside SFC/JSX                                                                                                    | **Partial mechanical evidence.** TS-12 source staleness is enforced, but TS-34/35/46 still need final production demand-graph and adoption reconciliation. Croquis product-use totals include test code and cannot establish production fact coverage.                                                        |

## R1 — Production LSP resource measurement

The successful [TS-44 job 108388832282](https://github.com/ubugeeei-prod/vize/actions/runs/36236392245/job/108388832282)
ran at measurement head
`e1bee278790a2d78742d21f5000fb8c3b3932131`, containing the production
resident implementation revision `56a65d3d314975ea29aa66f4f0f8cd942d67b8d0`. It used the existing
`linux-x64-ci` preset, three fresh `ci-opt` servers, twenty edits each and
a sixty-second idle window. Server, Corsa and tsgo RSS were sampled together
every 50 ms. The workload is the nine-file TS-45 `real-vue` fixture.

| Metric                    |    Result | Unchanged ceiling |
| ------------------------- | --------: | ----------------: |
| Cold start, median        |    4.9 ms |              9 ms |
| First diagnostics, median | 1118.0 ms |           1398 ms |
| Keystroke p95             |   31.1 ms |             46 ms |
| Peak process-tree RSS     | 288.8 MiB |           337 MiB |
| Idle process-tree RSS     | 276.6 MiB |           316 MiB |
| Idle CPU, maximum         |     0.15% |                1% |
| Server binary size        |  46.7 MiB |            59 MiB |

[Committed raw samples](./plan/completion-metrics/r1-resource-linux-x64-ci.json)
preserve the measurement after artifact expiry. Their source is the
[`resource-linux-x64-ci` artifact](https://github.com/ubugeeei-prod/vize/actions/runs/36236392245/artifacts/10903579774)
(artifact ID `10903579774`). The original method and baseline
remain in [P5-11a](./plan/phase-5-records/p5-11a.md).

The same workflow's
[synthetic resident job](https://github.com/ubugeeei-prod/vize/actions/runs/36236392245/job/108388832528)
passed its 10k-input probe. Its
[`resident-resource-linux-x64-ci` artifact](https://github.com/ubugeeei-prod/vize/actions/runs/36236392245/artifacts/10903969071)
is separate evidence: fixture alpha inputs, isolated resident database, ten-second
idle, no LSP/Corsa. See [P5-4b progress](./plan/phase-5-records/p5-4b-progress.md).

The strict all-provider job in that run did not complete; see [R3](#r3--production-scale-observations).
No wider workload may borrow the nine-file ceiling or establish the
largest-project/halving acceptance.

## R2 — Production source-snapshot refresh

After immutable source-world snapshots and editor overlays landed on the
production branch, the same nine-file preset passed again in
[job 108398702457](https://github.com/ubugeeei-prod/vize/actions/runs/36240038670/job/108398702457).
Measurement SHA: `a4e59f078ce1bb42d27681938bc591bce3b12e24`;
producer SHA: `049358b200baa82d99b04c40f95aa68d0e8fae7c`.
The required nine-file, synthetic resident and bounded production jobs all
succeeded in this workflow. Its
[artifact 10905303903](https://github.com/ubugeeei-prod/vize/actions/runs/36240038670/artifacts/10905303903)
is preserved as [retained samples](./plan/completion-metrics/r2-resource-linux-x64-ci.json)
without replacing R1's history.

| Metric                    |    Result | Unchanged ceiling |
| ------------------------- | --------: | ----------------: |
| Cold start, median        |    5.2 ms |              9 ms |
| First diagnostics, median | 1116.9 ms |           1398 ms |
| Keystroke p95             |   29.3 ms |             46 ms |
| Peak process-tree RSS     | 290.5 MiB |           337 MiB |
| Idle process-tree RSS     | 281.2 MiB |           316 MiB |
| Idle CPU, maximum         |    0.167% |                1% |
| Server binary size        |  47.0 MiB |            59 MiB |

The 15.05 ms halving target remains unmet. This refresh does not establish a
large-project or completed-substrate improvement claim.

## R3 — Production scale observations

The [corrected production job 108398702433](https://github.com/ubugeeei-prod/vize/actions/runs/36240038670/job/108398702433)
passed at resource SHA `a4e59f078ce1bb42d27681938bc591bce3b12e24`,
producer SHA `049358b200baa82d99b04c40f95aa68d0e8fae7c`.
[Artifact 10905478324](https://github.com/ubugeeei-prod/vize/actions/runs/36240038670/artifacts/10905478324)
contains the full observation and the separate two-file edit oracle.

This one session opens 10,000 SFCs and checks document-symbol residency for every
file, but warms only **16 distinct alpha providers**. Its 32 body edits and
public-interface oracle run on a two-file prelude. It records 1120.8 MiB peak RSS,
1039.5 MiB final RSS after ten seconds without keyboard input, and 143.835% summed
process-tree CPU during that window, over an 83.569-second session. Summed CPU can
exceed 100% because the process tree uses multiple cores. Background work is still
active; keyboard inactivity does not establish settled idle. This bounded
observation is record-only, not full 10k alpha coverage or P5-11b acceptance.

The corrected sampler retains observed PID/birth-time CPU counters through exit.
The earlier sampler lost retiring Corsa processes' counters; its 53.879% is an
uncorrected historical sample, excluded from current reconciliation. No numeric
ceiling is invented for this broader workload.

The prior [strict all-provider job](https://github.com/ubugeeei-prod/vize/actions/runs/36236392245/job/108388832165)
was cancelled after approximately sixty minutes. Its last checkpoint reached
2,304 of 10,000 open and metadata-warmed files at 7138.5 MiB. There is no completed
peak/idle artifact for that scope. Full provider warming and background resource
behavior remain concrete scale blockers. [Resource PR #6807](https://github.com/ubugeeei-prod/vize/pull/6807)
owns the [detailed production record](https://github.com/ubugeeei-prod/vize/blob/1fc3a8685c1dcfe3e00448b2ed2fd22a0953aa25/docs/davinci/plan/phase-5-records/p5-4b-production-lsp-resources.md)
and [full corrected JSON](https://github.com/ubugeeei-prod/vize/blob/1fc3a8685c1dcfe3e00448b2ed2fd22a0953aa25/docs/davinci/plan/phase-5-records/p5-4b-production-lsp-resources.json).

The later frozen runtime is measured separately in R4. The completed 049
measurements above remain exact history and are not transferred to a newer head.

## R4 — Frozen production-runtime refresh

[Run 36240385542](https://github.com/ubugeeei-prod/vize/actions/runs/36240385542)
completed all three resource jobs successfully at resource SHA
`1fc3a8685c1dcfe3e00448b2ed2fd22a0953aa25`, containing frozen producer
`ad5c60f710547d3f6431d9a12224616316b4449b`. This is the measured runtime revision;
subsequent source changes require their own evidence.

The unchanged nine-file TS-44 preset passed in
[job 108399647923](https://github.com/ubugeeei-prod/vize/actions/runs/36240385542/job/108399647923).
[Artifact 10905870609](https://github.com/ubugeeei-prod/vize/actions/runs/36240385542/artifacts/10905870609)
is retained as [R4 samples](./plan/completion-metrics/r4-resource-linux-x64-ci.json).

| Metric                    |    Result | Unchanged ceiling |
| ------------------------- | --------: | ----------------: |
| Cold start, median        |    5.6 ms |              9 ms |
| First diagnostics, median | 1116.7 ms |           1398 ms |
| Keystroke p95             |   29.9 ms |             46 ms |
| Peak process-tree RSS     | 286.4 MiB |           337 MiB |
| Idle process-tree RSS     | 277.5 MiB |           316 MiB |
| Idle CPU, maximum         |    0.217% |                1% |
| Server binary size        |  47.0 MiB |            59 MiB |

The [bounded production job 108399647897](https://github.com/ubugeeei-prod/vize/actions/runs/36240385542/job/108399647897)
retains the exact R3 workload: 10,000 document-residency witnesses, sixteen alpha
providers and the two-file edit prelude. Its
[artifact 10904744181](https://github.com/ubugeeei-prod/vize/actions/runs/36240385542/artifacts/10904744181)
records 1118.9 MiB peak, 1005.9 MiB final RSS and 144.8501% cumulative process-tree
CPU during ten seconds without keyboard input, over an 85.510-second session.
The owner maintains the full record in [PR #6807](https://github.com/ubugeeei-prod/vize/pull/6807).
The observation remains record-only, with active background work and no invented
broader ceiling. The strict all-provider scope, large-project acceptance and
15.05 ms halving target remain unmet.

## Review still required

- P5-14 must pass before the P6-10 finished-substrate review.
- Record comparable compile and large-project resource results, explaining each
  miss without changing the targets.
- Reconcile fact consumption with the historical adoption definition and current
  rule identities; preserve explicit gates for intentional non-consumers.
- Refresh source-map and matrix evidence at the final candidate SHA.
- Record the maintainer's review separately. No reviewer acceptance or v1 decision
  is recorded by this input package.

The [v1 evidence package](../release/level-v1-evidence.md) maps these inputs to
the release checklist and preserves its remaining blockers.
