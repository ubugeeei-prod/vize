# Deep template lint traversal safety

Issue: [#7615](https://github.com/ubugeeei-prod/vize/issues/7615).

The unchanged registered SFC ARIA rule's complete 4096-depth control aborts on
an ordinary test stack in Check 37133085047. The parser retains a bounded tree,
but its suppression pre-scan and subsequent lint visitor recurse without stack
headroom checks. The input must remain intact; a larger test-only stack would
hide the ordinary CLI/LSP failure.

The existing L0 recursion guard now covers each suppression-child-list descent
and each main child visit, including original Element/If/For traversal. Rule
callbacks, registration order, suppression state, source spans and diagnostics
remain unchanged. Shallow calls retain the guard's existing in-place behavior;
only exhausted stack headroom allocates a temporary heap segment. No parser,
semantic predicate, native receipt, pipeline stage or serialization is added.

Original branch/scope methods move by script into an ordinary private child in
a separate move-only commit, retaining their bodies and private effective API.
All touched visitor modules remain under 350 lines. The fix-history corpus
retains three physical sources: ordinary shallow control, own-pre at depth4095
and the exact depth4096 over-limit source. Ordinary-stack public SFC queries
compare every parser/product message, full attribute/header range, localized
Full Help, labels, fixes, filename and counts in En/Ja/Zh, twice without
filtering. The last case expects the original depth Error and ARIA Error.

This is a separate legacy safety correction. It grants no native migration or
whole-linter completion credit, leaves #6881 open, and does not alter the pending
native Stack's production source. No local build/install, enlarged test stack,
reduced fixture or increased instruction ceiling is allowed. Exact-head hosted
Actions, protected full suites and all 100 unchanged ceilings must pass before
actual merge; the native Error child must replay onto that genuine prerequisite.
