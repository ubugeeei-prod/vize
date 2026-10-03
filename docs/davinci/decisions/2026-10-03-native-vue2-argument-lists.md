# Native Vue 2 blank and trailing filter argument lists

Paired issues: [#6842](https://github.com/ubugeeei-prod/vize/issues/6842),
[#6841](https://github.com/ubugeeei-prod/vize/issues/6841) and
[#6892](https://github.com/ubugeeei-prod/vize/issues/6892).

## Provider and original syntax

The [native text/filter provider](./2026-10-03-native-vue2-text-filters.md)
actually merged through the protected queue as `c05e0ee71b` on 2026-10-03 at
09:09:16 UTC. Exact-head PR Check, full Check and real all-100 instruction
measurement passed; the actual queue candidate also passed all complete
source suites. Fresh main retains its 43 owned source/test/reference paths
byte-identically. This delivery is bounded syntax ownership, not complete
Vue 2 or a product/default route. Main push downstream checks remain separate.

The native invocation scanner now accepts whitespace-only argument windows
and one trailing comma after nonempty original arguments. This is the actual
Vue 2.7.16 compiler grammar: its generated call inserts the base value before
the original argument spelling, so `upper( )` invokes with only the base and
`add(2,)` invokes with the base plus one additional argument.

Each comma position precedes the final list sentinel. Only the final empty
window is omitted; every earlier empty window refuses. This preserves leading,
repeated and interior-hole refusals without accepting an extra omitted value.
The unchanged historical scanner retains nested commas. ECMAScript trim
recognizes NBSP/BOM/LF/CRLF; U+0085 remains an actual original language hole.
Exact registry names remain unchanged, including whitespace before `(`.

Complete structural validation still precedes all argument language parses.
Each selected original nonempty argument is parsed once through the existing
JS provider, retaining AST, comments, maps, holes and diagnostics. No generated
call is parser input. The complete invocation span retains trailing punctuation
and entities; each argument projection excludes those bytes. Later list
refusals retain every previously started base/filter observation.

## Evidence and unfinished work

Four focused native laws cover ordered arguments, the former `wrap(1,)`
refusal as a preserved positive, exact registry names, nested comma syntax,
encoded punctuation, nonzero Unicode `SourceBlock` custody, actual AST pointer
identity and exact decoded-AST-to-authored projection. Leading/repeated comma
controls, a real malformed trailing argument, U+0085 and later structural
refusal preserve diagnostics and subsequent bindings. The old doubled-comma
case remains an explicit negative; no assertion or original input disappears.

Complete pinned compiler expression/token/AST goldens and actual development
and production VNode/filter-call values cover the new lists. Seven complete
upstream/compiler/runtime tests pass locally; compressed compiler/runtime
reference bytes, package provenance, licenses and inventories are unchanged.
Exact-head hosted native laws, strict lint, full Check, unchanged all-100
measurement and protected queue/actual merge remain acceptance gates for this
new source. Independent peer review found no blocking source defect.

Spread arguments, comments, regex-class scanner families, custom/encoded
delimiter admission, Vue 2 directives/descriptors and native runtime lowering
remain unfinished, alongside complete file/product routes and other historical
dialects. Product fix-history gates remain open. This adds no parser stage,
serialization, production legacy dependency or default route replacement.
