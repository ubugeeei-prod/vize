# Level dump naming (#6832)

Collapses the #6906..#6947 stack into one change. Level crates have no
compatibility users, so no aliases or fallback readers are added.

- **L3 package identity.** `crates/vize_impeto` moves to `crates/vize_l3`
  (move-only commit) and the package/dependency name becomes `vize_l3`.
  First-name semver checks use the published `vize_impeto@0.429.0` baseline.
- **Stage view paths and feed types.** Playground `features/davinci/` becomes
  `features/stages/`; DOM witnesses drop the `davinci_` prefix. The feed family
  becomes `StageFeed`, `StagePage`, `StageRemark`, `StageNegotiation`; its JSON
  schema and negotiation behavior are unchanged.
- **Conversion registry.** `L2_TO_L3` is registered (metadata only).
- **Dump API.** Shared `vize_davinci::dump::{Dump, Mode, Error}`, the `Dump`
  derive with `#[dump(name = ...)]`, and per-level `dump::Page` replace the
  codename page types.
- **Versioned dump protocols.** L2 writes `[l2-dump-v2]` (published extension
  `s2-page@1` keeps its historical grammar via `dump::historical::v1`). L3
  writes `[l3-dump-v2]` with sixteen `l3.*` op mnemonics. The codec is chosen by
  typed constructor; there is no header inference.
- **Hash domains.** Artifact-key, key-manifest, SFC and global summary domains
  move to `.v2`; old cache records miss. Compiler output is unchanged.
- **CLI.** `vize dump --level l1|l2|l3 --roundtrip FILE` checks byte identity of
  a parse/print roundtrip. Exit 0 identity, 1 failure, 2 usage.

Instruction ceilings: the renames moved the process memory map, and
`stacker`'s first-use stack probe parses `/proc/self/maps` inside the measured
window. The harness now takes that probe before any window (the v-for lowering
probe drops from ~150k to ~21k instructions). The patina markup visit also
reads its rule name once and pre-sizes the content-model fact maps.

Dropped from the stack: one-shot rename/replay scripts under
`tools/support/levels/` and their tests, and the per-slice evidence records.
Legacy products stay byte-exact. Remaining #6832 work: formal/Lean namespaces,
shared production capture for CLI and playground.
