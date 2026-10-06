# Preserve Nuxt config comma style when sorting keys

Paired issue: [#7963](https://github.com/ubugeeei-prod/vize/issues/7963);
[decision comment](https://github.com/ubugeeei-prod/vize/issues/7963#issuecomment-6010317709).

The reported `0.432.0` input contains `ssr: false` before `modules` and has no
comma after the final property. Sorting is correct, but the existing fixer adds
that final comma. This repair keeps the authored final-comma choice for each
object independently; it changes key order without choosing punctuation style.

The same existing OXC object spans and property/comment ownership determine the
replacement. Each moved piece retains its exact separator offset. Interior
slots require a comma; the final slot keeps a comma only when the original final
property had one. Removing the separator at that offset preserves inline,
leading and block comments, including a comma after a block comment. The parser,
rule dispatch, ordering comparator, diagnostic range/message, spread barriers,
Nuxt 2 compatibility refusal and ambiguous-comment refusal stay unchanged.

`crates/vize_patina/tests/fixtures/issue-7963` retains the complete original issue,
command, 79-byte input and independently authored 79-byte expected output with
SHA-256 identities. Fifteen whole-source controls cover TS/JS, CRLF, compact
objects, both comma styles, comment placement, URL strings, spreads, already
sorted input and explicit Nuxt 2 compatibility. Rust also pins the complete
original diagnostic/edit and independently mixed environment-object styles.

The existing pinned Nuxt ESLint corpus and its upstream recording remain
immutable. A separate product-owned expectation table changes only inserted
final commas in those original sorting outputs. The earlier comment-ownership
correction remains pinned separately. This is an intentional compatibility
correction, not a re-recording of upstream output or a weakening of key order.

A source-built CLI law authenticates the existing build receipt, retained input
catalog and all fixture hashes, then compares complete JSON/stdout/stderr/status
and full file bytes over lint, fix, check and a second fix (60 original-command
and control runs). It saves each complete process result before assertions.
Hosted source Actions and the protected full queue must pass on the delivered
head; prepared assertions alone provide no runtime or merge acceptance.

Reporter: `ubugeeei`, verified GitHub user ID `71201308`. The meaningful source
commit and PR body retain
`Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.
The actual signed squash commit must be read back; same-primary normalization
can omit a literal footer and must be reported honestly.

This is a legacy Patina bug fix. Davinci native/history/default migration and
performance acceptance remain separate. Publication belongs to the coordinated
release owner after actual protected merge and installed-release verification.
