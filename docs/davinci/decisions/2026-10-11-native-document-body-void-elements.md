# Native Document body void elements (#6835)

The independently ready Stage 1 provider expands the native L1 Document
structure with `area`, `embed`, `source`, `track` and `wbr`. Previously, a
complete explicit HTML envelope containing `<wbr>` returned
`UnsupportedElement`; these tags now insert an empty element under the actual
current body-subtree parent. Subsequent text, comments and elements remain
siblings. ASCII case folding and optional slashes follow the same original
start-tag event and existing static attribute view.

The [HTML tree-construction rules](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody)
insert and immediately pop these tags. The active-formatting reconstruction
required for `area`, `embed` and `wbr` has no additional state in this provider:
formatting elements already refuse admission. General formatting, implied
ends, tables (`col` included), foreign content, raw text and implicit envelopes
remain unfinished. Obsolete `param` and `keygen` remain outside this slice.

Construction retains the sole original Document event tape, the same element
and stack storage, source spans, adjacency and content-end coordinates. Void
elements never enter the open-element stack and lend empty child cursors.
There is no second parser/decoder, retained value buffer, additional pipeline
stage, legacy-backed carrier or product-route switch. Tag classification adds
five comparisons after the earlier names; no speedup is claimed.

Ten complete original sources and independently authored whole packets are
shared by native Rust and actual Chromium. They compare every element's
parent and effective attributes plus all direct body-subtree child nodes.
Coverage includes every new tag, ordinary/slash forms, uppercase names,
nested parent/sibling boundaries, duplicate/Boolean attributes, Unicode,
literal NUL, CRLF versus character-reference CR and once-decoded values.
Negative native controls preserve wrong insertion modes, complete-name
matching, explicit void end-tag refusal, Vue syntax and the table policy.
All earlier source corpora and laws remain unchanged.

Before/after source execution, exact-head Actions and the unchanged complete
instruction ceilings precede queue admission. Protected full checks and the
actual merge remain required. This completes only these five Document void
element families. Whole Document tree policies, petite-vue scope/effect/L2,
all Vue dialects, native products and default cutover remain unfinished;
#6835 and #6843 stay open.
