# vize_impeto

`vize_impeto` is Davinci's S3 reactivity IR. It records backend-oriented UI
work as flat, id-addressed operations with explicit state and effect edges.

`Program::operands` retains value payloads beside the compact graph. Operands
identify their owning operation, binding target or conditional region, semantic
role, source span, and value kind. Missing values differ from empty literals;
opaque and foreign expressions remain classified rather than becoming JavaScript.
The payload text belongs to the S3 arena and does not borrow an earlier stage.

`S3ValuesFolio` is the owned value-page companion to the graph-only `S3Folio`.
Its `operand=` JSON tuples preserve escapes and authored ordering. Equality of
these serialized rows is document equality, not semantic expression equality.
The `S3V009` verifier checks value shape, source containment and references in
every phase. This is a value-transport contract, not yet executable stateful
Lean semantics or a replacement for the existing backend payloads.

`Program::placements` is the P3-10 placement overlay. `placement::annotate`
records where each op's work may run instead of inline: a hoisted static
subtree under a control region, a cached scope-free event handler, or a leaf
update grouped with the adjacent op reading the same direct reference. The
overlay never rewrites the graph, so exported partition facts stay canonical;
`S3V010` re-derives every recorded alternative and committed choice, and
`S3PlacementFolio` prints the overlay on its own page.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
