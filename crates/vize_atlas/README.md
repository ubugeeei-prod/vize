# vize_atlas

Source Atlas is the neutral request and capability ledger for Vize's multi-input
and multi-output compiler infrastructure. It records which source families,
dialects, semantic products, targets, and fallback routes were requested.

Atlas deliberately owns no AST nodes, semantic analysis, or render lowering:

- Relief owns source syntax.
- Croquis owns derived semantics and graphs.
- Rendu owns the output-facing render projection.
- Atelier crates own transformation and emission.

The ledger is allocation-free to construct and inspect. That is a property of
the compiler abstraction itself, not a claim about generated runtime code.
