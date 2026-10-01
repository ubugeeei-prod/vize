# Isolated PR tooling shards

Tracks [#6863](https://github.com/ubugeeei-prod/vize/issues/6863). This is a
bounded execution change to the existing task-input selector, rather than a
change to which tests a PR requires.

## Measured bottleneck

The exact-head [PR Check run 36810932045](https://github.com/ubugeeei-prod/vize/actions/runs/36810932045)
selected 507 of 655 tooling files, with 107 explicit runtime files deferred to
T1 by the existing policy. The tooling job took 8 minutes 25 seconds
(03:32:32–03:40:57 UTC), including preparation. Its Node test body took
345.49 seconds: 4,733 passed, 12 skipped, zero failed. The final required
`test-report` finished at 03:41:22 UTC. These are observations of that run,
not measurements of the proposed shard execution.

[Run 36813564572](https://github.com/ubugeeei-prod/vize/actions/runs/36813564572)
again had tooling on the critical path: 8 minutes 11 seconds
(04:07:07–04:15:18 UTC). The Rust report finished at 04:13:23 UTC and the
required `test-report` at 04:15:44 UTC. Rust build, archive transfer and test
execution also exceed two minutes; tooling sharding alone cannot establish
that whole-PR latency target.

## Execution contract

- Retain the current task-input selection, its conservative unknown-input
  fallback, transitive imports, and explicit T1 runtime inventory.
- Partition the selected PR files deterministically by their existing order
  into at most four disjoint shards. Their union is exactly the selected list.
- Every shard has its own Actions checkout and runner. Execute files serially
  inside each checkout, since tests share fixture and report paths there.
- Reject missing, malformed or incomplete shard coordinates. An empty
  selection must never invoke Node's implicit test discovery.
- Merge-group uses one runner with the complete tooling suite and cannot opt
  into the PR partition. Main, scheduled, manual and release full suites keep
  their current paths. No instruction, differential, source-map or product
  acceptance gate is reduced.
- Keep the stable source-report and `test-report` aggregate chain. All matrix
  jobs must succeed; disable fail-fast and allow no failed shard to report a
  successful aggregate. The active default ruleset requires `check-js`,
  `check-vize-apps`, `fmt-rust` and `test-report`, not individual shard names.

## Independently reviewable slices

1. Add the validated matrix and serial runner provider, including disjoint and
   exhaustive selection laws and merge rejection. Existing workflow execution
   remains unchanged until a consumer uses these outputs.
2. Wire the Actions consumer on a child branch, retain aggregate/context
   parity, and validate the actual PR and protected merge-group heads.

The child depends on the provider and must be registered in a native GitHub
Stack. Exact-head checks and an actual protected-queue merge are required
before claiming delivery.

## Remaining work

Measure all shard preparation and execution times in Actions, compare the
critical path with the observations above, and report any cold-cache cost.
Do not close #6863 or claim the two-minute goal from synthetic partition laws
or a queued PR. Broader audited task-input scopes and the Rust critical path
remain separate work.
