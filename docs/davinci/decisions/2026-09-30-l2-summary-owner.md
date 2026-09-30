# L2 owns interface summaries (2026-09-30)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833), after
[native consumers leave the substrate](./2026-09-30-native-l0-imports.md).

Move the existing per-SFC and project summary implementations to
`vize_l2::summary`, together with their invalidation and round-trip tests.
Resident, Croquis, LSP and bindings use that owner directly. The old package
does not re-export summaries or gain a dependency on L2. Croquis and LSP add
their explicit L2 dependency; the other consumers already declare it.

The move preserves declaration fingerprints, hash-domain tags, alpha schemas,
serialized Dump bytes and invalidation rules. It adds no pipeline stage or
serialization and does not replace a legacy product route. The existing
mixed hash-domain vectors continue to validate the actual L0 and L2 producers.
Storage inventory rows keep their exact counts and move from infrastructure
to L2 scope; generated source witnesses follow the moved tests.

Replay `python3 tools/support/levels/move-l2-summary.py moves`, commit the
eight file moves alone, then run the script with `integrate`, format Rust,
normalize locked Cargo metadata and regenerate source inventories. The
standalone script remains usable after its sources have moved.

Local verification passed nine summary/invalidation laws and the exact fixed
hash-domain vectors (the separate capture-only test stays ignored). Actions
must validate the consumer crates and full corpus before merge. #6833 remains
open for renderer, Croquis/feed/repro pages, legacy test plans, other imports
and final compatibility-package deletion.

The cross-platform TS-43 workflow runs its moved summary laws under L2 and retained key laws under their existing owner. Its trigger covers the complete L0 key/fact foundation. The workflow contract test passed with both Linux/macOS commands retained.
