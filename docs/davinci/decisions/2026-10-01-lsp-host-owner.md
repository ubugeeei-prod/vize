# LSP host ownership

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Provider: [UTF-16 coordinates](./2026-10-01-utf16-coordinate-owner.md).

Carton owns the existing JSON-RPC records, message framing, request IDs, LSP
method constants, host diagnostics and Vue host type-marker classification.
L0 retains only the protocol-neutral UTF-16 `line_index::Position` and
`line_index::Range` value types. `vize_carton::lsp` re-exports those exact
provider types, so indexed coordinates cross the host boundary without a
conversion, allocation, new representation or reversed level dependency.

The existing protocol implementation, its fixed message snapshot and the
coordinate identity law move in a move-only commit. Integration changes the
module owner, adds Carton's serde dependencies and retains a module-scoped
lint expectation for the existing std String/Vec wire representation. The
expectation does not extend to any other Carton module. JSON field names,
message content and framing, request counter ordering, Unicode columns and
Vue classification behavior remain unchanged. The sole existing protocol
consumer already imports `vize_carton::lsp`, so no product route changes.

Replay with `vp node tools/support/levels/move-lsp-host.ts moves`, commit the pure
moves, then run `integrate` and `check`. The script requires the UTF-16
provider and rejects source/target collisions or unmoved sources. Integration
is idempotent and removes redundant target-specific serde_json declarations
while preserving the host resolver's native-only `which` dependency.

This is the child of the coordinate provider in a native GitHub Stack. Both
heads require their own Actions validation; the ready prefix enters the
protected merge queue together. Numeric instruction ceilings and legacy
fixtures are unchanged. Config, internationalization, profiler and full L0
platform isolation remain open under #6834; this move does not close it or
establish whole-product native adoption.
