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

The first hosted source check at `06c5822b5359be312a08218a2534149bd930403e`
rejected four unhandled Node test registration promises under the existing
zero-warning gate. Marking the registrations explicitly `void` preserves the
same tests; fresh exact-head Actions remain required.

The corrected source `795e3d312e68f45bbba0e03b9dc4bc0be359f181` passed
[source Check 37121764436](https://github.com/ubugeeei-prod/vize/actions/runs/37121764436).
Its actual tooling report identifies the hosted PR merge-tree HEAD
`bfc7ea1f882985a18132285d6ca6cc05f059d372` and prints the complete per-product
and per-crate disagreements before its successful artifact check. Those
receipts remain historical after replay onto the actually merged Document
commit `6f36ea1aee9598ae4a15fb7b025171f7ecd4a381`; fresh source checks and a
protected candidate are still required. The replay regenerates only this
change's canonical artifact set and preserves every incoming resolved site.

Source `ce7373eec7efbbadf22d2eb7467fc027a61f864f` also passed
[Check 37124289409](https://github.com/ubugeeei-prod/vize/actions/runs/37124289409).
Its actual mandatory report names hosted PR merge-tree HEAD
`986daeacae5f0e153cb012a51977d6fa66dbf557` and accepts the complete 20-file
artifact set. The remaining accepted Vue, JSX and For predecessors then
actually merged. Replay onto their literal final main
`24f31e0608c83a42aef0a7522244f38037b1534c` regenerates the same policy and
independently preserves all 19 authoritative tables, removing only the 14
diagnostic-only shards. Source and regression implementation are unchanged;
the final replay still needs fresh hosted source checks and protected merge.
