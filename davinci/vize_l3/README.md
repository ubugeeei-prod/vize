# vize_l3

`vize_l3` is Davinci's L3 reactivity IR. It records backend-oriented UI
work as flat, id-addressed operations with explicit state and effect edges.

`decision::build_decisions` separately computes conservative native decisions
in one canonical L2 event walk. Its `NativeAnalysis` retains the actual sealed
L2 owner, selected policy and read-only tables as one input to L4. Local ids
from another artifact cannot establish that association. Placement remains
`Inline`; complete grouping, placement and product selection are unfinished.

`Program::operands` retains value payloads beside the compact graph. Operands
identify their owning operation, binding target or conditional region, semantic
role, source span, and value kind. Missing values differ from empty literals;
opaque and foreign expressions remain classified rather than becoming JavaScript.
The payload text belongs to the L3 arena and does not borrow an earlier stage.

`L3ValuesFolio` is the owned value-page companion to the graph-only `L3Folio`.
Its `operand=` JSON tuples preserve escapes and authored ordering. Equality of
these serialized rows is document equality, not semantic expression equality.
The `L3V009` verifier checks value shape, source containment and references in
every phase. This is a value-transport contract, not yet executable stateful
Lean semantics or a replacement for the existing backend payloads.

`Program::placements` is the P3-10 placement overlay. `placement::annotate`
records where each op's work may run instead of inline: a hoisted static
subtree under a control region, a cached scope-free event handler, or a leaf
update grouped with the adjacent op reading the same direct reference. The
overlay never rewrites the graph, so exported partition facts stay canonical;
`L3V010` re-derives every recorded alternative and committed choice, and
`L3PlacementFolio` prints the overlay on its own page.

`extract::extract` is the try-measure-commit pass over that overlay. Each
candidate is performed on a trial plan, measured (emitted size, reactive-edge
count, update-path length), and committed only under the `-O` tier's pinned
rule and candidate budget, which mirror `budgets.toml`. Every candidate yields
one applied or missed decision, printed on `L3ExtractionFolio`.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
