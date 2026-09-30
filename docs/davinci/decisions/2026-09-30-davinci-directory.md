# Davinci directory boundary (2026-09-30)

Tracking: [#6826](https://github.com/ubugeeei-prod/vize/issues/6826),
[#6831](https://github.com/ubugeeei-prod/vize/issues/6831).

The maintainer requested a single PR separating Davinci implementation from
legacy products. `davinci/` owns L0–L4, both conversion crates, the remaining
substrate and derive crate, extension contracts/hosting, and the shared Carton
foundation. Package identities and published features keep their current names.
Carton remains a shared foundation until its storage carve-out (#6834); placing
it below this boundary avoids an L0 dependency back into `crates/`.

Legacy compiler/product code stays in `crates/`. Resident and MoonBit's current
SFC adapter remain there because they consume the legacy Croquis parser; their
native replacement still belongs to the existing roadmap. The independently
published guest SDK also remains a product in `crates/`. Native MoonBit level
modules live in the moved level crates. This split does not credit adapters as
native implementations or close those roadmap issues.

`crates/` may depend on `davinci/`. Normal and build dependencies in the reverse
direction are forbidden, including renamed, optional, target-specific and
transitive workspace declarations. Dev-only differential oracles are allowed
under the existing compatibility policy. The Actions metadata gate checks every
package under `davinci/`, including shared helpers, with no directory allowlist.

The last L1→Relief edge is removed by moving the unchanged Namespace, compiler
error-code methods and stage identities into the shared foundation. Relief and
the substrate re-export their original public paths. ASTs, parser routes,
diagnostic text, wire values, captured oracle bytes and instruction ceilings
keep their existing behavior. Full Actions and merge-queue suites validate the
relocation and public API compatibility before merge.

Replay directory moves on fresh main with:

```sh
node tools/support/compat/levels/move-davinci-directory.mjs --move-only
# Commit the moves without content edits.
node tools/support/compat/levels/move-davinci-directory.mjs --references
node tools/support/compat/levels/move-davinci-directory.mjs --shared-moves-only
# Commit the shared file moves without content edits.
node tools/support/compat/levels/move-davinci-directory.mjs --shared-types
cargo fmt --all
```

The separate shared-type ownership commit moves stage/code files unchanged;
integration extracts the existing code/namespace definitions and retains all
their inherent methods and legacy re-exports. On a conflict, replay the scripts
on current main and regenerate inventories. CI discovery, source planning, caches,
publishing, assertion scans and generated inventories cover both directories.
