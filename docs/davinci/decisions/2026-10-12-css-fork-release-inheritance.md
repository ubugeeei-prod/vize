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

Future release metadata also changes the root manifest and lockfile. The
read-only `pr.rs verify-producer-metadata <frozen-directory>` CLI reuses the
existing official `metadata::rewrite` for both complete post-inheritance frozen
files. Current files must equal its entire forward output and reverse to the
entire frozen bytes. Versions must be canonically spelled stable `0.N.0`, equal
to the frozen baseline or a future minor; root and lock must advance together.
The official `verify_delta` remains unchanged. This is source authentication,
not release identity, registry publication or consumer acceptance.

The source-after guard retains all nine original references and every raw current
hash. Only after this whole-file official proof may root and lock use the frozen
post-inheritance bodies; the exact inheritance inverses still recover the
original before authority. All other seven producer sources remain literal.
The original archived authenticator still refuses the current root body. Whole
baseline fixtures and CLI/helper sources are separately pinned and archived.
The meaningful Actions law checks entire `0.443.0` bodies, official-transform
samples `0.448.0` and `0.65535.0`, and partial/mixed/stale/nonversion/invalid-version
refusals without changing either input. These samples are not selected or
published releases. Existing rust-script setup is reused; no capture campaign,
version stripping, arbitrary digest admission or extra release mutation exists.

Fresh source/after/protected qualification remains required. Actual prior
source guard failures and the separate MoonBit HTTP 403 remain retained; local
source review and checksum/format checks supply no Rust compilation, engine,
native, release or public acceptance. The sole release operator still verifies
the chosen official metadata cut and installed consumers before closing issues.
