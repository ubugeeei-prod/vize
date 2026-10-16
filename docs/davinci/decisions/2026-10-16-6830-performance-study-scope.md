# Run repeated performance studies outside the PR loop

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The ordinary PR budget is five minutes. Repeated base/head measurements are
explicit performance qualification, separate from the mandatory product tests
and Clippy described in [the Rust gate policy](./2026-10-16-6830-required-rust.md).
This changes when the studies run; their measurements, assertions, retained
samples and source custody remain intact.

| Study                         | Trigger           | Comparison authority                                                                                         |
| ----------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------ |
| Compact collections           | Manual            | Required exact ancestor `base_sha`, dispatch head                                                            |
| Doctor filters                | Manual            | Required exact ancestor `base_sha`, dispatch head                                                            |
| Type source snapshots and CLI | Manual            | Required exact ancestor `base_sha`, dispatch head for both jobs                                              |
| Text boundary                 | Weekly and manual | Current source compared with standard UTF-8/filesystem operations inside the same binary                     |
| Davinci Canon scaling         | Manual            | Existing frozen original control, successor, tree and manifest inputs; optional published-artifact authority |

Paired source comparisons deliberately have no default scheduled baseline.
The supplied baseline must be a full commit SHA and an ancestor of the selected
head. The CLI snapshot study uses that same explicit baseline instead of a
moving `origin/main`. The Canon workflow's no-input mode remains a comparison
of its recorded historical commits; it is not evidence for current main.
Current original400 qualification continues to require its existing frozen-cut
inputs, with public-artifact authority validated when supplied.

The studies contain useful checks that are not equivalent to ordinary Cargo
tests: complete base/head type-world signatures, stock TypeScript/native Corsa
CLI diagnostic comparisons, and original400 LSP packets with invalidation
controls. Those remain available as explicit qualification. Routine source
tests still cover filter matching, UTF-8 and filesystem errors, graph cycle
ordering, scopes, snapshot edits and concurrent consumers. Full merge-queue
Cargo tests retain native process reuse, editor/CLI diagnostic agreement and
editor dependency invalidation; current native LSP feature and navigation
checks also remain required under their existing selection policy.

The successful first-attempt runs below were read from GitHub Actions on
2026-10-16. Workflow elapsed time includes queue/setup; job time excludes the
initial workflow queue.

| Study                 | Workflow elapsed | Longest job | Evidence                                                                      |
| --------------------- | ---------------- | ----------- | ----------------------------------------------------------------------------- |
| Compact collections   | 4m38s            | 2m54s       | [38471385895](https://github.com/ubugeeei-prod/vize/actions/runs/38471385895) |
| Doctor filters        | 3m28s            | 1m46s       | [38471385968](https://github.com/ubugeeei-prod/vize/actions/runs/38471385968) |
| Text boundary         | 2m55s            | 1m09s       | [38471385956](https://github.com/ubugeeei-prod/vize/actions/runs/38471385956) |
| Type source snapshots | 7m38s            | 7m31s       | [38525035971](https://github.com/ubugeeei-prod/vize/actions/runs/38525035971) |
| Canon original400     | 16m25s           | 13m12s      | [38608756719](https://github.com/ubugeeei-prod/vize/actions/runs/38608756719) |

The last Canon job spent 6m27s building its baseline and 5m01s building its
successor before a 1m01s comparison. The snapshot CLI job spent 2m23s building
both CLIs and 1m58s on repeated paired measurements. These are historical
observations of their recorded commits, not a guaranteed current timing.
