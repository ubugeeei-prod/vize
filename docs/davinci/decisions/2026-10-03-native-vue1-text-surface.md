# Native Vue 1 text surface

Paired issues: [#6841](https://github.com/ubugeeei-prod/vize/issues/6841) and
[#6842](https://github.com/ubugeeei-prod/vize/issues/6842).

## Provider and ownership

`vize_l1::dialect::vue1::surface::{parse_component,
parse_component_with_authored}` constructs a version-owned, source-retaining
`ComponentParse`. Its intrinsic version is `LegacyVueVersion::V1`; its tree,
authored projection, diagnostics and unsupported syntax facts have read-only
accessors. It has no conversion to the modern component carrier. This provider
does not grant Vue 3 factory, embed, lowering or runtime admission.

The existing Vue scope sink moves without byte changes in a scripted move-only
commit from `dialect/vue3/surface/sink.rs` to
`dialect/vue/surface/sink.rs`. The integration adds static version policies to
that owner. Both versions retain the same existing tag-head iteration, recovery
tracker, scope stack, event recorder and tree construction. Concrete recognition
stays inside per-version dialect modules; generic markup profiles and the shared
Lexer state machine have no new Vue version policy.

Vue 1 enables the existing native raw-interpolation option. Complete nonempty
triple delimiters preserve three exact borrowed source pieces and a private raw
framing fact, exposed by `Interpolation::is_raw_html()`. This fact describes
lexical framing, including retained unsupported syntax; it is not expression or
runtime admission. Raw/public token constructors default false. Event width uses
the existing `aux` byte only for interpolation events, separately from quote and
opening-mode interpretations. The private token fact fits existing padding;
64-bit Token/Option/OpenTag/Element/Interpolation layouts remain
40/40/144/248/120 bytes, and Event remains 12 bytes. Public token Debug and wire
formats exclude private metadata.

Tree construction has a static raw-capable specialization. Compatibility and
modern providers instantiate the existing two-delimiter path; they do not load
the new width metadata. There is no new pipeline stage, serialization, event
buffer or whole-source/tree traversal. The historical separator admission check
inspects each retained callback content once without allocation. Numeric
instruction ceilings remain unchanged and require exact-head Actions and the
protected queue before acceptance.

## Actual historical reference

The dev-only fixture executes the complete upstream
[Vue 1.0.28 bundle](https://github.com/vuejs/vue/blob/a8d6330d7e6b30c252aa753f99c7cb73bfc67a70/dist/vue.common.js),
not a copied implementation of its regex. Commit
`a8d6330d7e6b30c252aa753f99c7cb73bfc67a70`, Git blob
`63ebe385546dec4073dacbe13878d3f6a1759053`, exact byte length and SHA-256 are
checked before the actual `Vue.parsers.text.parseText` and compiler run. The
deterministically compressed fixture retains the entire MIT license and a
source/gzip receipt. It introduces no normal/build oracle dependency or
production reference route.

Seventeen exact text cases establish ordinary/raw framing, whitespace, Unicode,
adjacency, missing third closers, empty forms, one-time markers and JS historical
dot behavior. CR, U+2028 and U+2029 do not form historical text tokens; LF does.
Six actual compiler control cases show that only a literal `v-pre` suppresses
children. Vue 1 argument/modifier forms are not terminal controls, unlike the
modern dialect's separately retained policy.

## Boundaries and remaining work

This is a bounded component-profile text provider, not complete Vue 1 grammar.
`SyntaxBoundaryKind` explicitly retains `RawDelimiterRecovery`,
`EmptyInterpolation`, `OneTimeInterpolation` and `HistoricalLineSeparator` with
authored byte spans. A lexer callback ending at `}}` before a later `}}}` cannot
prove the historical regex's preferred raw match; that recovered tree retains
bytes and an unsupported boundary. Empty and one-time semantics likewise stay
unfinished. Separators excluded by historical JS dot keep their recovered
payload and a refusal rather than becoming silently admitted historical syntax.
Sources beyond u32 offsets fail before construction. Other recovery diagnostics
remain alongside the retained tree and all later nodes.

TODO under #6841/#6842: complete Vue 1 raw-delimiter precedence without a second
lexer/state machine, empty and one-time syntax, browser document parsing,
attribute interpolation, filters, directive/typed embeds and native runtime
lowering. Vue 0.x, quirks, petite, other version-specific grammar and complete
file descriptors are also unfinished. Capability tables alone are not syntax
providers. No legacy product path or legacy behavior changes, and no compiler,
whole-dialect or product completion is claimed.

## Validation scope

Nine native source laws cover exact borrowed pieces, recovery spans, retained
attributes, literal pre, shared recovered scope, UTF-8 cuts, modern/compatibility
defaults, raw constructor defaults, Debug and unchanged layouts. Event laws
cover raw/quote aux separation. A compile-fail example documents the distinct
carrier. The actual current L1 source and existing modern/legacy dev-reference
integrations are tested with cached coherent Rust dependencies; this is finite
source evidence, not whole-current Cargo or hosted Actions acceptance. Exact
publication-head required/full checks, immutable all-100 ceilings, protected
queue and actual main merge remain required external evidence.
