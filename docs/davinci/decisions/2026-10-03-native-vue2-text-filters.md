# Native Vue 2 text and filter syntax

Paired issues: [#6842](https://github.com/ubugeeei-prod/vize/issues/6842),
[#6841](https://github.com/ubugeeei-prod/vize/issues/6841) and
[#6892](https://github.com/ubugeeei-prod/vize/issues/6892).

## Provider and custody

`vize_l1::dialect::vue2::surface` now owns an intrinsic Vue 2 `ComponentParse`.
Only its original native construction creates the carrier. It retains the
authentic L0 `SourceBlock`, source tree, optional authored recovery projection,
markup diagnostics and read-only text bindings. No conversion grants Vue 3
admission. Whole-source convenience construction first validates a `SourceRoot`;
block construction requires the original pointer-checked root slice. Native
markup errors are retained with complete-file offsets. Tree tokens remain
borrowed block slices, located through that same source frame.

A Vue 2 sink wraps the existing shared `VueSink`, forwarding the same native
Lexer callbacks and scope recovery. Its interpolation callback derives text
facts during that existing visit. It does not walk or parse a second surface
tree, change shared nodes/events/layouts, add a pipeline stage or serialize
syntax. Only literal `v-pre` suppresses the callback; modifier/argument forms
are retained ordinary attributes. Modern and Vue 1 source providers stay intact.

The typed `TextBinding` retains complete construct coordinates and an
`EmbedSource`. HTML text entities decode once and ECMAScript trim selects the
historical expression window with exact authored mappings. `FilterChain` owns
the actual native JS base observation and ordered `FilterInvocation` records.
Each filter retains its exact registry name, invocation span and original
once-parsed argument observations. The existing L1 language provider parses
each selected base/argument once, with existing resource admission, native AST,
comments, typed holes and corrected diagnostics. Generated `_f`/`_s` strings
are never language-parser input, and string matching never confers AST custody.
All language observations use ordinary owned vectors so parser diagnostics are
dropped. Three precise analysis-category rows record this new owned storage;
arena vectors retain only ordinary surface events/diagnostics.

## Pinned historical contract

The grammar is pinned to the official Vue 2.7.16
[text parser](https://github.com/vuejs/vue/blob/v2.7.16/src/compiler/parser/text-parser.ts)
and [filter parser](https://github.com/vuejs/vue/blob/v2.7.16/src/compiler/parser/filter-parser.ts).
Top-level single pipes select filters; `||`, quoted strings, opaque template
strings, regex literals, division and nested curly/square/round delimiters keep
their historical upstream behavior. Escaping uses the immediate previous
character and division look-behind skips only ASCII SPACE, exactly as upstream;
this is a dialect scanner, not a replacement JavaScript lexer. Names preserve
the original upstream registry key, including whitespace before `(`. No-argument
filters and `filter()` have no additional argument. Commas select only top-level
original arguments, each validated by the actual native expression provider.

The development oracle executes the complete official npm
`vue-template-compiler@2.7.16` bundle and its exact dependency closure. Published
tarball SRI, complete bytes, package/version identities and selected-file hashes
are verified before execution. Whole text-parser expression/token goldens and
actual compiler filter values establish the bounded historical reference. The
complete official `vue@2.7.16` npm main wrapper and both development/production
CJS bodies also execute unchanged. The real Vue constructor renders compiler
functions into VNodes: 52 complete text values and ordered filter-call goldens
cover chains, exact registry keys, camelized lookup, nested expressions,
Unicode/entities, literal pre and explicitly deferred families. Whole published
inventories bind all 228 runtime files and every selected original byte. This
runtime observation is a development reference; native runtime lowering remains
unfinished. The official `de-indent@1.0.2` package declares MIT but ships no
license file; its authentic package metadata records that absence. No
production package edge, installed package rewrite or runtime oracle route is
introduced. Official source is retained as compressed reference bytes with its
actual shipped license/provenance, not a copied approximation of the scanner.

## Refusals and unfinished work

Every selected binding retains source even if its base/argument parse has a
hole. A later filter refusal retains all prior started observations in an
explicit partial chain; argument lists validate structurally before any of their
language parses begin. Actual diagnostics stay available and subsequent bindings continue.
`admitted()` borrows only the original chain with no dialect boundary or native
language hole. CRLF/LF are historical text framing; lone CR and U+2028/U+2029,
empty callback content and missing closers retain typed refusals. Regex character
classes, comments, spread/trailing-comma argument lists, unsafe generated-name
spellings and unbalanced historical scanner state are explicit unsupported
families. Entity-produced braces outside literal interpolation retain authored
`EncodedDelimiter` boundaries: Vue 2 decodes text before delimiter recognition,
while this first provider only claims literal authored delimiter framing.

TODO under #6842/#6841/#6892: certify those remaining scanner/list/framing
families, custom delimiters and browser document behavior; implement Vue 2
directive embeds, descriptor selection, native legalization/runtime filter
resolution, complete file/product routes and real Vue 2 ecosystem acceptance.
Vue 0.x, remaining Vue 1 grammar, quirks and petite also remain unfinished.
No default product route, whole Vue 2 dialect, runtime lowering, control-flow
semantics or all-dialect completion is claimed.

## Validation boundary

Native laws check complete base/filter/argument facts, actual retained AST
identity, real syntax-hole diagnostics, nonzero Unicode block custody, decoded
entity maps, exact authored spans, every UTF-8 prefix, literal pre and shared
recovered/authored projections. Source-provider peers, whole L1 unit laws,
strict production and dialect linting, distinct-carrier compile-fail examples,
storage inventory and dialect/core dependency guards remain required. Cached
coherent local dependency metadata is finite source evidence: 337 actual L1
unit laws, 10 new dialect laws and 45 existing provider/dialect laws passed,
alongside strict lint and four complete upstream compiler/runtime tests. It is
not the whole
current Cargo graph. Exact-head hosted Check, unchanged all-100 instruction
ceilings, full protected queue and actual main merge determine acceptance.

## Current-main integration

The 2026-10-03 replay integrates actual main `dd05d54bd7` while retaining the
reviewed Vue 2 parser, laws and complete upstream reference bytes unchanged.
The only conflict was the generated L1 source-consumption matrix, regenerated
from the complete combined source. All incoming original/selected owner and
interpolation providers remain intact. GitHub reports no native Stack for this
PR; its required providers are already merged, so this bounded syntax slice
is an independent queue candidate. Earlier source-head checks grant no current
head or protected merge acceptance. Fresh hosted Check, full Check, all-100
measurement and actual merge are tracked separately.

## Decoded text framing correction

Independent review found that the initial separator guard inspected authored
bytes while the expression provider consumed decoded text. Entity-produced
U+2028, U+2029 or a lone CR at an expression edge could disappear during trim
and incorrectly grant native syntax admission. The complete pinned compiler
and both actual runtimes instead keep these constructs as literal text, while
subsequent valid interpolations still evaluate.

The Vue 2 observer now prepares the complete original callback content span
with the existing native Text entity decoder once. Its crate-private source
helper preserves the authored root and complete checked map; the historical
separator guard runs on that decoded window before ECMAScript trim. Complete
encoded and mixed authored/entity CRLF remain supported, including a CR entity
followed by authored trailing LF. Checking an already trimmed authored window
would incorrectly orphan that CR. The public modern interpolation preparation
and its selected-owner consumers retain their existing behavior.

Three native laws retain exact nonzero Unicode block/source coordinates,
unparsed refusal source, real AST pointer identity, single entity decoding,
fidelity and subsequent bindings. Complete pinned compiler goldens and both
real runtimes characterize nine separator positions plus six encoded/mixed
CRLF controls. The six upstream/compiler/runtime tests pass locally; corrected
head hosted laws, strict lint, full Check, all-100 measurement and protected
queue/actual merge remain external acceptance gates. The initial source and
its qualified checks remain historical evidence, not corrected-head credit.
