# Template continuation prefix width

Paired issue: [#7876](https://github.com/ubugeeei-prod/vize/issues/7876).

The original Example places text, an interpolation and an inline conditional
`template` on one physical line. The existing opening-tag decision measures
only indentation, tag name and attributes. Its compact header fits alone,
but the emitted text prefix makes the whole opening header exceed width 80.
The previous [attribute-prefix slice](./2026-10-08-directive-attribute-print-width.md)
actually merged as signed `f3ed2ee49c`; it deliberately left this case open.

Use the existing attribute printer and compare a continued header against its
actual emitted prefix and compact closing suffix (`>` or ` />`). Display columns include Unicode width, configured tabs
and the private SFC base depth exactly once. Keep the existing standalone tag
decision, sorting, attribute/value emission and bracket policy. Only normal
chunks continuing an unlocked line use the extra comparison. Line-scoped
suppressions retain their complete original line. The preserve-text entry
continues through its original writer wrapper without this new comparison.

All breaks remain inside opening-tag syntax. The LineJoiner still owns every
text/interpolation boundary, source gap and closing tag. No new expression
parse, pass, pipeline stage or serialization is added. Ordinary JavaScript
expression formatting and literal, `v-for`, raw and map controls remain on
their existing paths. Native handling remains unsupported; native handled
and equivalent counts remain zero.

The complete original Example, all original configurations and both later
reports remain intact. The dated earlier decision and every one of its old
assets stay byte-exact. In particular, preserve the original sixteen-case
corpus and its old partial Example expected output as an explicit witness.
A closed test-only current reference admits only `original-example`: it checks
the complete old corpus hash, input hash, old expected hash and separately
reviewed complete current expected hash. Both existing API/CLI consumers
continue every other old case unchanged. There is no bulk golden refresh.
The original thirteen-case historical audit, current shared sixteen cases,
all 300 historical API obligations/captures and all 104 ceilings stay exact.

Eleven complete new vectors cover the original Example and isolated original
paragraph, compact-header exact40/over41, Unicode display columns, tabs4,
authored CRLF with Auto, bracketSameLine, a line-scoped suppression and a
one-line paragraph with no leading text gap and a decisive self-closing img42/40 boundary. Whole input/output hashes and
complete options are recorded. The exact40 and suppression cases may retain
long complete lines: children and closing syntax are outside this slice.

References are independently authored without reading modified production or
consulting Vize output. Cached Oxfmt 0.63.0 supplies the opening-tag and script
syntax; the reporter's source-owned text adjacency supplies the retained
closing/content line. Auto uses an explicit CRLF provider probe. The complete
Example reference has a maximum of 75 columns, including its 74-column
closing/content line. The img header probe explicitly uses singleAttributePerLine; its default Oxfmt whole layout is retained as separate evidence. It deliberately retains the existing join behavior
rather than introducing whitespace at a runtime boundary.

Ten separately sealed stock Vue contracts retain exact full DOM innerHTML,
textContent, SSR and event observations across seven mounted transitions:
false0, true1, true2, empty0, Unicode10, negative1 and back0. Public Vue,
compiler and renderer 3.5.35 use one coherent runtime; happy-dom is 20.11.2.
The unavailable reporter MyList module is an explicitly declared deterministic
prop/event observer, not an assertion about the real component implementation.
A complete short-paragraph Oxfmt probe adds a rendered leading space; it is
retained as negative evidence and excluded from expected-output authority.

Public API laws require all eleven complete outputs over three passes and exact
changed flags. Configured CLI laws require initial read-only check, three
writes and final check for all eleven vectors, plus discovered/explicit original
TS and explicit original JSON configuration. A source-receipt-bound CLI
observer runs all ten sealed runtime cases over an initial read-only check, three writes and final check,
retaining complete configurations, processes, streams, bytes and compiled
DOM/SSR observations. Twenty additional raw CLI calls retain the isolated original paragraph and three original discovered/explicit TS/JSON configurations, for seventy full CLI processes in this artifact. Independently authored expected contracts are never
regenerated from tested Vize output.

Local qualification is limited to independent reference/stock-runtime work,
isolated compilation of the actual attribute helper and finite reference
adapter, syntax/formatting, custody and source validators. It is not a full
source-built formatter execution. Fresh exact-head Actions must execute the
whole API/CLI laws, original historical/shared corpus and canonical checks;
root owns protected qualification, unchanged instruction measurements and
actual merge. No earlier source or installed-package run grants successor
credit. The parallel #7871 installed-public preparation is untouched. The fresh source branch incorporates actual signed f8b4d88b6a main without a formatter production conflict; canonical paragraph195 alone receives this owned suffix, preserving all incoming lines and the complete footer.

Internal expression reflow (the deep call remains 101 columns at width100),
child-inclusive start-tag/mustache layout, closing-tag reflow and the complete
Edit.vue multiline guard remain TODOs in #7876. This bounded PR does not close
that issue or claim release, history, native or performance completion.

Initial a5 hosted JS qualification correctly rejects five new observer/tooling
warnings: two unhandled test promises, one inferred provider tuple union and
two Function constructors. The successor uses explicit void tests, named
provider descriptors and node:vm compileFunction for genuine stock-generated
render code. Production, all whole inputs/expected contracts, current-reference
hashes and every ceiling remain unchanged. Re-run the same140 stock states
and custody laws; all source-built acceptance requires fresh successor Actions.

## Original report closure (2026-10-09)

The original [#7876 report](https://github.com/ubugeeei-prod/vize/issues/7876)
describes an 87-column attribute and a 197-column whitespace-sensitive
paragraph at `printWidth: 80`. Both reported defects are fixed on main,
including signed [#8307](https://github.com/ubugeeei-prod/vize/pull/8307)
merge `09dc856cc6a3b1d98bc1e46bb02f1684e65b7595`.
Its [protected Check](https://github.com/ubugeeei-prod/vize/actions/runs/37772908396)
and four protected real-project workflows completed successfully. The
291-SFC count describes the reporter's project; it is not an additional
acceptance gate for the two original examples.

A bounded replay retains the literal complete Example and TypeScript
configuration from the issue without rewriting either. Input SHA-256 is
`cda86e54a1e3f15b9c45652171e664c8fad2aa8e57064dfa6ee6998940e633fb`;
config SHA-256 is
`7f877b7d4e19c930cb35ba081beffae553c5245606bf2790c85627bd84a1ea0e`.
Initial `fmt --check` exits 1. Three `fmt --write` / `fmt --check` pairs
exit 0 and retain exactly the already sealed
[original Example expected output](../../../tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876/original-example.expected),
SHA-256 `e379010d0e5b534b84dad6f3d6774c599e59ed7845243799ca42ec62432190c7`.
Its longest line is 75 columns. The existing
[seven original stock contracts](../../../tests/_fixtures/differential/formatter-regressions/continuation-prefix-width-7876/original-example.stock-contract.json)
also pass for both original and formatted sources: whole DOM, text, SSR and
event observations preserve the reported whitespace behavior.

This replay uses the retained native CLI built at
`5ed1feff6b2a2b74905bb14b46f970906679a511`, binary SHA-256
`e0f42e98c2a33a35d5e284f02725cec1470fceffbbbd8d89d219c40e0f8a76d6`,
with the real installed Node 24.14.0 runtime for unchanged TS config loading.
The relevant formatter, CLI/config, parser and vendor source blobs match
qualified source `29493199d01d3579b6f81b4d38cd7ceaf33b7f76` and observed
main `7d645dbca9d3be2e0c96feae7cd826f0f49b02fa`; workspace version metadata
differs. This is source verification, not an installed release assertion.
Whole CLI receipt SHA-256 is
`17067e17fdee2df4531d3c83ea03c90c3de2fcf9406d1c1deae2e945c74ffa2e`;
whole runtime receipt SHA-256 is
`a4b04115f7928706de59b8a6e7b7311c473041acf587c203dd0e1e7a84c32071`.
The earlier missing-runtime attempt and comparison against a different
historical probe remain retained; no frozen reference was edited.

The earlier broader TODOs remain historical and independently scoped. They
do not reopen these two original defects or add a closure condition.
Publication remains pending. This addendum changes no product, fixture,
test, instruction ceiling or release gate.
