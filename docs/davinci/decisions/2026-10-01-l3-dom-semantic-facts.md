# Bounded native DOM semantic facts

Issue pair: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839) and
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

The authoritative stage boundary remains **L3 decides, L4 encodes**.
`NativeAnalysis` retains private DOM facts with the exact immutable L2 owner.
The producer feeds optional DOM scratch from its existing enter/leave events
and computes these facts in the same single `Artifact::visit_events` call. It creates no
second tree walk, numbering authority, parser, flat program or serialized IR.

The bounded facts retain the exact borrowed `Op` and `BindingOp` payloads from
their original events. L4 follows ordered canonical ids and these payloads;
it does not build a lookup index or replay the L2 walk. Public accessors expose
only read-only facts through the owner-bound analysis. SSR/Vapor do not allocate
these DOM tables or scratch frames. The shared neutral frame keeps its original
layout; DOM scratch lives solely inside the optional target builder.

L3 records contiguous text/interpolation groups, direct text versus mixed child
sequences, root direct/fragment eligibility, single non-comment fragment roots,
and root-element block eligibility. It records which admitted text, class,
style and ordinary property values can change, preserving binding order and
ordinary dynamic-property binding ids. These facts contain no Vue flag bits,
runtime helpers, dynamic-property strings, emitted normalization or block syntax.
L4 owns those encodings and source links.

The default expression policy proves retained literal ASTs only. Unsupported
nonliteral expressions are explicit, never silently classified as dynamic.
`build_dom_decisions` accepts a statically dispatched semantic provider through
`DomExpressionFacts`, using the same traversal. The actual scoped `ContextOnly` provider checks complete L2 resolver results. It must
match the exact AST, source, span and native coordinates against a complete L2
resolution table, require at least one occurrence, and require every binding id
to belong to the explicitly declared context registry. Zero-reference nonliteral
expressions require a real constant policy and remain unsupported. Script setup
access spelling and global-name guesses never prove constantness.

The initial surface admits native HTML elements, text, comments, interpolation,
ordinary static attributes, and unmodified static-name bindings. Class/style
bindings have semantic roles distinct from ordinary properties. Unsupported rows
retain their canonical id, actual authored span and typed reason in event order.
Consumers must reject such a result before emitting an admitted fragment.

Unfinished: full file binding and script/effect constant analysis, static class/style normalization, key/ref/is/on-property
semantics, modified or dynamic-name bindings, object spreads, other directives,
components, control flow, slots, SVG/MathML and namespace crossing, placement,
hoists/caches, complete native target parity and compiler product selection.
Existing neutral decisions and `Inline` placement remain unchanged.

Storage is source-sized analysis scratch: ordered text ids/groups, dynamic
property binding ids, binding-name duplicate checks and typed unsupported rows.
The per-file inventory explicitly reviews the new `alloc::Vec` counts.
The scoped provider borrows complete resolution tables and a shared context-id
registry; it allocates no occurrence lists and scans no AST. Sparse
side tables remain; no allocation or instruction improvement is claimed.
Narrow laws use real canonical artifacts and retained ASTs. Full Actions,
differential output, all 100 pinned instruction ceilings and the protected queue
remain required before merge; registry identities and ceilings do not change.
