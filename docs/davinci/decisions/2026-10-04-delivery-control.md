# Vue Fes delivery control (2026-10-04)

Issues: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239),
[#6826](https://github.com/ubugeeei-prod/vize/issues/6826),
[#3957](https://github.com/ubugeeei-prod/vize/issues/3957).

The maintainer requested completion of the open PRs, Davinci and outstanding
issues, improved existing-product quality, frequent releases and a 10x typechecker
speed target before Vue Fes Japan on October 24. These instructions extend the
existing order and acceptance records; they do not waive native product gates.

## Delivery rules

- Prioritize actionable third-party reports. Preserve the reported input in the
  regression corpus and add the reporter's verified public identity as a
  `Co-authored-by` trailer. Source commit attribution and final squash attribution
  are checked separately: a repository squash policy can discard a body.
- Use `wt` isolated worktrees and sub-agents for independent work. Existing
  worktrees can contain another active session's edits; preserve those edits and
  recheck remote heads before branch updates.
- Audit every existing PR through a concrete disposition. Ready code requires
  exact-head Actions, protected queue acceptance and actual merge. Record a
  substantive blocker and continue its repair when code is unfinished.
- Register dependent slices as GitHub native Stacks and verify their number and
  order. Merge the accepted prefix through the protected queue, then rebase and
  retarget remaining children against actual merged main. Independent PRs use
  squash auto-merge after source checks pass. Remove known-red candidates promptly.
- Release ready existing-product fixes frequently through the supported release
  command. Verify terminal publication and registry/editor visibility; neither a
  merged PR nor a tag proves publication.

## Reporter attribution through the queue

The repository previously used `PR_TITLE` and `BLANK` for squash messages.
Protected merge `78866102` (#7691) still retained the distinct community reporter
Danila Poyarkov automatically, but `298f8794` (#7700) omitted an explicit trailer
whose identity was already the primary author. A source trailer therefore cannot
establish that the final message contains the requested attribution.

Use `PR_TITLE` with `COMMIT_MESSAGES` for squash merges. Each source change carries
the relevant reporter's verified public identity as `Co-authored-by`; preserving
source messages gives the queue an explicit attribution record, including when
the reporter is also the primary author. Conventional titles, protected checks,
Stack order and queue admission stay unchanged. The setting can be restored to
`BLANK`, but existing signed history is never rewritten. Previously generated
queue candidates remain historical; inspect each newly generated candidate and
its actual final merge for the expected trailer before claiming attribution.

## Typechecker measurement and remaining work

The target is 10x the current typecheck throughput with the same diagnostic
behavior. Before claiming it, identify the exact baseline and candidate source,
toolchain, corpus, backend, worker count, cold/warm state and measurement protocol.
Compare repeated Actions measurements under those same conditions. Report
projection-only or import-scan improvements with their actual scope; they do not
establish a 10x whole-project result.

Current investigations target repeated dependency parsing in a shared immutable
type-source snapshot and repeated lexical import collection in one CLI session.
Snapshot invalidation, in-memory root edits, resolution ownership, traversal
limits and diagnostic parity remain acceptance conditions. The 10x target is
unfinished until a complete workload measurement demonstrates it.

The [native completion ledger](../plan/completion-2026-10-03.md) is a dated source
snapshot. New source-qualified merges extend it without relabeling historical
proof. A fixture-history issue closes only after all of its requirements are
accounted for in an accepted record; native product replacement remains a separate
gate. Vue templates and every Vue dialect, JS/TS and JSX/TSX retain their existing
scope and no-legacy-shortcut rule. Unfinished implementations and skipped or
cancelled checks must be reported as unfinished or unknown.
