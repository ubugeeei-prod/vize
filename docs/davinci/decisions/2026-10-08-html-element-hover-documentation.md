# HTML element hover documentation

Tracker: [#3957](https://github.com/ubugeeei-prod/vize/issues/3957).
Paired [decision comment](https://github.com/ubugeeei-prod/vize/issues/3957#issuecomment-6059601577).

Native HTML element hovers gain a concise explanation of the selected element,
an MDN reference, and a direct link to its definition in the HTML Living
Standard. Both the static fallback and a successful Corsa hover receive these
links. Existing type signatures, examples, ranges, and component/directive
precedence stay intact. Tags classified as SVG or MathML, native attributes,
and Vue built-ins keep their existing hover behavior.

The existing native classifier uses tag spelling and gives admitted HTML tags
priority. It does not resolve the authored element namespace: for example,
`a`, `title`, and `style` inside an SVG ancestor retain their existing HTML map
and now receive its documentation. Complete overlap controls preserve that
behavior honestly. Namespace-aware native hover remains future work under
[#3957](https://github.com/ubugeeei-prod/vize/issues/3957); this slice adds no parse
or tree scan to repair that existing limitation.

## Static data and request cost

The private, borrowed descriptor table covers exactly the 110 HTML tags admitted
by the existing Maestro DOM map. It does not change L0's tag set, the DOM virtual
document, or definition routing. Common tags use a direct table index; other
selected HTML tags use a sorted binary search with at most seven comparisons.
No request fetches documentation, parses JSON, or analyzes the document again.
Components and attributes return through their existing paths before this
lookup. Distinct SVG/MathML-classified spellings bypass the HTML descriptor
table; authored namespace overlaps keep the existing HTML-first classifier.

The Corsa path appends the selected description and links to the complete
converted native response, reserving its existing output buffer once. It keeps
the converted range. The fallback retains its existing signature, editor notes,
and example and adds the two links under its documentation section. Markdown is
constructed only for the selected response; there is no eager rendered catalog.
Across all 110 descriptors, native enrichment appends 224–320 UTF-8 bytes to the
existing response. Lookup borrows one static row and allocates nothing; appending
has one capacity reservation and copies only those selected bytes. Those are
structural bounds, not measured request-latency or speedup claims. Ordinary
server observations below include transport, context creation, and hover work.

## Reference verification and authorship

The descriptions are original Vize prose. MDN documentation prose and WHATWG
standard text were not copied into the table. The reference links were checked
read-only on 2026-10-08: all 110 standard anchors and 105 distinct MDN canonical
pages exist. The six headings share MDN's `Heading_Elements` page and the
standard's combined heading anchor; `sub` and `sup` share their standard anchor.

Authoritative sources are the [MDN element index](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements)
and [HTML Living Standard element index](https://html.spec.whatwg.org/multipage/indices.html#elements-3).
Future incorporation of source prose must follow [MDN's attribution and
license policy](https://developer.mozilla.org/en-US/docs/MDN/Writing_guidelines/Attrib_copyright_license)
and the [WHATWG license](https://github.com/whatwg/html/blob/main/LICENSE).

## Verification and delivery

Complete response fixtures cover selected element summaries, both references,
unchanged signatures/examples, opening/closing tag ranges, Unicode, and CRLF.
Native payload laws keep complete type text and existing ranges for all native
content variants. Table coverage and negative routing controls accompany them.
The ordinary source-built server test records bounded selected/unrelated hover
latency observations; those are not an A/B speedup claim. Existing original400
resource budgets and protected instruction-count gates remain unchanged.
The original400 source guard admits only the reviewed seven Rust paths through
`warm-type-backed-html-hover-host.ts`. It retains the existing closed alias and
workspace-symbol allowances and changes no input, expected packet, build recipe,
numeric ceiling, or dependency authority. This is a bounded source delta for the
same performance gate, not an additional collector or campaign.

The feature is unfinished until its exact-head Actions pass and the protected
queue delivers it to main. Source validation does not claim publication or
installed-host acceptance.
