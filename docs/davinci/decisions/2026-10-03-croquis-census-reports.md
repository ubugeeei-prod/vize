# Fresh Croquis census reports (2026-10-03)

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original HTML Document provider (#7567) and Vue root-text provider
(#7548) each had source checks pass and then could not compose with the
healthy event-provider queue prefix. An exact `git merge-tree` comparison
against candidate `5c678dda03e0abd333bb50a4b8465094342ee54a` found the sole
conflict in `docs/davinci/plan/croquis-consumption/vize_l1.md` for each.
Independent native implementation changed raw counts of unrelated symbols
named `Span`, creating a shared numeric edit in the same generated shard.
Neither comparison established a production-source conflict.

The consumption scanner and product enumeration remain unchanged. Resolved
legacy product sites and non-product imports keep their byte-exact committed
shards and strict missing/changed/stray artifact check. Orphan products still
require a real demand gate; raw text matches cannot satisfy that gate.

Raw naive-grep disagreements move to the mandatory `--check` report and the
existing `--summary` command. Every check prints the analyzed working-tree
HEAD, complete product totals and exact disagreeing crate identities with
their resolved/grep counts. The matrix test forwards the report into Actions
logs, including checks that subsequently reject stale committed consumption.
The report describes the analyzed working tree; it does not certify an
uncommitted tree as identical to its HEAD. Successful hosted source and merge
checks provide the immutable revision context for their own reports.

Generated shards with only raw grep diagnostics are removed by the existing
strict artifact-set writer. A new or changed actual legacy consumer still
changes its committed resolved shard. The generated index explains the
diagnostic producer and retains the scanner's existing inference limits.
Fixture inventories, original source inputs, differential output oracles,
required CI contexts, instruction budgets and product defaults do not change.

Regression controls cover independent same-crate grep additions, grep-only
crates, real resolved/non-product changes and the actual check producer's
source-qualified report. Hosted exact-head checks, protected composed checks
and actual merge remain required. This decision records a queue-conflict fix,
not a measured whole-gate latency improvement or completed Davinci product.
