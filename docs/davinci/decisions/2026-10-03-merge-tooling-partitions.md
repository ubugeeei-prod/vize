# Complete merge tooling partitions

Tracks [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and
[#6863](https://github.com/ubugeeei-prod/vize/issues/6863). This revises the
one-runner merge restriction in the [PR partition record](./2026-10-01-tooling-pr-shards.md).

## Observed bottleneck

Protected merge-group [Check 37112347462](https://github.com/ubugeeei-prod/vize/actions/runs/37112347462)
ran the one tooling worker from 09:15:52 to 09:32:40 UTC: 16 minutes 48 seconds.
Its full test step took 15 minutes 15 seconds (09:17:19–09:32:34).
The source Rust report had already completed at 09:31:27. These are actual
baseline measurements, not projected savings or proof of a two-minute target.

## Execution contract

- Keep the full discovered merge suite, including every deferred PR runtime
  file. Validate the complete plan before partitioning. Missing, duplicate or
  unknown files and malformed or incomplete shard coordinates remain fatal.
- Reuse the existing deterministic maximum-four partition. Each Actions
  runner owns a separate checkout and executes its files serially. The union
  is exactly the original full list, with no repeated file.
- Every runner regenerates its plan from the shared comparison base with the
  actual event tier. On baseline `44533d11de`, the 709 full files partition as
  178/177/177/177, with zero deferred files.
- Keep native preparation, tool-layout verification, source-built CLI receipts,
  pinned plugin runtime and required TSGO dependencies in every runner. The
  merge test step retains `VIZE_LSP_BIN` and `VIZE_LSP_REQUIRE_SOURCE_BUILD`.
- The new `test:scripts:planned` task consumes the validated plan. Ordinary
  `test:scripts`, main, scheduled, manual and release execution stay intact.
  Matrix fail-fast stays disabled; the required source report waits for the
  whole matrix and rejects any failed or missing job.
- Coordinated standalone native SSR/setup captures use the first tooling
  worker, so their mandatory current-source runtime proof executes once.
  Their own source actions and assertions remain separate feature changes.
- Full merge Rust tests, differential/feature recipes, all 100 instruction
  gates and ratchet budgets are unchanged. The live main rules require
  `check-js`, `check-vize-apps`, `fmt-rust` and `test-report`; none changes.

The single verified narrative `docs/davinci/plan/completion-2026-10-03.md`
is exempt from the PR Rust fallback. It has no compiled Rust consumer, guarded
by an actual source-input scan. Its docs/tooling validation remains selected.
Every other plan input, including new Markdown contracts, remains conservative;
the actual compiled complexity/key/fact Markdown and budgets still select Rust.
Merge groups retain the complete Rust suite even for this prose-only change.

## Acceptance and remaining work

Focused selector, real full-file union, malformed-plan, source-receipt and
workflow aggregate laws pass locally. Exact-head Actions and actual protected
merge-group acceptance are required before delivery. Measure real worker
preparation and critical-path times after merge; partitioning alone does not
prove balanced runtime or the two-minute latency goal. No roadmap is closed
from synthetic partition laws or an unmerged queue entry.
