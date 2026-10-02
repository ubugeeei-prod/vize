# Native Vue interpolation source consumer

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836) and
[#6838](https://github.com/ubugeeei-prod/vize/issues/6838), after the independent
L1 `prepare_vue_interpolation_in` provider and the checked retained-expression
coordinate/factory providers.

The real Component consumer replaces raw interpolation source borrowing with
the checked L1 helper: select authored HTML ASCII trim bytes 9/10/12/13/32,
then decode in TEXT context once, before the same single Expr `parse_once`.
The narrowed checked EmbedSource remains with the original retained AST,
comments and complete owned Diagnostics. Artifact, operation and provenance
spans retain the full authored interpolation including delimiters and trivia.
No L4 trim, expression field, second decode, parse or AST walk is introduced.

A small per-use admission callback runs in the existing expression construction
path before coordinate adaptation or id mint. For the actual Identifier root,
NBSP/BOM at the actual prepared source edge remain a typed
`InterpolationIdentifierTrivia` refusal until Vue's exact spelling is supported.
The full valid retained observation stays in NativeEmbed with node=None;
supported later fragments keep their actual consecutive ids. Characters inside
comments and literals never trigger this edge check. Generic retained-expression
adaptation and explicit named directive binding admission remain neutral.
Pinned Vue 3.5.35 complete module/map observations for leading/trailing NBSP
and BOM retain that edge in the prepared content and identifier map name, and
prefix `_ctx.` before the complete raw spelling. A native AST-span rewrite has
a different spelling, so these four observed families remain explicit refusals.

Authored ASCII trim can remove a real newline after a trailing line comment.
L1's private wrapper newline permits syntax parsing but is never authored source.
The unchanged L2 `OutsideExpression` guard continues to refuse that expression
before mint/output, with the exact comments/source/observations preserved.
No separator is guessed or invented and no shared guard is relaxed.

Five new source-law groups cover actual narrowed/full ranges and provenance;
once-decoded multi-scalar entities and unchanged AST pointers; removed trailing
comment newline and continued later nodes; raw/entity NBSP/BOM edge refusals;
characters inside comments/literals; and unchanged neutral named bindings.
All 31 actual native Component/pattern/interpolation laws pass with the real
whole L1 source and the complete current L2 source registration. Strict native
production Clippy and the canonical inventory/storage checks pass. This is
scoped source evidence, not current whole-workspace or performance-gate proof.

The helper is an independent actual-main L1 prerequisite. The private consumer
patch must be replayed after that provider and the actual checked L2 factories
in a true native GitHub Stack; its research source ancestry is not publication
ancestry. Exact-head full Actions, unchanged all-100 ceilings and protected
actual merge remain mandatory. Other interpolation spelling/admission families,
file language resolution, controls/events and product-route migration remain
unfinished rather than being hidden behind a legacy parser or emitter.

The opening-mode consumer follows the separate native L1 grammar correction
and resolved-mode provider. It reads `OpenTag::is_verbatim()` before the existing
attribute loop. Refused own or inherited v-pre owners retain the complete
original carrier and one PreCarrier refusal without decomposing ignored heads,
decoding ignored static values or admitting descendant embeds. There is no
temporary hole rollback, new buffer, second head/tree scan or added ordinary
allocation. Missing-owner facts and earlier tags' admission facts stay intact.

Seven regression groups cover both positions of an ignored over-budget head,
both positions of equal-length literal/entity static values, original earlier
tag admission diagnostics/provenance, full pre arguments/modifiers, and
shorthand/refused dynamic/no-colon noncontrols. The equal-value law measures
only arena bytes; it makes no whole heap or instruction-cost claim. Retained
embed count is measured separately from the source parse-once contract.

The genuine source chain now includes the actually merged checked core and
interpolation helper, actual signed f634 main including Params, Dense and
resolver providers, and the immutable native opening-mode provider. The
independent metadata/correction layers are not copied into this consumer.
All 31 native producer and 51 current L2 laws (44 unit and seven resolver
integration) pass with whole actual L1 source and coherent cached dependencies.
Strict production Clippy also passes. This is source evidence, not a fresh-L0
or workspace build. Canonical source checks are required before freezing.
Exact-head complete Actions, unchanged all-100 ceilings and protected
actual merge remain mandatory; these source checks do not establish them.
