# Retained template reads for the existing type checker

Date: 2026-10-04. Tracking: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).

## Decision

Reuse the existing per-compile `SimpleExpressionNode.js_ast` in the SFC
import-retention visitor when its `raw` equals the node's current `content`
and the existing `js_module_compatible` gate accepts it. That gate preserves
the older wrapped parser's strict module goal; TypeScript-only, strictness,
sloppy-literal and HTML-comment differences keep the existing parser.

The original visitor, builtin filtering, arrow-binding scope and identifiers
precedence stay unchanged. Static nodes and simple identifiers retain their
existing paths. Slot binding patterns keep their wrapped arrow parser, and
absent or stale ASTs keep the existing expression parser. Nothing is cached
across edits, and no pipeline stage or serialization is added.

Production counters `atelier.template_reads.retained` and
`atelier.template_reads.parses` count admitted walks and actual wrapped parse
attempts. `davinci.expr.parses` still counts original expression nodes; it is
not a repeated-parse counter.

## Verification and performance protocol

Meaningful tests compare the retained and fallback identifier sets, including
stale content, missing ASTs, pre-parsed identifiers, lexical scopes, comments,
invalid syntax, builtin names, decoded entities and Unicode. A single identical
public projection probe is built on the exact baseline and candidate and
compares complete generated code, structured mappings, semantic links and
mapper diagnostics. The full CLI report must also remain identical, including
the program file count and planted diagnostics.

Use the same Blacksmith runner for two warmups and nine alternating baseline /
candidate runs. Measure the historical 500-SFC explicit corpus and the
501-SFC shared-leaf default CLI corpus, with max width and one thread. Profiling
and complete projection comparisons run outside timings. Archive exact SHAs,
binary hashes, all samples, diagnostics, projection receipts and counter totals.
The expected shared-leaf retained count is 40,000; the baseline has no new
counters and remains explicitly uninstrumented at those sites.

The instruction ceilings stay unchanged. Actual paired Actions results, full
PR checks and protected queue acceptance are required before merge. The 10x
whole-command target remains unfinished; this change has no measured speed
claim until the paired run succeeds.
