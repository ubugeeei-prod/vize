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
sources and config, then derives the native file, URI, UTF8 opaque byte offset
and entire UTF16 inner-literal replacement range from the unique complete
current literal. The pinned native converter explicitly translates negotiated
LSP coordinates to a UTF8 byte offset before creating completion data.
Only these independently witnessed ephemeral coordinates plus explicit
authored URI/revision substitute into the complete fixed oracle. No response
field supplies its own expected value. Complete requests, responses, raw wires,
publications, failures, source/runtime hashes and shutdown evidence are retained
by existing artifact capture before assertions. Elapsed request wall time is
observational and includes the existing client/capture work; it grants no
classification-only cost, speed or budget acceptance.

The first source was sealed privately, then genuinely replayed onto signed
`6b5e6fd8357bc892d2ea9cefe271a13fc08100cb` after #8116 delivered.
The maintainer authorized one conventional non-Draft PR, #8119, for ordinary
Actions qualification, with admission pending gates. Initial source compilation,
Clippy, formatting and static JS checks passed; they remain distinct from full
native acceptance. TODO: qualify compilation, all whole
original/dirty/null/lifecycle results, selected-completion work and unchanged
protected suites/104 instruction ceilings on the genuinely composed current
source. Actual signed merge, issue closure and installed release inclusion are
still required. No local native build/install/run, new workflow, manual campaign, budget
increase or private project input is authorized by this record.

The [first failure decision](https://github.com/ubugeeei-prod/vize/issues/7997#issuecomment-6015525186)
retains real failures rather than accepting partial output.
Both original LF completion arrays, four complete resolves and the first stale
revision guard passed. The first dirty CRLF/Unicode array differed only in opaque
`data.position`: its independently owned generated prefix is 14,580 UTF16 code
units and 14,584 UTF8 bytes. The native converter's exact-revision source, not
the observed item, determines the corrected witness calculation. Every fixed
item field and every complete original/dirty source remains unchanged.

Two new Rust controls exposed real classifier gaps. The existing coarse
projection caret method ignores an attribute value's exact sub-spans; the
completion-local lookup now prefers the producer's narrowest sub-span and uses
the existing diagnostic range mapper to prove the entire authored token. Fatal
OXC parsing clears token output for an unfinished string, but the same lexer
retains its exact `Unterminated string` range. Only an EOF-ended quote diagnostic
containing the cursor can recover that route, without another parse or source
scan. Existing lexer/domain expectations stay fixed; additional unfinished
comment/regex and directive controls remain independently authored.

The initial generator check also requires exactly two new raw-OXC inventory
rows. Deleting those two rows recovers the complete prior TSV. The classifier
observation test passed initially, but its JSON was outside the existing shard
artifact directory, so no duration evidence is accepted from that run. Its
output now lives under the actual `NEXTEST_PROFILE` shard envelope; the existing
upload and all workflow commands remain unchanged. The necessary successor
needs fresh complete source/runtime gates and retained classifier observations
before admission. No initial failure or partial proof qualifies its execution.
