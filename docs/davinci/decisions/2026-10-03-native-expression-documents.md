# Native retained-expression documents

Date: 2026-10-03

Owner: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847)

## Decision

Add an independent opt-in `expression_document` provider over the genuine
immutable L1 `RetainedExpression`, rooted on fresh authenticated `origin/main`
`c05e0ee71b6010a1b6bf1a8488737ab2736efdbc`. The API accepts the original
retained owner, a checked caller SourceBlock and the shared allocator. Its
ExpressionDocument retains that same owner borrow and exposes its Doc; consuming
`into_parts` transfers both the same borrow and owned Doc to a dependent
consumer without cloning either syntax or the document.

The parser-owned admitted expression and original decoded content must agree
by pointer and length. The retained authored root must agree with the caller's
checked root by pointer and length, and the selected embed must lie entirely
inside its block. Every AST/comment span passes through the original wrapper
correction and exact decode-map projection; partial entities, UTF-8 splits,
foreign roots and uncovered blocks refuse. Physical byte custody does not
establish document/version identity or complete native product admission.

The first bounded families are identifiers, numeric/string/boolean/null atoms,
original ParenthesizedExpression, BinaryExpression and LogicalExpression.
One immutable AST walk builds the existing arena Doc vocabulary. It preserves
literal spelling, identifier escapes, operator spelling, original parentheses
and complete entity spelling. Operators and delimiters are located only inside
the gaps already selected by genuine typed child spans, against the AST's
operator and the original typed comments; this is no JS-string reparse.

Original typed comments are projected and checked once, in source order. Their
complete decoded kind/delimiter evidence and exact authored ranges must agree,
and every comment must be consumed in its original gap. Comment-bearing gaps
remain completely authored, including whitespace and CR/LF. Encoded whitespace
gaps also remain authored. Only plain ASCII whitespace gaps normalize, with
breakable binary/logical right operands using the existing printer. Unproven
non-ASCII gaps explicitly refuse rather than silently claim formatting success.

Unsupported descendants, local L1 holes, invalid framing/gaps, projection
failures and excessive document depth refuse the whole document while retaining
original diagnostics, comments, AST and decode-map storage. Syntax/gap/depth
refusal spans are decoded-relative; source mismatch spans are authored-file
coordinates. The defensive depth ceiling is 16, above the nesting achievable
under the unchanged current 31-unit L1 admission; that provider capacity remains
inherited and is not bypassed.

## Validation boundary

Three projection/custody laws pin original borrowed atom pointers, consumed
owner/Doc identity, partial-entity/UTF-8 refusal and unknown token/child framing.
Ten integration laws pin complete atom/binary/logical/parenthesis output,
operator spelling, width behavior, comments/entities, nonzero source blocks,
decode-map storage, JS/TS, CRLF and fixed points. Five whole-refusal laws retain
foreign-root, unsupported descendant, syntax/capacity and unproven-gap evidence.
A separate semantic preservation law reparses actual formatted JS/TS output in
tests and independently compares typed node kinds, operators, original
parentheses, literal values/spellings, and comment kinds/text at narrow/wide
widths. No legacy oracle or fallback grants acceptance.

Local proof checks real module resolution and Rust/Markdown formatting,
canonical Glyph inventories, diff hygiene and 20 actual inventory/storage
laws. Fresh hosted affected Rust Clippy/tests and protected full/instruction
queue acceptance remain required. No stale Rust artifact ABI proof or manual
full campaign is credited. This provider is independently reviewable and adds
no template/directive runtime integration, pipeline stage or serialization.

## Current-main integration

The independently published provider replays onto actual protected-queue main
`44533d11deede25c82d24e83fba8be7250ff3c5d`, which includes the accepted Full
directive layer. Its expression runtime and nineteen source laws remain exact;
the incoming formatter paragraph is preserved and only the two owned Glyph
shards regenerate from actual current source. Fresh exact-head hosted checks
and protected queue acceptance remain required.

## Remaining work

The merged [original interpolation operand provider](./2026-10-03-l1-original-interpolation-operands.md)
from #7491 already supplies private original-body, source-preparation and
intrinsic grammar custody through NativeInterpolationOperand. Its actual
formatter consumer integration remains separate and unfinished. Attribute
consumers, other expression families, handler/parameter/loop shapes, other Vue
dialects, SFC assembly, formatter options and checked span-edit integration
remain unfinished. The
compiler-profile AST is consumed directly here; the incompatible OXC
`parse_for_format` path, AST normalization, reparsing and legacy format helpers
are not used. Default replacement waits for
[#6882](https://github.com/ubugeeei-prod/vize/issues/6882) and complete native
corpus acceptance. Existing formatter routes, legacy bytes, budgets and oracle
policy remain unchanged.
