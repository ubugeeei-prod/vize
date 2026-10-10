# Inline immutable setup bindings in slot defaults (#8142)

`controls.json` fixes 24 complete parameter outputs before implementation or
product execution. The bounded change keeps a default-RHS name lexical only
when the existing codegen context says `inline` and its existing binding kind
is `LiteralConst` or `SetupConst`. Local declarations, nested function shadowing,
globals, object shorthand, unknown names, absent metadata, and non-inline mode
keep their existing decisions. Ref, maybe-ref, mutable, reactive, props, data,
and options bindings are negative preservation controls; these expectations
make no claim that their broader default-RHS semantics are correct.

Both source unit tests retain all 24 complete contexts, inputs, and outputs
before comparing complete parameter strings. Their packets are separate from
compiler and mounted runtime acceptance, under
`target/differential/slot-default-setup-bindings-8142/`.

`original-runtime-cases.json` is a byte-exact copy of the six independently
authored full SFC inputs from the original943 default probe. The separate
`original-runtime-desired-laws.json` preserves its 12 complete selected/legacy
desired runtime laws without changing their initial tree, records, phases,
handler identities, or errors. Original complete compiler/runtime
classification is preserved byte-exact in `original-before.classification-audit.json`,
including all 36 runtime laws and 24 whole compiler comparisons across the original
three source states. Two default laws failed with `_ctx.fallback` and an empty
initial slot. This historical evidence does not qualify the new source or
attribute an improvement to any earlier unrelated change.

The production Core path visits its already-retained parameter AST. The native
path still uses its existing guarded wrapper parser. This change adds no parser
or pipeline stage and does not claim universal direct-parameter custody. It
borrows existing context; no metadata cloning or hidden identifier storage is
introduced. Complete SFC compilation and actual mounted runtime validation are
separate gates. Fixture authoring and static review do not qualify either gate.

The source Actions job runs both complete 24-packet unit vectors and the actual
six-source/two-lane SFC compiler and mounted runtime child. Every raw packet is
uploaded even on failure. The runtime borrows the existing checksum-pinned Vue
loader and the already locked TypeScript 6.0.3 consumer from the UI workspace;
no new package or dependency version is added. The complete authored runtime
laws are judged only after all actual observations are retained.
