# Script call ownership for advisory rules

Public [#7989](https://github.com/ubugeeei-prod/vize/issues/7989), reported by
`ubugeeei` (verified GitHub ID `71201308`), contains six findings in comments
and strings from four opt-in rules. This partial script correction is paired
with [the issue decision](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6019446202).
The complete original issue body, 397-byte `MyNotes.vue`, 235-byte config,
literal command and reporter's whole six-warning output are pinned in
`tests/_fixtures/differential/linter/call-source-7989/`. The original expected
clean result remains unchanged; the issue stays open until its CSS findings
are also corrected.

Actual signed main `c804736f3e1b24109a2a105d26266d22c66d4077` searches script
text for `reactive(` and the three ID generator spellings. Switch these two
rules to `uses_ast` and the existing shared `Program` dispatch. Reactive
findings require a real call whose identifier or static member name is
`reactive`; references, comments, strings and template/regex text are outside.
Report the actual callee name span, preserving the old ordinary-call span.
Whitespace, comments between tokens, generic calls and actual template
interpolations remain syntax owned by the parser. Name matching remains
advisory and does not prove the call's Vue import identity.

The ID rule visits actual zero-argument `Math.random`, `Date.now` and
`crypto.randomUUID` calls. ID-bearing identifier bindings, assignment targets,
literal property assignments, named ID functions and noncomputed property
values own the inference; comments, string values
and neighboring bindings cannot supply that context. Retain the previous
advisory ID-name heuristic (case-insensitive `id` or lowercase `unique`) on those AST
names, restore context after each owner, and report the complete call span.
The rule does not prove global symbol identity or SSR intent. Existing tests'
raw inputs, counts and reactive snapshot are preserved; this correction
changes syntax authority without enabling a default rule.

The independently authored corpus has 41 complete configured service and CLI
vectors. It covers the whole original with LF/CRLF/Unicode framing; quoted,
template and regex text; actual calls/interpolations, member/generic calls and
multiline trivia; ID bindings/properties/assignments, context restoration and
non-ID controls; ordinary TS/JS and both SFC script blocks. Every diagnostic,
count, severity, span, label, help and fix is checked. The CLI law retains all
status/stdout/stderr bytes before assertions, executes the original plain
command first and verifies source/config bytes after each invocation. The
literal current plain formatter includes builtin help even at the original
`--help-level none`; the full expectation preserves that separate behavior.

The two CSS rules are unchanged. The complete original four-rule result
retains all three remaining CSS findings, while the same entire original
under only its two script rules is clean. A genuine later CSS correction must
update those full partial vectors to the issue's original clean expectation
while retaining input/config/report custody; partial assertions grant no
whole-issue closure. Existing real CSS positive controls stay present.

Run the existing Croquis-consumer and migration-surface census generators
before push. Fresh exact-head Actions, protected full suites and actual signed
merge govern delivery. No extra parse, provider query, stage, dependency or
instruction-budget change is introduced. Native handled cases remain zero;
no native migration, performance or separate LSP acceptance is claimed. Root
remains the sole release publisher and owns later installed verification.

The [paired positive-control correction and retained audit rejection](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6019730327)
conserves real ID functions and literal property assignments, which the old
same-line heuristic already reported. Four whole controls are appended; the
initial 37 source/result vectors remain exact. Historical `2aae5016` had actual
Check37487267761/job112351566807 security red for the new unreviewed critical
GHSA-pqg4-j6r4-53mv in Nuxt's transitive shell-quote1.9.0. Preserve all three
original attempts, 82,372 raw bytes, SHA-256
`5672e53b763f99b2a20079aa00dfc2888ebbaecfc59052c7c69b192a68aaaae4`.
PR8135 was moved to Draft/offqueue while its healthy product workers continued.
No allowlist, budget or rerun waiver is offered.

[Complete success-result custody](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6019945572)
uses one existing-style default nextest override, filtered to exactly the
#7989 CLI/API law names, so both inherited PR/full JUnit receipts retain their
successful output. The actual `2aae` four Rust/tooling workers passed, including
38 complete CLI invocations and 37 service vectors, but its passing raw output
was discarded by prior JUnit policy. Do not treat those PASS lines as a raw
protocol archive. The 41-case successor retains all 42 CLI results and the
whole separate original script-only service result through the existing
uploader. A real TOML parse and complete prior configuration inverse pass;
retry, deadline, threads, selection and all other settings remain exact.
Fresh hosted execution and extraction must authenticate actual raw retention.

The [paired security-source composition and Stack decision](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6020585011)
replays this complete source onto actual signed main
`ef84821d30fa0d8538b2472fef34418e75380523`, delivered by #8137 at
`2026-10-06T16:16:25Z`. The dependency correction genuinely passed its protected
queue; those results grant no current script-runtime credit. Preserve all
41 vectors and qualify the new exact head in Actions before admitting it.
The separately owned CSS layer must branch from that published script parent,
then update the three complete original CSS findings under its real source.
Register and verify their native Stack; a green parent prefix can enter the
queue before its child is ready. The issue remains open after the partial
parent. Root owns subsequent publication and installed verification.

The [new current audit rejection](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6033226959)
at exact `a77c035c` is Check37587584375/job112681210568: all three attempts
newly reject high GHSA-6qxp-vccf-f47h in the locked MCP SDK1.30.0 consumer.
The original shell-quote critical is absent; this is a separate dependency
finding, with patched SDK versions `>=1.31.0` in the audit. Retain all 81,162
raw bytes, SHA-256
`b1b0fcc30c6dfa201d5d9f6b55d46370da4e7dd738147488814f97980b22a4e3`.
Parent8135 is again Draft/offqueue with auto-merge unset; preserve its healthy
workers and route the dependency correction to the existing security owner.
The real CSS child may replay from the published parent, while current red
layers remain offqueue. Actual corrected signed main and fresh exact-source
qualification are required before admitting a native Stack prefix. Preserve
every script input/oracle; this audit grants no product-cause inference.

[Actual native Stack registration](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6033382557)
is Stack8147: official reads from both PRs show script8135/a77 at position1
then CSS8143/d5f at position2, size2 and real parent-base chaining. Both are
Draft with auto-merge unset and no admission. Authenticate the true parent
ancestor, all41 whole sources/five original raw pins and other38 complete
vectors; only the first three full remaining CSS findings become clean under
the child's actual production source. The parent drivers/config are exact
apart from the API carrier's corrected combined-clean comment. This grants
source/registration proof only. After genuine dependency delivery and fresh
exact greens, admit the highest ready prefix through protected Stack merge;
no individual layer auto-merge or partial whole-issue closure.
