# Original physical newlines in decoded expression gaps

Paired issue: [#6847](https://github.com/ubugeeei-prod/vize/issues/6847).
Decision: [checked authored gap retention](https://github.com/ubugeeei-prod/vize/issues/6847#issuecomment-5969436848).

This independent repair starts in a new `wt` from signed actual `main`
`62d3926a4070356deb48c64f2d37a218d9f3c4b8`. The genuine retained-expression
Doc provider, original selected interpolation consumer and unary extension
already merged through the protected queue. Their accepted source heads and
original raw evidence remain unchanged.

The actual `vize_l1::container::vue::scan::skip_interpolation` scans authored
bytes before text decoding. Encoded quotes around literal `//` do not start
its raw quote state; only physical LF ends that raw boundary. For example,
`(&#39;//x&#39;` followed by LF and `)` contains a genuine decoded string
literal, no typed comment, and an internal identity-map whitespace gap.
Normalizing away that LF can lose full Descriptor admission. Keeping only
the original trimmed interpolation tail does not cover an internal gap.
This source inspection identifies the requirement; hosted actual-provider
controls still have to prove the complete input/output behavior.

The existing shared Doc gap helper first validates original typed comments,
decoded ASCII whitespace and checked authored projection. In addition to its
existing comment and encoded-byte rules, it now borrows the complete authored
gap when the original source has a decode map and that gap contains LF.
It preserves authored LF/CRLF without classifying raw comments or changing
the original AST, map or source. Unmapped identity sources still normalize
ordinary gaps. An entity-decoded LF never supplies a physical terminator.
Generated printer line endings remain independently selectable.

Independent standalone controls cover complete mapped parentheses/unary and
binary/logical output, identity/no-reference normalization, original
AST/map/source storage and real JS/TS AST/comment reparsing with fixed points.
Selected controls use the actual Descriptor and genuine original operands:
encoded single/named-double quote string contents require physical LF;
complete absent-LF/CR-only source mutations retain Descriptor refusals.
Whole printed blocks at widths 0/1/7/80/200 preserve LF/CRLF, and actual
Descriptor re-selection/reparse compares typed syntax, original atom spelling
and complete authored/decoded comments before verifying a fixed point.
Additional laws retain original owner/operand/AST/comment/map/source addresses
and unchanged real typed-comment bytes. A genuine computed-member input
proves only its original provider framing here. Complete Member Doc output
is deferred to the private consumer after this prerequisite actually merges.

Local formatting, source checks and the two owned canonical Glyph shards
precede exact-head hosted source/capture acceptance. No local Cargo cache or
frozen executable supplies runtime credit. All affected native laws, protected
full suites, every unchanged instruction ceiling and actual signed merge are
required. Prior Unary protected evidence proves its actual bounded laws;
this newly identified internal-gap coverage was absent there.

TODO: Member consumer acceptance, other expression families, directive values,
all Vue dialects, other embed shapes, enclosing SFC assembly, options and edits
remain unfinished. [#6882](https://github.com/ubugeeei-prod/vize/issues/6882)
still gates default replacement. This adds no parse/decode, visitor, L1 API,
pipeline stage, legacy helper, shipped route, oracle change or budget increase.
