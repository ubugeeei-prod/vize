# SFC document symbols (2026-10-05)

Decision paired with [#8006](https://github.com/ubugeeei-prod/vize/issues/8006).
This fixes the existing editor provider, independently of the native LSP
migration and [#6883](https://github.com/ubugeeei-prod/vize/issues/6883).

## User-visible contract

Retain the existing block names, discovery order, kinds, language details,
whole block ranges and tag-name selection ranges. Add children under script,
script setup and HTML/art template blocks. Existing style blocks stay intact.

Script declarations follow the original OXC AST order. Variables and patterns
select the declared identifier; functions, classes, interfaces and type aliases
select their names. Object-valued bindings carry statically named object
members, including methods and nested objects. Classes carry statically named
methods and fields. Dynamic computed names and spreads are not invented;
imports and anonymous default exports are outside this bounded outline.
Function parameters and local statements are not promoted to top-level symbols.

Template children follow the original untransformed element AST. Elements
carry their descendants, full source ranges and tag-name selection ranges;
components use Class and ordinary elements use Object. Directives stay on their
original elements; interpolation expressions, text and comments are not symbols.
Only an exact original `<tag` name span is selectable. Parser-inserted implicit
tbody/tr nodes are omitted while their authored descendants remain visible.
All positions use one original-document UTF-16 line index, including CRLF and
astral characters. Rejected child parses keep the existing block without
inventing a partial tree. Unknown template languages keep the block.

The resident SFC descriptor already supplies original block content and offsets.
The request uses existing OXC/template parser entry points once per requested
block, without a new pipeline stage, generated module, native IPC or per-node
type query. This is structural editor data, not a type-accuracy or latency claim.

## Original input and independently authored oracle

Retain the issue's complete `MySwitch.vue` (205 bytes, SHA256
`6d7fa96a27d5338e8c9f9c9f3368c2d5be78576756b70bd53e414f96d662fa66`)
and `Parent.vue` (429 bytes, SHA256
`116cac3641306e4bb0c53fc7ed5fd662b0a3a203868a6c648be24579f0f3c40c`).
The complete public issue body and original `vize.config.json` are retained;
`tsconfig.json` is authored from the issue's complete stated options/include.

The full expected symbol arrays are newly authored from those source positions
and the feature contract. They are not original raw server captures. Parent
expects template children MySwitch/p and script children store/get/set/on/toggle;
MySwitch expects span and the destructured items/checked bindings. The arrays
preserve every name, kind, range, selectionRange, child order and omitted field.
Rust whole-array controls and shared corpus real stdio sessions use the same
immutable expected bytes. Synthetic framed data only exercises comparator laws.

Legacy element locations cover opening tags. A postorder walk also retains the
maximum original child end and accepts only an immediately adjacent authored
closing tag of the same name, using the parser's ASCII case-insensitive match
and allowing ASCII whitespace before it. This
extends the range through the closing `>` without another parse or source scan.
Self-closing and void nodes retain their opening-tag extent. Opaque original
text, comments and interpolations bound the walk; their fake tags are not names.

The new corpus requests hierarchical document symbols explicitly while leaving
all old case capability objects untouched. Both original Vue files and configs
are materialized; each corpus session opens its entry only, with the existing
editor-on/lint-off/typecheck-off observation profile. It does not replay the
reporter's complete typechecking/workspace-symbol session. Fresh source-built
RPC framing, executable/build identity, readiness, responses and shutdown must
be retained before runtime acceptance; no local build or RPC capture is credited.

## Validation and remaining work

Prepared Rust controls cover complete reported projects, destructuring/defaults,
exports, nested members/classes, TS/JSX, directives, UTF-16/CRLF, malformed and
empty content, art, and honest Pug fallback. Eleven local pure controls pass;
the original five highlight sessions are selected by their fixed IDs so new
outline cases cannot extend their original nine-request vector. The corrected
source Actions pass as recorded below; the integrated source and protected
whole RPC corpus still require fresh actual terminal proof.

Pug has no original-span element tree in the current resident descriptor. Keep
its block-only template outline rather than substitute generated HTML offsets;
its script outline works. Original-span Pug elements, style selectors, richer
anonymous/default export structure and unresolved computed members remain TODO.
No native/default or whole-fix-history acceptance, speed, memory, ranking or
10x claim follows from this change. Keep #6883 open and instruction budgets
unchanged. The verified original reporter is included as a source Co-author;
final same-primary squash normalization is reported from the actual commit.

The source-qualified consumption and migration inventories record these legacy
SFC/parser and raw OXC imports. Their authoritative generators refresh only the
Maestro shards; the existing rows and fixture gates remain intact. These rows
record current source use and grant no native migration acceptance.

The first source Check `37300853974` at `12079e0392` failed: its generated Maestro
inventories were stale, and the whole Parent oracle caught `p.range.end` at 26
instead of the required 38. Preserve that failure. Refresh only those inventory
shards and qualify the closing-tag witness on a new source; keep both original
full expected arrays unchanged. New containment, opaque-content, same-name
self-closing, mixed-case and false-closing controls require fresh source/protected execution.

The corrected source `c7f811d7` completed Check 37304360456 with all 12
symbol laws passing in the authenticated four PR Rust packets. The original
whole Parent/MySwitch arrays remain unchanged. These are source tests; the
whole LSP RPC corpus is explicitly retained for the protected merge tier.
The independent shared-registry order is the actual hover fix #8027, then
links #8029, then this outline fix. Each integration retains every accepted
case and refreshes its source qualification; queue entry grants no acceptance.

The actual signed hover commit `4e818a4e` and document-link commit `d8cd6a20`
are genuine integration parents. Preserve all 14 accepted session objects and
their complete 22 response contracts, then append the two unchanged outline
sessions: 16 sessions and 24 responses. All five outline Rust files and both
whole expected arrays remain exact corrected-source bytes. Restore the exact
reviewed hierarchy-capability helper when uniting the shared manifest; old
capability objects remain untouched. Fresh source Actions qualify this union.
The full RPC corpus runs only in the protected tier. The prior finite v0.433
publication hold was explicitly thawed; current source qualification remains required.

## Actual-main refresh after publication

The earlier genuine union `bc77650a917f049d4ba5ac7da053b3bdd131a3c2`
passed source Check 37316200242 with twenty-five successful jobs and sixteen
intentional policy skips. Its authenticated four source Rust packets contained
16,168 passing cases and all twelve outline laws. The sixteen-session,
twenty-four-response CLI corpus is merge-only, so those source results never
certified its real RPC execution. Preserve both earlier source failures and
source-qualified successes as historical receipts.

This successor genuinely merges signed actual main
`58e6a0272b4044e3aaa8b7a9cb3ac62100ccec7c` after accepted #8047 component
attribute authority, #8035 warm-query inputs, #8043 diagnostic-identity actions
and #8045 delivery ledger. It retains all five corrected outline Rust blobs,
all nine original artifacts and the complete sixteen-session/twenty-four-reply
contract. Incoming fourteen cases, capability objects, methods and complete
response vectors remain unchanged. Only the canonical append conflicts; keep
every incoming substantive clause, then append this exact source decision.
Consumer inventory rows are genuinely united and checked without waivers.

The prior `bc776`/`c7f811` greens do not transfer. Fresh exact-source Actions,
all twelve Rust laws and current protected real CLI sixteen-session/twenty-four
whole replies, complete raw framing/source/build/PID/exit custody, full Rust
and unchanged 104 instruction ceilings are required before actual merge.
Actual signed delivery and supported publication remain pending. Native,
default, Pug element and full #6883 history completion remain unclaimed.

## Complete create-vue consumer contract

The [paired #8006 consumer decision](https://github.com/ubugeeei-prod/vize/issues/8006#issuecomment-6000978399)
repairs the existing #8064 child in native Stack #8065 above #8059. The
unchanged pinned create-vue App.vue has upstream SHA256
`bdafe70baf73a040d432b574108ce0e11823a3c1c1cc8bdfe01d118d6ff7d35a`;
the complete original authored patch has SHA256
`1f763acdf1e64ad0bef203e2c3e495c528eee932336c2c75fe4dcc008396a13b`.
Keep every parent field/range and the complete three-entry folding vector.
Require exactly h1, button, p with nested a, and visitCount/doubled const
children, with whole authored kind/range/selection fields. The original outline
owner independently derived this literal hierarchy from the actual producer
and original tokens; it equals authenticated artifact 11364862052, old94a RPC
id16. The two original #8006 whole outline corpus arrays stay unchanged.

A move-only commit extracts the existing whole symbol/folding literals into a
bounded helper before the child additions. Its exact inverse recovers the old
complete test. Preserve the original input, requests, options, all other editor
assertions, timeout/retry policy, production and ceilings. This correction
uses the producer contract, not snapshot recapture or child filtering.

Old94a full 37356286971/job111919421349 remains failed, with raw SHA256
`b97bb913541e6fa51ca369dd11f2f9b169561de2ccc3ba09b754bd8374006149`.
Current b82b full 37357485998/job111923482024 also failed the same old symbol
expectation, with raw SHA256
`8c77c53368ba260e08f772a30e7c6e1c08539c1f6684f3ed3a118845cdaaa90c`.
Folding was not requested after either failed symbol assertion, so its retained
vector is source-derived without runtime credit. B82b ordinary Check
37357491035 succeeded; that separate green cannot qualify the failed full run.

Keep both Stack layers Draft/off queue. The literal-e5 child needs independent
source review, fresh ordinary and full whole editor Actions, then protected
full Rust/all104, signed actual merges and supported publication. No old
execution, native-stage/history completion or release credit transfers.

The [paired unused-import correction](https://github.com/ubugeeei-prod/vize/issues/8006#issuecomment-6001256829)
retains d72 Check 37360927155/check-js111935033092, raw SHA256
`daccd335a737a72c0e2f819f6313f6a1ddbfb82e678e81493e7218fcc8d84d6d`.
All 7,837 files formatted successfully; the zero-warning gate rejected only the
old runner's now-unused authoredPosition alias. Remove that import, preserving
the helper, complete hierarchy/folding/parents, all input/setup/other assertions
and every budget. Restoring the import then reversing the recorded extraction
still recovers the whole original runner. The independent d72 source receipt
`a2d74342b0a0416d1effeadd900ea527ab0d2c43376cfa12cc8257ba7b067052`
and separate full 37361011907 remain historical scope. Fresh exact successor
ordinary/full Stack gates and protected actual delivery are required.

The [paired support-location correction](https://github.com/ubugeeei-prod/vize/issues/8006#issuecomment-6001603927)
retains 10f ordinary PRtooling1/4 job111939628478, raw SHA256
`cc4e4866b416b739288fd61b3b9c4790fab7a2c3d32afa0690e38405ead04ebf`.
The existing snapshot-baselines law requires every check-directory TypeScript
file to be a declared runner. Move the expectation-only helper byte-for-byte
into the existing tests/_helpers directory before adjusting its two relative
imports. Keep every declaration and the strict inventory law unchanged; add
no fixture phase or checker filter. Full vectors/input/setup/other laws and
e5 parent remain exact. The successful 10f Vue/editor observations retain only
that source's scope; fresh successor ordinary/full/protected execution is
required before actual Stack delivery.
