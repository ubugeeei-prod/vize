# Inspector owns its stage feed (2026-10-01)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).

The feed model and JSON serializer live in `vize_curator::inspector::feed`,
next to the inspector that produces and exports them. The existing public
inspector re-exports keep CLI/Build/Playground consumers on the same types.
The old substrate has no feed re-export. Collector, plan, remark and timing
contracts import L0 directly, removing Curator's normal compatibility-substrate
dependency. No native level gains a legacy dependency.

A separate move-only commit relocates the serializer and its six schema laws.
Version negotiation, page order, remark field order, gating and JSON escaping
stay unchanged. The schema and protocol bytes are frozen; the shared string
escaper and remark writer remain in L0. This relocates existing serialization
at the host boundary and adds no pipeline stage or between-level conversion.
The module stays IO-free; Curator is a host library, so it makes no crate-level
no-std claim.

Replay `python3 tools/support/levels/move-inspector-feed.py moves`, commit
those two moves alone, then run `integrate`, format Rust, normalize Cargo
metadata and regenerate source inventories. Move preflight checks all targets
before changing the index. Replaying integration on the resulting tree makes
no additional changes. Current test commands and local links follow their
owners, and the native storage inventory drops the host-owned serializer row.

Local verification passed thirty-five storage/dependency/target tests and all
six feed schema laws plus ten existing inspector payload/ladder/timing laws.
Full Actions, WASM consumer checks and the protected queue remain required
before merge. #6833 remains open for repro pages, legacy test plans, other
imports and final substrate deletion; native product migration is unfinished.
