# Original expression literal receipt (2026-10-03)

Issue: [#6836](https://github.com/ubugeeei-prod/vize/issues/6836).
This is a bounded prerequisite for the original selected interpolation path,
not a complete strict-module or output certificate.

## Decision

Expose `AdmittedExpression::has_legacy_literals()` as a read-only borrow of
`ParserReturn.has_legacy_literals` through the existing complete original
`ExpressionObservation` and `EmbeddingObservation` owner chain. The stock
lexer already creates this scalar while decoding committed numeric literals
and string escapes. The existing checkpoint and committed reparse behavior
remain unchanged. No new stored flag, source scan, AST traversal, reparsing,
serialization or pipeline stage is added.

The fixed original expression wrapper contains no literal, so the receipt
belongs to its original embedded content. Ordinary syntax admission and
diagnostics remain unchanged. A true receipt identifies spelling forbidden
in strict modules; false does not establish early-error, context, reference
or runtime validity. Raw ASTs or caller-supplied source cannot construct the
private admitted expression owner.

## Original-source laws

Four genuine L1 laws start with the actual Vue Descriptor and selected
Component, observe the original interpolation at its existing body event,
and read this borrowed scalar through the original admitted expression:

- Legacy decimal/octal spellings and octal/non-octal string escapes retain
  the existing true scalar alongside complete content and stock comments.
- Modern numeric forms, valid escapes, escaped literal backslashes, comment
  text and raw tagged-template text retain the existing false scalar.
- Original HTML entity decoding, intrinsic TS Module grammar and operand
  moves preserve the actual expression/comment addresses and receipt.
- Syntax holes retain complete original content, comments and diagnostics;
  they cannot supply an admitted expression receipt.

The laws run through affected `vize_l1` Rust tests in hosted Actions. They
verify the original receipt API rather than changing parser syntax behavior;
no new parser coverage fixture or local Cargo run is required. Fresh
exact-head Actions and the protected full/instruction queue decide delivery.
All numeric ceilings and legacy product behavior remain unchanged.

## Remaining work

The genuine L2 input and root File association in PRs #7576 and #7582 must
actually merge and remain normally owned. A DOM target may consume this
receipt only by joining that same actual File interpolation association at
its existing sole L3 visit. A neutral expression or detached scalar cannot
grant original File completion. Literal output still requires complete
module/map/runtime laws against pinned Vue and the actual selected-template
DOM provider. Broader expression validity, reads, nested interpolation,
controls, whole-SFC default replacement and roadmap completion remain
unfinished.
