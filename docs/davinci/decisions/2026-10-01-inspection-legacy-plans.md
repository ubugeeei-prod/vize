# Inspection library owns legacy plan metadata (2026-10-01)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).

The historical DOM/SSR/Vapor traversal declarations live in
`vize_curator::legacy_plan`, next to crash-report metadata. The CLI already
imports Curator normally and selects these same plans for its legacy compile
report. Backend walk-baseline laws use version-less, path-only Curator dev
dependencies, which are stripped from packaged manifests. Published backend
libraries gain no normal Curator dependency and native levels gain no legacy
edge.

One file moves in a separate move-only commit. The declared stages, pass names,
mandatory barriers, preservation sets and group counts stay unchanged. The
actual traversal assertions keep their measured production floors. These are
historical baseline declarations; native per-pass capture remains unfinished.
The old substrate has no plan re-export. The previous pre-publication packing
rationale is replaced with the current host owner and oracle boundary.

Replay `python3 tools/support/levels/move-inspection-legacy-plans.py moves`,
commit the move alone, then run `integrate`, format Rust, normalize Cargo
metadata and regenerate source inventories. Preflight checks the source and
target before changing the index; integration is idempotent.

Local verification passed all three plan laws and all three actual backend
walk-baseline harnesses over their six-fixture ladder, plus thirty-seven
storage/dependency/publish-firewall checks. Packaging and protected full queue
validation remain required in Actions before merge. #6833 remains unfinished
for the remaining consumer/test imports and final compatibility-package
deletion; this change adds no stage, serialization or product route switch.
