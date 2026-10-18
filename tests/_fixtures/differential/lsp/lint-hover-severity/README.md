# Diagnostic hover response witness

This authored handler-boundary fixture covers the Maestro portion of historical
fix `ef8c3e059ed645e319154d475ef6d994dc489ee6` (#7053), selected by #6883's
unchanged historical ledger. The commit replaced the five emoji severity badges
and documentation-link icon with literal severity labels and plain link text.

`App.vue.txt` is a new comment-only input, not a recovered historical reproduction.
`responses.authored.json` supplies typed diagnostics at that comment, an authored
query, and complete serialized Hover results. The real `MaestroServer::hover`
handler consumes the revision-bound diagnostic cache. No Patina diagnostic,
checker response or CLI process is fabricated or claimed by this fixture.

The response tests retain all five severity cases, message/help/documentation
content, omitted Hover range, and ordered lint/Musea merging. The controls check
an out-of-range query, a foreign diagnostic source, disabled lint, a stale cache,
and a fresh publication at the new document version. No actual Hover is filtered,
sorted, trimmed, or rewritten before comparison.

The original emoji-bearing helper would fail these whole-result expectations.
This is a handler response witness, not an executed historical binary, stdio
wire session, shared-adapter registration, whole-fix-history closure, native L4
parity, or publication proof. The existing Maestro Rust cohort must execute the
tests on the current source before they receive runtime credit.
