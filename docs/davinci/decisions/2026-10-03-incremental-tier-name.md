# Incremental tier crate name (#7304)

On 2026-10-03, the maintainer selected `vize_incremental` to replace
`vize_resident`: the name describes the incremental tier used by long-lived
LSP, check-server and watch processes. This is an explicit crate-name decision,
not a level rename or a rename of other art-named products.

The crate remains `publish = false` and experimental. Its directory is
`crates/vize_incremental/`: it is currently a transitional product adapter
with normal dependencies on `vize_croquis` and `vize_relief`, so placing it in
`davinci/` would violate the program's dependency direction. The rename
does not remove those dependencies or give any legacy-backed path native credit.
`SfcSummary` remains owned by `vize_l2::summary`.

Replay `python3 tools/commands/davinci/rename-incremental-tier.py --phase moves`
on fresh main and commit the byte-exact moves first. Then run the same script
with `--phase references` to update package/workspace/lock identities, Rust
imports, workflow triggers and commands, tooling assertions and documentation.
Run `cargo update --offline -p vize_incremental` to restore Cargo's canonical
package ordering; the actual resolver locks zero changed dependency versions.
Format the changed Rust imports with `cargo fmt --package vize_incremental
--package vize_maestro` and changed Markdown tables with `vp fmt`. Regenerate
the existing Croquis and consumer-migration inventories through their
`tools/support/compat/davinci/{croquis-consumers,consumer-migration-surfaces}.mjs`
`--write` entry points. The import regrouping changes three LSP inventory
source-line coordinates; the natural scanner remains unchanged.
The key-capture fixture's historical build logs retain their original bytes.
The release publisher discovers manifests and continues excluding this
unpublished crate; no published package alias is introduced.

Acceptance requires the actual locked Cargo graph, incremental tests and their
seeded stale-cache detection, the salsa-only tier and one-shot CLI boundary,
generated source inventories, exact-head Actions and protected merge-queue
checks. The move commit has zero changed source bytes; all adaptation is
separate. #7304 closes only after actual merge. Native LSP state and remaining
legacy dependencies are still tracked by [#6872](https://github.com/ubugeeei-prod/vize/issues/6872).
