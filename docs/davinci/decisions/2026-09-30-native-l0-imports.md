# Native consumers use the L0 owner (2026-09-30)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833), following
[the runtime ownership move](./2026-09-30-l0-runtime.md).

Level and conversion crates, extension contracts/hosting, and MoonBit use the
canonical L0 key, Dump, pass/fact, diagnostic/witness and identity APIs directly.
Their Cargo dependency declarations no longer name `vize_davinci`, including
dev dependencies. The contract acceptance module names its diagnostic alias
`diag`. This changes Rust paths and dependency ownership; wire schemas,
historical fixtures, generated JS and instruction ceilings remain unchanged.

Replay `python3 tools/support/levels/migrate-native-l0-imports.py` on fresh main,
then `cargo fmt --all` and `cargo metadata --offline --format-version 1`.
The dependency graph test pins the new direct edges; the whole directory gate
continues to enforce zero reverse normal/build paths. The existing artifact,
projection, protocol and compiler corpora validate the same implementation
through its canonical owner.

The old package still serves legacy products and test consumers for renderer,
summary and Croquis/repro/feed pages. Move each to its assigned owner, migrate
those users and tests, and delete the old package before closing #6833. Direct
L0 imports alone do not complete the higher-level native artifacts or any
product-route migration.
