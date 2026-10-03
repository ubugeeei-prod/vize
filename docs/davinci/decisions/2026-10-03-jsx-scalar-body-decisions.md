# Original JSX scalar body decisions

Issues: #6838 (L2), #6839 (L3), #6840 (L4), #6829 (dialects).

The existing original-Program resolver visit now records authentic identifier,
numeric, string, boolean and null payloads in its existing balanced syntax
records. Numeric values retain the original AST IEEE-754 bits, and strings and
identifiers borrow the parser's original semantic values. Authored source spans
remain separate, so hexadecimal numbers, escaped strings and escaped identifiers
retain both their actual values and their original spelling. No source parsing,
AST clone, raw AST pointer, event buffer or additional traversal is introduced.
Generic JsExpr JSX refusal and the existing resolver work/depth caps remain intact.

The private recorder still validates balanced enter/leave identity and rolls
back with the owning File transaction. Public SyntaxKind values alone cannot
mint a JsxFile or associate foreign records with another owner. The lower result
retains the genuine completed ProgramObservation and same-Program File across
moves; incomplete semantics and interrupted producers retain their typed refusal.

L3 admits a scalar only as the sole direct child of an original JSX expression
container. Identifier decisions carry the actual resolved File BindingId after
checking one real read against the same owning scope lookup. Literal decisions
require no reference rows. The existing readonly decision retains the original
node and owning File together; there is no new table or caller-provided authority.

This permits function-body JSX and TSX such as
`function render(message) { return <div>{message}{0x10}{false}{null}</div>; }`.
It deliberately does not admit ordinary module initializers, compound/unary
expressions or dynamic attributes. If any unsupported record is present, the
whole projection still refuses and retains the complete original owner. Booleans
and null are original values, not a claim about target child normalization.

Regression laws compare hexadecimal/escaped semantic values with their authored
source and exact reference rows, preserve Unicode/escaped identifiers and original
Program storage after moves, exercise genuine JSX and TSX function bindings, and
verify complete lower custody on compound/module/attribute refusal. Existing
owning/static laws remain registered. Fresh exact-head Actions and protected
merge-queue full tests, allocation and instruction gates remain required.

TODO: actual expression payloads beyond these scalars, complete whole-module
ownership/transform authority, JSX whitespace/entity normalization, native L4
output and source maps, target/runtime child rules, all directives/slots/spreads,
and product routing. This child adds no legacy adapter and claims no full JSX,
module emission or roadmap completion.
