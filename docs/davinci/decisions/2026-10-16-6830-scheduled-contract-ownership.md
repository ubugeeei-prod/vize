# Keep scheduled contracts attached to their actual owners

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The full [qualification run](https://github.com/ubugeeei-prod/vize/actions/runs/38887139884)
on `99caba071ada01a251c3618683fcfe44701ecfa0` exposed scheduled tooling tests
that still inspected workflow structure from before the CI refactor. Ordinary
PR checks had not exercised those assertions. Full scheduled qualification is
therefore part of this repair's acceptance, alongside App E2E and editor checks.

Keep substantive release platform, linker, cache, test-inventory and capture
guarantees, and inspect their current implementation owners. Release platform
values now come from the TypeScript planner; binary build behavior belongs to
the shared release actions. Preflight intentionally does not install a linker.
Remove obsolete source-text assertions when an existing executable contract
already checks the same property. Do not reproduce entire workflow command
strings in new expectations.

Historical fixture receipts authenticate the captures made at their recorded
source. They must retain their original bytes, hashes and diagnostic identities
when a later reviewed product fix changes today's application golden. A test
must not require a mutable current golden to remain equal to an older capture.
The separately reviewed current App expectations and complete App E2E retain
the current-source contract.

The historical Vue1/Vue2 capture composition had no remaining caller. Restore
it once in the full tooling worker after the actual tests-package Chromium is
installed. Its selectors capture every scheduled and dispatched full run without
diffing a zero comparison base, while retaining their merge and PR selection
behavior. Preserve the original Vue2 text, Vue1 text, then whole Vue2 SFC order,
all complete capture packets, their primary runtime joins and failure uploads.
This qualification remains outside the ordinary PR iteration.

The timing refresh reader now recognizes the semantic `Tooling (...)` workers
owned by `test-full-tooling`, while retaining historical numeric job logs.
It reads the existing tier, worker identity and per-file measurements from each
successful worker's log instead of reconstructing an index from its display
label. Current capture-only workers contribute their actual successful lead
step durations, including the restored historical capture, even without test
files. Failed workers and aggregate checks do not contribute measurements.

Product assertions and cache trust policy remain unchanged. The final source
must pass the focused contracts and a new full Actions qualification; the
earlier failing run is evidence of the gap, not acceptance of the repair.
