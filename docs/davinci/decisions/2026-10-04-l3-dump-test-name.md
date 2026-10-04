# L3 dump law test target name

Decision for [#6832](https://github.com/ubugeeei-prod/vize/issues/6832), 2026-10-04.

The audit is pinned to `ac1675fef35c5637c38b1e366a91d6bdfe419885`.
`davinci/vize_l3/tests/folio_laws.rs` directly exercises the native L3 Dump
page. Its two tests pin full canonical print identity and structural
parse/print identity for the same scheduled Program, regions, operations,
effect and ordering edge. The auto-discovered target alone retains the old
page codename.

Move this file byte-for-byte to `davinci/vize_l3/tests/dump_laws.rs`, in a
move-only commit. All Rust, test function names and the complete
`[l3-dump-v2]`/`l3.*` canonical text remain exact. Cargo discovers the target
without manifest changes. The target has no snapshots or active external
selectors to migrate. This is a target-name change, with no implementation,
wire, dependency or product default change.

## Replay and acceptance

Run `node tools/support/levels/rename-l3-dump-test.ts moves`, commit only the
move, then run `integrate` and `check`. The script refuses missing or colliding
source/destination paths before index mutation. Repeating `moves` changes
nothing. Integration adds only the paired canonical decision link after
validating its unique anchor and companion; a repeated execution changes
nothing.

Earlier L3 proof records keep their original target paths, source pins and
commands, including the original Impeto-era format record. The similarly
named `folio_laws` target in `move-croquis-dump.py` belongs to Croquis and its
different preserved snapshot; it is outside this native L3 rename.

Locked Cargo metadata must discover `dump_laws` and omit the old L3 target.
Fresh exact source-head Actions must build and execute both unchanged tests;
earlier `folio_laws` results do not authorize the new target. The protected
queue must run the complete workspace and instruction ceilings before actual
merge. The two native codec laws give no default compiler, L3 reactive
execution or legacy fix-history completion credit.

## Independent scope and remaining work

This change has no source or contract dependency on the separate L0 naming
PR #7732. It branches from accepted `main`, without an artificial Stack.
Both slices remain outside the queue during the release publication hold;
source checks can proceed. #6832 stays open for its complete naming, wire,
CLI/product capture and crate audit.

L1's corpus target is separately feature-gated and needs explicit
feature-enabled acceptance; it is not included here. All retained `.folio`
fixtures, L0 timing/page-name strings, benchmark feature spellings, other
corpus/support paths and pass snapshots remain unchanged. Published features
keep their [compatibility policy](./2026-09-29-published-feature-compatibility.md).
No historical witness, original failed campaign or declined Lean migration
is rewritten or superseded by this slice.
