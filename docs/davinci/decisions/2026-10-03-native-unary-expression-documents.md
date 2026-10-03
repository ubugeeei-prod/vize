# Original unary expression documents

Paired issue: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847).
Decision: [retained unary consumption](https://github.com/ubugeeei-prod/vize/issues/6847#issuecomment-5968522783).

The original [Expr Doc provider](./2026-10-03-native-expression-documents.md)
and genuine [selected interpolation consumer](./2026-10-03-native-template-interpolation-documents.md)
are already protected-merged. This independent slice starts from their actual
signed `main` merge `5c6f735a7dd82e6d89fd5b1d10bd67e4a3911e87` in a new `wt`.

The existing immutable expression walk now accepts original `UnaryExpression`
nodes over supported descendants. It uses their original operator, argument
and node span. The checked source gap may contain only that exact AST operator,
ASCII whitespace and original typed comments. Operator start must equal node
start, and operand end must equal node end. Existing checked projection and
arena documents retain complete authored operator/literal/entity spelling,
parentheses, comments and original source borrows.

All seven operators (`!`, `~`, `+`, `-`, `typeof`, `void`, `delete`) use one
separating space when the original prefix gap is proven plain ASCII whitespace.
This conservative layout keeps nested `+ +a` and `- -a`, infix signs and word
operators distinct. A comment-bearing or encoded whitespace gap stays
verbatim. No precedence reconstruction or semantic rewrite removes original
parentheses. Existing inner infix groups may break at narrow widths; the
prefix separator does not introduce another break.

Updates, await nodes and unsupported descendants still refuse the complete
document. This includes a supported unary parent containing an unsupported
call/member child. Actual parser holes retain their original observations.
Strict-module `delete` of an identifier cannot receive syntax admission credit;
the positive operator control uses genuinely admitted `delete 0`.

The Doc depth limit remains 16, and L1's existing 31-unit admission is unchanged.
Unary nesting can reach that consumer bound while the actual L1 parser still
admits the input: 16 prefixes are supported, and 17 prefixes return the existing
whole-document `DepthLimit` with the original AST/source intact. This does not
relax a provider or printer budget.

Independent laws cover full standalone and selected-template output, JS/TS,
seven operators, nested/infix signs, original parentheses and precedence,
encoded operators/gaps, typed comments and CRLF, extreme widths, original
AST/comment/map/source custody, actual AST/comment reparsing and fixed points.
The selected consumer uses the same authentic Descriptor-selected original
interpolation operands and original source block; no raw syntax substitutes
for its private admission. Existing interpolation framing and refusal laws
remain in the same hosted run.

Local source formatting and the owned canonical Glyph inventories precede
exact-head affected Actions. Those source-built laws, protected full suites,
all unchanged instruction ceilings and actual merge decide acceptance. Local
Cargo caches or pre-existing executables provide no runtime acceptance credit.

TODO: other expression families, directive values, all Vue dialects, other
embeds, enclosing SFC assembly, formatter options and checked edits remain
unfinished. [#6882](https://github.com/ubugeeei-prod/vize/issues/6882) still gates
default replacement. This adds no parser/decode pass, visitor, L1 API, legacy
helper or shipped route; oracle policy and all budgets remain unchanged.
