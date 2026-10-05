# Bare template props follow the native property declaration

Decision for [#7917](https://github.com/ubugeeei-prod/vize/issues/7917).

The checker resolves a bare template prop to its synthesized const declaration.
For imported props that declaration has no authored mapping; for local props
its existing anchor can be the withDefaults default key. The typed property
access already retains the interface declaration owner. Record the emitted
const-name and property-key byte ranges as a semantic edge at generation time,
including the imported keyed-prop path. Generated TypeScript bytes and existing
mappings stay unchanged. Before mapping a native template definition response,
follow only the exact synthetic declaration endpoint and ask the native checker
for the linked typed property's definition. Ordinary definitions and lexical
shadows retain their existing identities.

This adds no source-name sweep, reconstructed property access, legacy AST
fallback, new pipeline stage or edit-time project discovery. The additional
native definition request occurs only when a response names the recorded edge.
No native migration, complete history, instant response or 10x gain is claimed.

The corpus preserves all three complete original sources and configured strict
ownership. Source-built typed stdio compares complete definition objects at all
four original coordinates, the exact matching interface member and bare hover
type/range. Controls retain imported/local targets, foreign same-name members,
script parameters, v-for shadows, LF/CRLF, astral source/target positions and
complete versioned unsaved diagnostic publications. A generator law checks the
real emitted endpoints after import rewriting and UTF-16 round trips.

The first hosted compile caught the new edge in exhaustive protocol and native
inspection matches. Keep this Vize-only navigation edge out of upstream protocol
v1, retain its inspection name, and extend the complete existing protocol law;
native definition expectations and all instruction ceilings remain unchanged.

Fresh exact-source Actions, unchanged protected instruction ceilings, actual
merge, release and installed editor proof remain required.
