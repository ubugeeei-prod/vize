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
The same newline-ownership branch also carries an authored CRLF together,
rather than leaving its CRLF at the next piece and moving it to the first slot.

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
and control runs). It saves each complete process result before reading rewritten output and
before assertions, then retains full output bytes/length/hash.
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

The first source Action passed production Clippy but could not compile the new
whole-diagnostic test: `LintDiagnostic` has Debug, not Serialize. Compare the
same complete original/expected Debug values without changing public types or
fields. That failed build is not test execution. Source review also established
the old LF-only piece boundary misplaced CRLF; carry the exact CRLF in that
same branch while preserving the existing LF path and original expectations.

The complete Debug comparison uses the existing `vize_l0::cstr` macro, as
required by workspace Clippy; it does not introduce a lint exception or alter
an expected field. Its added test-only L0 reference is in the generated consumer
inventory. Fresh successor Actions remain the execution authority.

The first compact-macro import attempt was rejected by hosted compilation:
actual L0 exports `cstr`, not `format`. Bind the verified existing `cstr` export
and keep the same complete Debug comparison. That failed test build remains
historical; no runtime result transfers to the corrected source.

First actual CLI execution at `eec24` confirms the original report, CRLF and
preceding controls, then rejects an incorrectly normalized new inline-block
reference. Independent review of immutable `de40` ownership established that
both leading and final horizontal spaces travel with their respective pieces:
the three inline-block controls therefore retain two spaces between moved
pieces and none before the closing brace. Correct only those new expected
trivia strings and their catalog hashes. All authored inputs, the original
79-byte expectation, old Nuxt recording and every strict whole comparison stay
unchanged; no production whitespace normalization is introduced. The failed
run remains historical and does not establish all 60 calls passed.

The same genuine source run also reached the existing NAPI overlap-convergence
law: its original outer and `$test` objects both omit a final comma. Change
only that law's complete expected output to preserve both omissions, retaining
the original input, convergence path, diagnostics and on-disk equality checks.
No NAPI production code changes. The failed old expectation remains recorded;
current complete consumer and source Actions are still required.

The next exact source run at `e3674` passes all 60 complete CLI calls and
16,329 of 16,330 Rust cases. Its only failure is the existing Nuxt preset law's
complete expected output, which also added a comma to an originally omitted
final slot. Remove only that generated terminal comma from its expected string;
preserve the original source, complete diagnostic message, range and fix
assertion. All production code remains unchanged. That failed run stays
historical; the corrected head still requires fresh source and protected checks.
