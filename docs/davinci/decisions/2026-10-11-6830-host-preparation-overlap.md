# Overlap independent host preparation with the authenticated n8n baseline

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original-project qualification starts each pinned Oxlint host's install,
transport controls and HTML controls while preparing the n8n baseline and
historical reference. Each host waits for an authenticated readiness packet
before its original n8n replay. Keep both host processes, their sequential
commands, all full-corpus inputs, thresholds, native receipts and deadlines.
Keep `--preparation-mode sequential` as the same-source comparison control.

This follows the
[protected build result](./2026-10-11-6830-protected-build-results.md).
It changes the next scheduling boundary only. Historical measurements and
cache trust boundaries in that record remain observations of their original
sources. No product behavior or release qualification changes here.

## Measured critical path

Protected [Check 38140512895](https://github.com/ubugeeei-prod/vize/actions/runs/38140512895)
qualified `fc551dc09faaa3187cd37a3a02cb94ebab0c3352`. Its
[JS job](https://github.com/ubugeeei-prod/vize/actions/runs/38140512895/job/114473394967)
took 1,077 seconds, including 881 seconds in `Test JS packages` and 145 seconds
in UI conformance. The complete protected workload and its two earlier
comparisons are retained in the linked build result.

The source-bound artifact
`oxlint-original-locations-fc551dc09faaa3187cd37a3a02cb94ebab0c3352-1-pr-js-packages`
(artifact ID `11703471223`) contains the original controller and per-host
`phase-walltime.json` receipts. Within that one qualification:

| Controller phase                              | Measured wall time |
| --------------------------------------------- | -----------------: |
| Baseline and historical reference preparation |          194.263 s |
| Both original host workers                    |          378.905 s |
| Final native and diagnostic aggregation       |           10.176 s |
| Complete original-project qualification       |          588.179 s |

The independent mixed-directory branch already overlaps these phases; its
161.207 seconds are not added to the critical path. Root `test:js` took
267.003 seconds in a separate earlier action step and remains unchanged.

| Host          | Install | Transport |      HTML | n8n replay |     Total |
| ------------- | ------: | --------: | --------: | ---------: | --------: |
| Oxlint 1.78.0 | 0.471 s |  46.211 s | 135.495 s |  195.388 s | 378.180 s |
| Oxlint 1.86.0 | 0.470 s |  46.932 s | 135.485 s |  193.369 s | 376.858 s |

The original host preparation is about 182 seconds after the baseline.
Overlapping that work exposes about 182 seconds of arithmetic opportunity.
This is a projection, **not a measured speedup**: shared CPU, filesystem and
process overhead can change the observed result. The full Actions comparison
and protected qualification of the new source remain unfinished.

## Readiness and failure ownership

The parent publishes one complete success or failure packet by atomic rename.
Success binds the current source head, tree and working diff, toolchain,
physical staged binary, the whole original replay descriptor, joined baseline
vectors and native-call ledger, and the complete historical reference receipt.
Each host checks the linked reference source files and both runtime and saved
bundle bytes before replay. Stale, missing, malformed or changed evidence
refuses qualification. A failed baseline releases waiting hosts through the
failure packet; it cannot become an empty successful replay.

The original outer join still contains exactly the mixed-directory and n8n
branches. The n8n branch joins its baseline and both complete host outcomes
before the existing upload or cleanup can run, including on failure or
cancellation. Each worker's original 1,800,000 ms process-group deadline owns
its preparation, readiness wait and n8n replay together. Individual install,
transport and HTML commands retain 180,000 ms and 64 MiB output bounds.
Readiness polling does not extend those deadlines.

The original six input-law files remain in the production baseline campaign.
Additional real-process laws cover overlap and the sequential control,
actual host-worker command membership and refusal, baseline and host failures,
stale and changed whole receipts, parent cancellation with descendants, and a
host deadline expiring while its baseline sibling still runs. Controlled
process fixtures qualify scheduling and custody refusal only; they are not
native, n8n-diagnostic or performance acceptance.

## Same-source Actions comparison

Manually select `n8n-preparation` in
[CI walltime comparison](../../../.github/workflows/ci-walltime-benchmark.yml).
The opt-in job checks out one exact SHA on the existing 32-vCPU runner and
builds the original source native binary and bridge once. Both full controller
arms use the same partitioned baseline and two concurrent original hosts.
Only `--preparation-mode sequential` versus `overlap` changes. Ordinary PR
events and `component=all` do not start this extra workload.

The second arm still runs after an ordinary first-arm failure, while
cancellation remains authoritative. Neither arm ignores failure. The final
upload always retains both full original campaigns, timing receipts, native
build receipt and process packet, and original binary stdout/stderr streams.
Compare matching `source` and `binarySha256`, passed qualification packets,
all host phases, `baselineWait`, and total wall time before reporting a gain.

```sh
gh workflow run ci-walltime-benchmark.yml \
  --ref <reviewed-branch> -f component=n8n-preparation
```

## First completed same-source result

[Actions 38144141051](https://github.com/ubugeeei-prod/vize/actions/runs/38144141051)
completed both full arms on `03c034f34a6f7a4817726dcb087270bdb5211458`, tree
`989a5ffe3240f2c4fce6c4d4197836e4a2557410`, with an empty working diff.
Both used the one source-built native SHA-256
`2e953850ee1e763fde77e5122a40a1bf56746918e1ce9d331f25250787faeaa3`,
Rust 1.99.0, Cargo 1.99.0 and Node 24.14.0. Build setup is outside the paired
controller timings; it ran once for both arms.

| Complete controller phase                                  | Sequential |   Overlap |
| ---------------------------------------------------------- | ---------: | --------: |
| Baseline and historical reference preparation              |  197.272 s | 206.126 s |
| Both complete pinned host workers                          |  381.293 s | 408.558 s |
| Final native and diagnostic aggregation                    |   10.317 s |  10.346 s |
| Complete original-project qualification, including cleanup |  595.993 s | 423.791 s |

This paired observation reduces the complete controller wall time by
172.202 seconds, or 28.9%. It is one sequential-then-overlap comparison,
not a p50/p90 estimate or a complete Check workflow result. Contention also
increased baseline, transport and HTML durations; the recorded benefit comes
from overlap despite that cost, rather than faster individual commands.
The independent mixed-directory branch took 163.268 and 184.100 seconds.
Readiness wait remained within the unchanged host deadline: 0.907/0.885 seconds
for sequential hosts and 2.375/1.599 seconds for overlap hosts.

Both arms passed with equal source, toolchain, physical native binary,
n8n fixture commit/tree, requirements configuration, projected layer,
complete diagnostic totals, limits, 1,369 original files, 291,633 native
events and 34 native processes. Both original hosts retained all six replay
modes and the same complete diagnostic vector SHA-256
`cf980b3e52dab3ee9d2124eb9cc77d3a80e67f5a25ec2e38e506d649e1bb06f0`.
Each retained 74 wrapper calls, 18 batch calls, 99 transport processes,
41 original HTML cases, 36 whole native calls and 57 whole host phases.
Historical reference commit, complete physical source records and limits
matched. Native-call ledger bytes differ because each arm owns fresh process
and staging identities; every original ledger is retained and qualified.

The retained artifact is
`ci-walltime-n8n-preparation-03c034f34a6f7a4817726dcb087270bdb5211458`,
ID `11704466680`, 441,993,007 bytes, GitHub digest SHA-256
`8fc413ffcd0ee757e4490a74f3d02fe31f3abe205442cc7ed510c4b288002afd`.
Its sequential staging directory is `run-7v1V5B`; overlap is `run-riamtB`.
The controller and per-host phase receipts, whole qualification packets,
readiness/outcome packets and source-build receipt remain in that artifact.

TODO: carry the final source through fresh exact-head Actions and the
complete protected queue before calling the optimization delivered.
Inspect trusted main cache seeding separately: merge-group restores remain
read-only, and oversized tooling targets must not be aliased into a smaller
cache role. The #6830 three/six-minute targets and #6863 remain unfinished.
