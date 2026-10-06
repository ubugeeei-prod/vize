# String literal completion ownership

Issue: [#7997](https://github.com/ubugeeei-prod/vize/issues/7997).
Paired [source preparation decision](https://github.com/ubugeeei-prod/vize/issues/7997#issuecomment-6014880361).

The original public `Page.vue` calls an imported function whose argument is
`"form.name" | "form.help"`. At zero-based template position `8:11`, the old
route offers `t` and `title`; script position `3:17` appends Vue API identifiers
to the native literal candidates. Both positions are independently derived
from the unchanged 178-byte source. The expected values are `form.help` and
`form.name` without unrelated identifiers.

The prepared correction routes an ordinary Vue literal to the existing native
completion request and returns its whole result. It neither appends structural
items nor supplies a fallback when native values are absent. The existing
current-source revision, cancellation/deadline, canonical project/options,
whole opaque native item and resolve guards remain the authority. Art/dialect,
nonliteral, member, markup, comment and regex routes retain their old dispatch.

The classifier borrows the current cached SFC block or template projection.
A conservative quote-candidate guard precedes one existing OXC token-parser
invocation; its lexer owns positive quoted tokens. Template routing additionally
requires the entire generated token to map to byte-identical authored text.
Comments, regex literals, template substitutions and cursor positions after the
closing quote cannot grant the new route. Incomplete quoted tokens can suppress
unrelated identifiers, while the native provider decides their values.

This is additional completion-local classification work, not a hover change.
The negative guard can admit ordinary cursors after earlier multiline template
or continued-string syntax; it does not claim that every parse has a positive
literal result. There is no unconditional document parse, new pipeline stage,
cache, backend query or intermediate serialization. Selected-completion cost
is unmeasured and must be qualified before publication; no speed gain is claimed.

The original full issue, commands, `lsp-req.mjs`, `messages.ts`, `Page.vue` and
tsconfig are pinned in the differential corpus. The SFC carrier is named
`Page.vue.txt`; the native fixture writes its identical bytes to `Page.vue`.
This prevents an unregistered standalone corpus SFC from changing unrelated
native fixture discovery. Authored dirty Unicode/CRLF and single-quote sources
and the unconstrained-string dependency are separate complete pinned controls.
Git explicitly retains the LF/CRLF carrier bytes.

Five Rust laws cover both original positions, lexer/source domains, Unicode
boundaries, native-unprovided synchronous behavior and classifier-only work.
The last records sixteen actual classifier calls for each original position and
an ordinary template identifier after context preparation, retaining whole
input/projection and individual wall times before assertions. This is a noisy
test-host observation without provider/hover requests, not CPU/allocation
attribution, a gain claim or a new performance ceiling. The supplemental stdio
oracle uses the existing source-built CLI receipt and whole-wire lifecycle
capture in normal tooling Actions. It prepares two real server sessions and
37 exact completion/resolve results: ten whole union arrays, twenty whole
resolved items, five stale/closed/reopened resolves and two null plain-string
controls. Six current publications must retain their complete empty diagnostic
arrays. The original ten-second labels-only client is preserved and is not the
supplemental observer's execution contract.

The complete native item oracle is authored from the pinned Microsoft
typescript-go [string completion implementation](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/string_completions.go)
and [item converter](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/ls/completions.go).
Constant kind `21`, sort text `11`, full replacement edits, data/name and
resolved detail/documentation remain explicit. The fixed Vize outer item retains
the existing absence of an authored `textEdit`; its whole native edit remains
in `data.vizeCompletion.item`. `oracle-source.json` pins whole primary files
and the unchanged local converter/resolve/mirror/revision/options bodies.

Before reading each response, an unreferenced bootstrap marker identifies one
physical fixture-owned mirror. The observer retains full generated/authored
sources and config, then derives the native file, URI, UTF16 cursor and entire
inner-literal replacement range from the unique complete current literal.
Only these independently witnessed ephemeral coordinates plus explicit
authored URI/revision substitute into the complete fixed oracle. No response
field supplies its own expected value. Complete requests, responses, raw wires,
publications, failures, source/runtime hashes and shutdown evidence are retained
by existing artifact capture before assertions. Elapsed request wall time is
observational and includes the existing client/capture work; it grants no
classification-only cost, speed or budget acceptance.

The first source was sealed privately, then genuinely replayed onto signed
`6b5e6fd8357bc892d2ea9cefe271a13fc08100cb` after #8116 delivered.
The branch remains uncompiled; the maintainer authorized one conventional
non-Draft PR for ordinary Actions qualification, with admission pending gates. Rust formatting and static JS checks are
distinct from real native acceptance. TODO: qualify compilation, all whole
original/dirty/null/lifecycle results, selected-completion work and unchanged
protected suites/104 instruction ceilings on the genuinely composed current
source. Actual signed merge, issue closure and installed release inclusion are
still required. No local native build/install/run, new workflow, manual campaign, budget
increase or private project input is authorized by this record.
