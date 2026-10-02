# Tag-cache test lint repair (#6883)

The full source Check on L1 Params head
`63337311751953cb7e174ac485e76f541888846a` failed before its tests in
[job 110577029906](https://github.com/ubugeeei-prod/vize/actions/runs/36924096700/job/110577029906).
Clippy reports `useless_conversion` at
`crates/vize_maestro/src/server/state/global_tag_names.rs:248`: the oversized
open-document fixture converts `std::string::String` to the same type. The
test originated in merged component-tag completion #7385 and is unchanged
by the L1 Params implementation.

Remove only `.into()` after `" ".repeat(4 * 1024 * 1024 + 1)`. The input
remains exactly 4,194,305 ASCII spaces, and the cached disk/unsaved/reopened
declaration sequence plus both complete empty-result assertions are retained.
Production tag discovery, cache policy and the 4 MiB limit are unchanged.
No lint waiver, warning-policy change, performance-budget change or additional
product execution is introduced.

Exact-head full Actions must run the existing Maestro test and strict Clippy
gate before the protected queue's terminal merge. This test-only repair gives
no native, fixture-history or whole-fix credit. The seven-session/ten-response
LSP history Stack remains a separate prepared change; #6883 stays open.

Local execution of this unchanged cache test exposes a separate path-identity
TODO: default macOS `/var/folders` temporary paths differ from the disk scan's
canonical `/private/var/folders` paths, so the unsaved declaration can coexist
with its disk names. The same actual test passes with canonical
`TMPDIR=/private/tmp`; this establishes the platform condition, not a fix.
Retain and reconcile open-document URI identity under filesystem symlinks in a
separate behavior change with a corpus fixture. No production path policy or
assertion is changed here, and Linux Actions remains required.
