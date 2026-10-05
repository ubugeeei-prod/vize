# Private L2 dump span support naming

Issue: [#6832](https://github.com/ubugeeei-prod/vize/issues/6832).
Audited actual main: `d71d8398187dfbee63c26f1e4f2946fbe9fe38f9`.

Move `davinci/vize_l1_to_l2/tests/support/folio_spans.rs` to
`tests/support/dump_spans.rs` in a byte-identical move-only commit. A separate
integration commit changes only the private module declaration and its
re-export prefix in `tests/support/mod.rs`.

The complete oracle body and `assert_folio_spans_resolve` spelling stay exact.
All five `ir_contract_spans` law identities and bodies, the committed battery,
optional corpus environment and output, diagnostic text, fixture and wire bytes
remain unchanged. The existing tooling witness keeps its original assertions.
Callable names and other codename residues remain separate unfinished work.
Cargo manifests, features, test targets, the lockfile and all normal, build and
dev dependencies retain their bytes. This private module already compiles in
default tests; it needs no new feature-enabled test lane.

Replay with `vp node tools/support/levels/rename-l2-span-support.ts moves`, commit
only the move, then run the same script with `integrate` and `check`. The script
uses Node's TypeScript support. It checks the original SHA-256,
old/new collisions, both module anchors and the paired decision before any
write. Repeated replay is idempotent and partial integration is rejected.

Fresh exact-head Actions must compile and execute the unchanged default span
controls and existing tooling witness. The full protected queue must retain
all configured differential commands, pass the unchanged 100 level and four
formatter instruction ceilings and actual-parent ratchets, and actually merge
before delivery is claimed. No local Rust runtime campaign is used.

This slice is independent of the held Draft #7757 surface-corpus rename and
has no production, For/setup, L0 platform or Vue 1 descriptor source overlap.
Keep #7757's original allocation failure and source freeze intact. Both #6832
and #6835 remain open; this rename supplies no product history, default-route,
whole-dialect or zero-fallback completion credit.
