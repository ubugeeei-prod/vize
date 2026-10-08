# Reviewed Rust 1.98 to 1.99 instruction methodology transition

The active 100 level and four formatter instruction registries move only their
compiler identity to `rustc 1.99.0 (b940084d7 2026-09-28)` and their measurement
provenance to the complete original-input evidence from signed Actions producer
`1a3996963af208c13cd732420bdee586df88bf7a`, run `37653130635`. Every original numeric
cap, fixture path, fixture SHA256, window, inventory and non-rustc methodology
field remains unchanged. These are inherited ceilings; the refreshed compiler
metadata does not replace them with the newer, lower observed instruction totals.

The numeric ceilings retain their original Rust 1.98 provenance: level source
`b79010ff63b21612ab1dcc927a221f4000a65fe5`, run `36307058591`; formatter source
`c225f11707b6e42c08ea5379df255fb3ddb5e5cd`, run `37163400887`. Full original registry
bytes and the full three-execution 1.99 JSON packets are retained as fixture
controls. The original measurement and MSRV contracts remain untouched.

The forward ratchet route authenticates the complete two old registry objects,
not just matching compiler strings. Only the literal reviewed 1.98-to-1.99
transition may change methodology. Different versions/builds, backwards changes,
other methodology changes, changed old/current provenance, input identities,
inventories and numeric caps refuse. Ordinary same-methodology downward ratchets
remain unchanged; measurement comparison still requires exact methodology equality.

The CLI additionally requires source toolchain `1.99.0`, its exact compiler
identity and a measurement source commit equal to the actual checkout's Git HEAD.
Registry evidence provenance is the earlier actual producer; it is not required
to equal the future checkout. An archived packet may be authenticated in a test
with an explicit producer context, but local historical replay never grants
current-source acceptance.

The signed producer is a genuine merge of main `8f01ff9c320f4b474b410efe7d66baed462df7aa`
and PR source `e24c56aa60f9b6db4371e82317f72f89750cf591`, with tree
`ddec13710c1c1ba0ed0e9e924cb363b28caa4c44`. Its tree differs from the PR's tree.
Observed improvements belong to that whole producer; they are not measurements
isolating a single performance patch. Independent audit SHA256
`163bd91033c41ac62ce2476c64d40e66e7604866f1d715d6e34cfcb318322afd` authenticated
all 104 rows, all three identical executions and 390 raw Callgrind/allocator/input
packets against the original ceilings. It grants no future-head gate credit.

TODO: qualify the composed source with fresh Rust 1.99 source, native, corpus,
allocation and protected instruction Actions, including the three parser controls.
Pair this decision with the existing #6830/#8195 record during owner integration;
the common canonical decision file is owned centrally and is not edited here.
