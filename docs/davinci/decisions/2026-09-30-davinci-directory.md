# Davinci directory boundary (2026-09-30)

Tracking: [#6826](https://github.com/ubugeeei-prod/vize/issues/6826),
[#6831](https://github.com/ubugeeei-prod/vize/issues/6831).

The maintainer requested one PR separating Davinci implementation from legacy
products. `davinci/` owns L0–L4, both conversions, the remaining substrate and
derive crate, extension contracts/hosting, the guest SDK and MoonBit dialect.
Package identities and published features retain their current names.

Carton is legacy and remains in `crates/`. Its shared foundation implementation
moves unchanged into L0; Carton re-exports it and forwards its extension and
lint-glob features. This inverts the old L0→Carton edge without duplicating types
or changing the legacy public API. The accepted std-bound foundation stays
std-bound; this move does not complete the no-std isolation work in #6834.
The [foundation storage inventory](../plan/foundation-storage-bridges.json)
records each pre-existing std storage use now inside L0. The stage gate compares
that exact list, including duplicate uses, so new std uses and opaque imports
still fail. Native artifact storage inventory and instruction ceilings are
unchanged; no directory or module is exempted wholesale.

MoonBit's experimental SFC entry point uses the existing native L1 Vue container
instead of Croquis. It refuses container errors and retains authored spans.
The pinned projection and diagnostic fixtures verify that switch. Legacy product
SFC parsing and Resident stay in `crates/`; this does not close the product
container migration (#6837) or promote unfinished dialect work to completion.

`crates/` may depend on `davinci/`. Normal and build dependencies in the reverse
direction are forbidden, including renamed, optional, target-specific and
transitive workspace declarations. Dev-only differential oracles are allowed.
The Actions metadata gate checks every package under `davinci/`, including
helpers, without a directory allowlist.

The last L1→Relief edge is removed by moving Namespace, compiler error codes
and their methods, and stage identities into L0. Relief and the substrate
re-export their original public paths. Legacy ASTs, parser routes, diagnostic
text, wire values and captured oracle bytes retain their existing behavior.

Four benchmark fixture digests change because those probes identify their own
source file, whose location labels changed. Three identical measurements from
[Actions](https://github.com/ubugeeei-prod/vize/actions/runs/36679539802) stayed
within every existing instruction ceiling. Only these source digests are
refreshed; numeric ceilings and measurement methodology are unchanged. The final foundation move passed PR Actions and the protected merge queue in
[#7271](https://github.com/ubugeeei-prod/vize/pull/7271), merged as
`507ac3cc683c044fc136016cd4bfb7d3bde44965`. The fresh-main
[dependency job](https://github.com/ubugeeei-prod/vize/actions/runs/36697179934/job/109827822993)
passed with an empty allowlist and no normal/build reverse paths, satisfying
[#6831](https://github.com/ubugeeei-prod/vize/issues/6831).

Replay directory and ownership moves on fresh main:

```sh
node tools/support/compat/levels/move-davinci-directory.mjs --move-only
# Commit directory moves without content edits.
node tools/support/compat/levels/move-davinci-directory.mjs --foundation-moves-only
# Commit foundation file moves without content edits.
node tools/support/compat/levels/move-davinci-directory.mjs --foundation
node tools/support/compat/levels/move-davinci-directory.mjs --shared-moves-only
# Commit stage/code file moves without content edits.
node tools/support/compat/levels/move-davinci-directory.mjs --shared-types
node tools/support/compat/levels/move-davinci-directory.mjs --references
cargo fmt --all
```

Integration extracts existing namespace/code definitions, retains their methods
and public re-exports, and connects the dialect to L1. On a conflict, replay the
scripts on current main and regenerate inventories. CI discovery, source
planning, caches, publishing, assertion scans and inventories cover both roots.
