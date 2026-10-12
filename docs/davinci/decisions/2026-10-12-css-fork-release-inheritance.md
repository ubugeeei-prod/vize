# Keep the checked CSS forks aligned during releases

Issue: [#3864](https://github.com/ubugeeei-prod/vize/issues/3864).

[#8709](https://github.com/ubugeeei-prod/vize/pull/8709) actually merged as
`cb8847872754019d51da42d290643c0f675040eb` after all five protected workflows
passed. Its authenticated source controls retain six original compilation
successes, three expected nominal-type failures, and three successful
namespace-only migrations. All 100 + 4 instruction workloads pass their unchanged
ceilings. That evidence qualifies the source; fork publication and installed
consumer acceptance are still required.

The next release would fail before publication: LightningCSS's effective
manifest fixes `vize_parcel_sourcemap` to `=0.443.0`, while both fork package
versions inherit the workspace version. The existing Moon release preparer
updates the root workspace pins before `cargo update --workspace`; it does not
rewrite the nested fork manifest. A minimal two-package Cargo metadata control
actually rejects the stale requirement after changing the workspace version,
with exit 101; the inherited dependency resolves with exit 0. This control builds
and executes no CSS engine or archive code.

Declare the exact Parcel package alias, registry version and local path in
`[workspace.dependencies]`. LightningCSS inherits it with `workspace = true`
and retains `optional = true` and `features = ["json"]`. The existing root
version updater now owns both fork requirements. No release mutation path is
added; publication order remains Parcel before LightningCSS before their Vize
consumers. Bundler, source-map and default feature declarations remain exact.

[Cargo's multiple-location rules](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#multiple-locations)
require the local package version to satisfy the declared registry requirement.
The source-only regression runs the real Moon manifest rewrite, checks its
complete output, resolves the two minimal package manifests offline, and then
reinstates the historical stale pin to require a genuine Cargo refusal. It also
checks the complete effective dependency and existing feature declarations.

The core engine, checked rkyv 0.8 decoder, original upstream sources, frozen
consumer inputs, finite expectations, Cargo lockfile and selected public AST
namespace are unchanged. This correction supplies no native-binding, registry,
published no-patch Cargo, release or performance acceptance. The next minor must
still document `vize_patina::css_ast`, publish both forks in the existing order,
authenticate the installed dependency/feature graph and qualify its consumers.
#3864 and #3295 remain open. The earlier third-party release remains separate.

Fresh ordinary and protected checks must qualify this independent source
correction before the next release is prepared. The earlier merge and local
metadata controls do not transfer their acceptance to the new proposal.

The first source `524704fe` reached genuine
[Actions 38227420772](https://github.com/ubugeeei-prod/vize/actions/runs/38227420772).
Both engine profiles correctly refused the changed root manifest under the
existing source-after producer guard; no 63-slot after capture ran. The original
failed logs stay retained. Separately, downstream types and Nuxt stopped at the
pinned MoonBit download's HTTP 403 before their commands ran. Neither failure is
a waived check, an engine regression claim or a successful qualification.

The source-after guard now admits only the two reviewed entire current manifest
bodies, reverses their exact declared alias/inheritance edits and verifies the
complete original manifest hashes from the frozen before packet. All other seven
producer inputs still require their original entire bytes. Receipts record both
current and original hashes, never rewrite the actual source identity, and the
existing artifact retains the added authenticator's whole source. Two complete
original manifest fixtures independently qualify the inverses; changed bytes,
wrong path/authority and self-consistent current-reference substitutions refuse.
The original two source laws remain, with one additional law exercising the real
source-after authenticator and every nine-input identity.

Future release metadata also changes the root manifest and lockfile. Their
original authorities remain strict: the sole release operator must authenticate
the exact chosen target's metadata successor. No general version normalization
or dynamic source exception is added. Fresh source/after/protected and published
consumer qualification remain required.
