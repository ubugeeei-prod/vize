# Native opening-tag mode fact

This is a bounded provider for [#6836](https://github.com/ubugeeei-prod/vize/issues/6836),
its real [#6838](https://github.com/ubugeeei-prod/vize/issues/6838) consumer and
the [#6841](https://github.com/ubugeeei-prod/vize/issues/6841) Vue syntax policy.
Native L2 control emission, full Vue grammar/admission and product completion
remain unfinished. Compiler products keep their preserved parser routes.

## The actual consumer defect

The native surface already resolves each opening tag's lexical mode after
shared recovery. Its L2 consumer instead decomposes and prepares every
attribute before discovering `v-pre`. A balanced but over-budget unrelated
directive on the same tag creates a false `DirectiveSyntax` refusal, diagnostic
and provenance record even though the retained L1 carrier correctly has no
unsupported admission. This occurs before and after the control attribute.

Equal-length ignored static values `xxxxx` and `&amp;` use identical L1 arena
bytes, but the old consumer decodes the latter into discarded semantic payloads
before refusing the pre owner. A hole-list rollback cannot remove this work or
the already committed diagnostic/provenance strings. The real dependent
consumer must guard on the existing resolved opening fact before any attribute
semantics. It still records `PreCarrier`, keeps the complete L1 carrier and
does not claim to emit native L2 pre semantics.

## Provider and storage contract

`VueSink::finish_tag` returns its already computed own/inherited verbatim mode
after existing recovery. Its one existing opening-end event stores that bool
in `Event::aux`. `AttrEnd` continues to interpret the same byte as quote type;
the event kinds are disjoint. All compatibility recorder events default to
zero. There is no extra event, source/head/tree scan, buffer or pipeline stage.

The existing tag-building loop transfers the fact into a private bool in the
opening-name token's padding. `OpenTag::is_verbatim()` exposes the resolved
lexical fact; it is not a directive semantics or syntax-admission witness.
Normal and authored projections share the recorded fact, even when authored
parentage differs from the recovered live owner. Inherited descendants,
implicit closures, void/self-closing tags and scope exits retain their
existing owner decisions.

On 64-bit targets, the measured sizes remain Token 40, Option<Token> 40,
OpenTag 144 and Element 248 bytes. Event remains 12 bytes. Existing size and
storage caps do not increase, and ordinary construction adds no allocation.

The bool has no public setter. `Token::present` constructs raw syntax with a
false fact; compatibility parsing and raw extension-page materialization also
default to false. The three differential mutator literals use that same raw
constructor. The extension materializer retains its existing public missing
status assignment. PageToken wire fields and serialization are unchanged.
Token's manual Debug prints only its original leading/text/status fields in
the original order, preserving public tree/dump bytes rather than exposing
private metadata.

## Pinned upstream control proof and the narrow name correction

The actual installed `@vue/compiler-dom` 3.5.35 package is the independent
policy oracle, guarded by an exact version assertion. Its
[parser implementation](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/parser.ts)
enables pre from the full logical name. Plain `v-pre`, `.foo`, `:arg`, `:[key]`,
`:.`, `..`, `:` and malformed `:[broken`/`:[`
all suppress descendant interpolation. The malformed dynamic arguments keep
their actual recovery diagnostic. An argument/modifier prohibition would
contradict this pinned parser and is not introduced.

A bracket without a preceding colon belongs to the full name. Thus
`v-pre[broken` and `v-pre[key].foo` do not activate pre. The native checked
provider previously split at that bracket. The narrow correction removes
only `[` from the full-name stop set; colon and dot still delimit arguments
and modifiers. It uses the existing single decomposition and introduces no
recognition pass. Independent coordinate and native-source goldens pin the
whole name, no argument, ordinary descendant interpolation and raw fidelity.

The shared legacy tokenizer still accepts a no-colon bracket as an argument.
Its production callback and parser behavior stay unchanged. The dev-only
actual AST law now explicitly asserts this native/upstream versus legacy
policy difference instead of asserting false equality. Complete lexer parity
remains unfinished: the native generic lexer can still report its existing
dynamic-argument recovery diagnostic for that malformed no-colon source.

Both same-tag orders of an unrelated balanced deep head plus admitted plain
pre are raw in the pinned upstream parser. Vue's converted semantic attribute
name can omit the colon in one order; the oracle uses its authored location
and actual raw child policy. Native surface fidelity retains the authored
name bytes. A sole over-budget pre retains `NestingLimit`; raw mode is never
invented without a checked admitted control. Similar names and shorthand
`:pre`, `@pre`, `#pre`, `.pre` stay ordinary.

## Evidence and remaining work

Independent laws cover own/inherited/recovered opening facts, both surface
projections, quote interpretations, default-false raw construction, public
Debug, layout bounds, typed admissions, malformed UTF-8 cuts and exact source
fidelity. The existing frozen original-machine contracts and legacy product
routes remain intact. The upstream package oracle and native goldens do not
compare two implementations of the same recorder.

The assertion-lint review replaced three weak substring checks with exact
oracles: complete page-wire goldens for both fixtures, the public opening-tag
Debug projection against the raw compatibility owner, and authored byte-cut
expectations for every UTF-8 boundary. No assertion allowlist or gate changes.

The dependent L2 fix must keep original Surface and DirectiveAdmission facts,
skip ignored attributes before decoding or extra diagnostics, retain raw
children and avoid minting fallback nodes or parsing ignored expressions.
Its independent same-tag/arena laws prove the observed defect; retained embed
counts alone are not an instrumented parser-call count or a whole-heap
allocation proof.

The 64 delimiter-run admission bound, complete directive/shape/language
dispatch, all Vue dialects, complete surface/core isolation and product
fix-history gates remain unfinished. Exact-head Actions, source suites and
all 100 unchanged instruction ceilings are still required before protected
queue acceptance and actual merge. Local source proofs are not merge evidence.
