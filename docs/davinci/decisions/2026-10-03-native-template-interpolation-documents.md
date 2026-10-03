# Original native template interpolation documents

Paired issue: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847).
Decision: [original interpolation consumption](https://github.com/ubugeeei-prod/vize/issues/6847#issuecomment-5967932134).
Prerequisites: the genuine L1 [original interpolation operand](./2026-10-03-l1-original-interpolation-operands.md)
merged in [#7491](https://github.com/ubugeeei-prod/vize/pull/7491), and the
actual [retained-expression Doc provider](./2026-10-03-native-expression-documents.md)
in [#7534](https://github.com/ubugeeei-prod/vize/pull/7534).

`native_template_document` borrows a genuine `NativeTemplateComponent` and
ordered original `NativeInterpolationOperand` references. The existing L1
provider privately derives each original body event's full/content spans,
source preparation and intrinsic JS/TS Module profile from the authentic
Descriptor-selected owner, then decodes and parses once. This consumer joins
each operand with `admitted_for` at that exact `NativeChild` during its actual
depth-first document traversal. It does not add another source helper,
preliminary walk, parser, decoder, AST clone or normalization stage.

The resulting `NativeTemplateDocument` retains the selected owner and all
original operand borrows beside its arena Doc. It covers only the selected
template block; assembly of the enclosing SFC and its scripts/styles is
separate unfinished work. Bare `template_document` keeps its existing typed
interpolation refusal. Opening-tag, attribute and closing layout helpers move
without behavior changes in a separate refactor commit before this feature.

Original framing passes the shared source cursor. Each complete expression
document receives the exact selected `SourceBlock`, including its full authored
root and nonzero block start. Canonical interpolation edge spaces can break
according to the separate printer and actual structural depth. Only existing
provider-selected authored HTML edge whitespace is replaced: expression
comments, entity-produced whitespace, original literal/operator/entity
spellings and internal authored newlines remain source slices. Canonical
closing spaces/newlines disappear during ordinary pre-parse authored-edge
selection and do not extend the retained comment text. The enclosing
Descriptor independently scans authored `//` through its next literal LF.
For every interpolation, an original trimmed authored tail containing LF
(including CRLF) is therefore emitted verbatim; other closing edges use the
canonical layout. This rule depends only on genuine original wire bytes.
Decoded AST or comment kinds cannot classify the outer authored scan: encoded
block-comment delimiters containing literal `//` and encoded quotes around a
`//` string also need that original physical LF. An entity-decoded LF or
private parser-wrapper LF never supplies a replacement terminator. Encoded
line comments without literal `//` remain supported through their authentic
Descriptor admission; no new syntax gate or comment-classifier pass is added.

Missing, extra, reordered, duplicate or foreign operands reject the whole
document. Original syntax holes reject while retaining all diagnostics and
comments. Unsupported expression descendants return the original Expr Doc
refusal. Generic authored syntax cannot mint an interpolation operand: the
raw admitted string `"a&#10;b"` is a concrete counterexample, because the actual
Vue text decode produces a newline and a syntax hole. No raw admitted source
or AST can replace that genuine provider observation. Native `v-pre` regions
remain original Text and require no operand.

## Evidence and remaining work

Fifteen new integration laws use the actual Descriptor, selected component,
native body projections, once-retained decoder/parser and Doc printer. They
cover complete selected-block output and ordering, JS/TS profile, nesting and
width 0/1/7, CRLF and authored comment newlines, complete entities, native
verbatim suppression, fixed points, missing/extra/reversed/duplicate/foreign
operands, syntax holes and unsupported descendants, generic authored entity
context refusal, unchanged bare/refusal behavior, real JS/TS output reparses,
and stable original AST/comment/map/content addresses. The independent
reparse compares typed nodes, values, original atom spelling, parentheses,
operators and complete decoded/authored comment text across LF/CRLF and
flat/narrow widths. A compile-fail law prevents raw retained syntax admission.

## Hosted framing correction

The first published child head `91ee9e2236f9c8bb194620bcb61e47acfbd0f0da`
in #7541 built successfully in Check `37114543673`, including the actual
raw-syntax compile-fail doctest. Eleven integration laws passed; the fixed-point
and semantic-reparse groups failed at Descriptor admission because their
trailing `//` fixtures omitted the actual authored LF. They have no complete
acceptance credit. Inspection of the existing `container/vue/scan.rs`
established the additional outer framing requirement: pre-parse whitespace
selection alone cannot make a flat closing edge safe after a line comment.

The focused correction keeps original LF/CRLF tail bytes from every genuine
operand, replaces those invalid inputs with actually admitted authored-LF
fixtures, and adds whole output/full-Descriptor reparse/fixed-point controls
for LF, CRLF and encoded LF followed by a physical LF. Independent review adds
the encoded block-comment/literal-`//` and encoded quote/`//` string cases,
which demonstrate why decoded last-comment classification is insufficient.
Separate controls retain raw-CR-only/absent-LF Descriptor refusals and genuine
encoded-line-comment admission. It changes no lower provider, parser, source
map, capacity, oracle or parent queue head. The corrected head still needs
fresh hosted validation and protected actual merge.

Local source formatting, diff hygiene and generated Glyph inventories precede
hosted acceptance. The new Rust laws have no acceptance credit until the
exact-head affected Actions runs them; protected full/instruction queue gates
and actual merge remain required. The child is branched from the real Expr Doc
parent and must be registered as a GitHub native Stack before queue entry.

TODO: other expression families, directive-value formatting, every Vue
dialect, other embed shapes, SFC assembly, formatter options and checked edit
integration remain unfinished. Product corpus/fix-history completion and
[#6882](https://github.com/ubugeeei-prod/vize/issues/6882) still gate any default
replacement. Existing legacy output, budgets, oracle policy and shipped
formatter routes remain unchanged; #6847 remains open.
