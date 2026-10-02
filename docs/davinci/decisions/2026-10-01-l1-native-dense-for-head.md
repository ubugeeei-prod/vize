# L1 native dense Vue for-head

This bounded provider implements the approved first ForHead slice for
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836). It builds on checked
source slicing, the original-expression handoff and the original-slot-binding
handoff. No product route selects it, and no native ForOp lowering is claimed.

## Dialect partition and one parse per language piece

`parse_vue_for_head_once` is an explicit Vue dialect entry point. The ordinary
single-Program `parse_once` continues to refuse composite ForHead/FilterChain.
The strict text splitter selects the first whitespace-delimited `in`/`of` on
untrimmed decoded source, matching the existing strict repository grammar.
Thus `a in b in c` keeps alias `a` and collection `b in c`; it never mistakes
the whole head for a JavaScript binary expression. Only a matched outer alias
parenthesis pair is stripped. Collection trailing whitespace stays in its
checked part, including authored line-comment bytes.

Both ranges pass through `EmbedSource::slice_in`. Existing entity atoms and
UTF-8 boundaries remain authoritative; no part is decoded again and no authored
offset is computed by adding decoded lengths. Whole-head admission applies
before any source allocation or language parse. The strict text split itself
allocates nothing. Each actual generated parser wrapper also keeps the current
31-unit limit and shared expression safety admission.

The entire alias block is parsed once as SlotParams, and the collection once
as Expr. Real OXC FormalParameter roots determine value/key/index positions.
There is no comma scanner, per-alias reparse, placeholder parameter, AST clone,
combined Program or additional pipeline stage. Commas inside object/array
patterns, defaults, strings, regexes, templates, comments and TS generic types
are represented by the actual parameter AST.

## Owning observations and authored views

`NativeForHead` owns the whole checked source and two separate optional
`Result<RetainedPart, Box<NativeSyntax>>` observations. Each part keeps its full
owned Diagnostics with ordinary Drop and its original arena comment slice. A
defensive wrong-shape handoff keeps the original AST and observations intact
in a cold box. No observation owner is stored in drop-free arena memory.

A `DenseForHeadView` exposes only one to three actual FormalParameter roots
and the actual collection Expression. Those arena roots, source copies and
separate coordinate projections can outlive the composite observation owner.
Generated arrow, parameter-list and wrapper Program/container nodes are never
advertised as authored syntax. Comments and diagnostics carry an aliases or
collection role and use that part's checked projection; no merged diagnostic
record is fabricated or discarded.

## Typed refusals and unfinished scope

Empty or sparse alias positions, more than three aliases, rest aliases, syntax
and observed contextual errors remain typed refusals. The whole source and every
available part owner stay inspectable. A trailing comma authors a sparse
position even though SlotParams accepts it; this provider examines only the
tail after the final real parameter root, skipping exact retained OXC comment
spans. It never silently collapses the third position of `(value,index,)`.

The actual [parameter-context prerequisite](./2026-10-01-l1-native-parameter-context.md)
refuses defaults such as `x=await y` without rejecting a nested async body. The
dense provider consumes that precise local hole and keeps the original source
and observations; parser success alone never establishes valid alias context.

This view remains a syntax carrier, not a strict runtime-formal/module or
file-origin capability. Actual JS/TS probes retain `arguments`, `eval`, `yield`
and duplicate aliases with no parser diagnostic while their emitted strict ESM
modules fail. A separate correction must observe runtime-formal restrictions
in the same visitor; duplicate For bindings use the existing canonical bound-name
preflight. Authentic whole-head/file association remains a separate prerequisite.
Native control admission stays refused until those real contracts are established.

Full sparse syntax, older repeat grammar, Vue compatibility quirks, FilterChain,
full JS/TS admission, JSX/TSX, file-language selection, identifier resolution and
native structural/product consumption remain unfinished. The first-separator
rule is the current strict repository rule, not a claim of complete upstream
Vue-version equivalence.

Native laws cover real collection/binding forms, first-separator and Unicode
positions, entity-atom projections, live arena roots after owner Drop, commas
inside lexical forms, sparse/extra/rest/trailing refusal, complete role-labelled
observations, strict malformed splits, whole-head and real-wrapper limits, and
intact defensive rejection. Publication requires the parameter-context law,
strict source checks and exact-head Actions to pass.
