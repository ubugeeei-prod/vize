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
`crypto.randomUUID` calls. ID-bearing identifier bindings, assignment targets
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
initial 37 source/result vectors remain exact. Current `2aae5016` has actual
Check37487267761/job112351566807 security red for the new unreviewed critical
GHSA-pqg4-j6r4-53mv in Nuxt's transitive shell-quote1.9.0. Preserve all three
original attempts, 82,372 raw bytes, SHA-256
`5672e53b763f99b2a20079aa00dfc2888ebbaecfc59052c7c69b192a68aaaae4`.
PR8135 is Draft/offqueue; healthy product workers continue. The security owner
must supply a genuine correction and current exact-source green before queue
admission; no allowlist, budget or rerun waiver is offered.
