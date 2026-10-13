# Owned Corsa packages for external Canon consumers

Canon's default native feature must compile outside the Vize workspace.
The actual CSS downstream consumer on source `97f3db6d` instead resolved
registry Corsa 1.14.0, which lacks the checked `ProcessObserver`,
`ProcessReceipt` and `with_process_observer` APIs. Cargo patches in a
dependency's workspace root do not apply to an independent consumer.
Successful workspace compilation therefore did not qualify this surface.

## Package graph and public types

Keep the complete checked dependency graph under Vize's package names:

| Existing dependency key and library | Published package         |
| ----------------------------------- | ------------------------- |
| `corsa_core`                        | `vize_corsa_core`         |
| `corsa_jsonrpc`                     | `vize_corsa_jsonrpc`      |
| `corsa_client`                      | `vize_corsa_client`       |
| `corsa_lsp`                         | `vize_corsa_lsp`          |
| `corsa_orchestrator`                | `vize_corsa_orchestrator` |
| `corsa`                             | `vize_corsa`              |

These six packages inherit Vize's workspace version and use exact versioned
workspace aliases. All internal core, client, JSON-RPC, LSP and orchestrator
edges resolve to this graph. `corsa_runtime = =1.14.0` remains the independent
registry runtime. It has no dependency on the core error or command types.
Library names, exports, facade defaults and the `experimental-distributed`
feature forwarding remain unchanged. The two Corsa registry patches are
replaced by ordinary package dependencies.

This is a next-minor nominal Rust API migration. Values from separately
imported registry Corsa packages are different Rust types. A preserved
source-spelled import is insufficient to establish interoperability.
Consumers that directly share those values with Canon must use the checked
packages consistently, for example:

```toml
[dependencies]
# Select the same published Vize release for both dependencies.
vize_canon = "=0.443.0"
corsa = { package = "vize_corsa", version = "=0.443.0" }
```

The versions above describe the preparation source. They do not assert
that the new packages are already available on the public registry. The
release must document its actual next-minor version and migration before
publication. These packages carry `compatibility-preview` stability metadata.

## Source and packaging custody

Retain the complete authenticated published 1.14.0 members of all six
packages, including original manifests and VCS records. Existing checked
core/LSP production bodies remain unchanged. The other four source trees
are imported from their checksum-authenticated published archives. Their
MIT notice is copied byte-for-byte from the authenticated upstream core
license and retained in every packaged tree.

Cargo reserves `.cargo_vcs_info.json` and `Cargo.toml.orig`. Explicitly
exclude those original files from package input while keeping them in the
repository; Cargo generates the checked package's own metadata. Original
archive identity and resulting package identity are distinct authorities.
The archive VCS field is archive-authenticated metadata, rather than proof
that a separate Git checkout was reconstructed.

The supported `cargo update --workspace --offline` resolver keeps 579
package records: 570 unrelated whole records remain identical, six package
identities change, and three Vize consumer dependency lists use the checked
names. The facade's existing `tempfile` dev dependency and orchestrator's
existing optional `lsp-types` edge are now represented as workspace member
dependencies. The runtime's complete record remains unchanged.

Actual `cargo package --no-verify` generated all six archives. This proves
package assembly and normalization only. It does not prove consumer
compilation, native lifecycle correctness or installed public acceptance.

## Required delivery

Qualify an independent Cargo root without consumer patches, with Canon's
default native feature and both facade feature modes. Authenticate the
complete normalized packaged graph and compile the sealed package consumer
separately. Keep every original SDK source, raw native lifecycle/refusal,
owned-image, original400, differential and protected check mandatory.
Partial, foreign, mixed and unknown package source contexts must refuse
before recording benchmark results. These wiring changes make no speed or
broader real-project diagnostic claim.

Publish in dependency order: core, JSON-RPC, client, LSP, orchestrator,
facade, then Canon. New package ownership/trusted publishing and actual
registry propagation are release prerequisites. After publication, compile
the same consumer from the public registry. A sealed local graph or a
source review does not substitute for that acceptance. The original CSS
consumer failure and all failed source-custody packets remain retained.

The current source successor preserves the complete actual `2489128d` baseline and all 237 production pairs (121 original, 112 imported members and four license copies). Both complete caller bodies qualify before the private whole historical source view is available; the original 121/122/23/2 tables, seven native fixture bodies and 32 requests remain mandatory. The original 206 copied-source census remains distinct from the explicit complete 359-member owned graph capture. Splitting the four unchanged literal plan arrays keeps the formatted metadata helper under 350 rows; a missing or changed plan refuses before source projection. All seventeen source/caller/copied-source laws pass locally on committed source; their hosted runtime/package evidence remains pending. No source receipt grants compiler, native, performance, protected merge or registry acceptance.

The original NoDeep positive refuses current SDK source 4be because its raw first-parent search from the current head exceeds the unchanged 64-commit bound. Its complete four negative tails and 204 pair-side controls remain literal. The new source-only frame authenticates the actual current raw commit and 111 complete raw/physical cells; the two ASCII caller successors, one added caller frame and whole original-law transport inverse are explicit. The separate original five-law module executes on 113 authenticated files from actual main 248. The genuine historical 4be → fbdaa → cd7 → 248 commit bodies and eafe → 248 first-parent proof remain separate from current source qualification.

A real private commit with the equivalent full tree and sole parent 248 verifies the squash merge boundary without asserting that main descends from an unpublished branch commit. Eight real Git/filesystem refusals and all frame/part metadata mode, missing, symlink and body refusals retain unknown-source rejection. No local shared-root fetch occurs. This gives source evidence only; current repository checks, independent default/distributed Cargo consumers, native diagnostics, original400 and protected merge qualification remain mandatory. The SDK-specific CSS metadata context and any later NoUnused source class need their own finite full-body admission; no old green evidence is transferred.

The SDK source also needs a distinct CSS producer context. Its complete root/lock baseline is the actual reviewed six-package graph, while the original CSS root/lock remain separate. The existing official Rust metadata rewrite proves the entire current pair forward and backward on a byte-bound private snapshot. All other 235 source bodies and the same raw Git modes/blobs must still match the reviewed graph. The original CSS continuation inverses and official kernel stay unchanged. Four authored real-kernel laws cover current and future stable versions plus mixed metadata, unrelated bytes, foreign source, mode, missing, replacement and fake-original input refusals. They require Actions and grant no package/native acceptance.

The integrated current NoDeep module passes all five source laws locally (64.98 seconds), including its separate complete original replay and genuine squash boundary refusals. This timing is source-test execution only, not type-checker speed or native qualification.

Shallow hosted source checks need the pinned historical objects before raw verification. A shared literal-object helper reads existing commits without replacements or lazy fetching. Missing objects may use the unchanged exact-object loader only in the actual owned Linux Actions checkout, with canonical repository/origin and physical unshared Git storage. A local/shared/private missing object refuses before mutation. Six private laws preserve full repository and shallow bytes. Original 64-commit bounds, retained five-law timeout and all 111/113 body checks remain unchanged; successful hosted acquisition is still a fresh Actions condition.

## Source qualification repairs

Source `b2cae` failed before package acceptance: native vendor custody still expected the two old manifests, original400 required the old baseline identity, and Clippy applied Vize's allocation-style rules to retained upstream APIs. The source repair authenticates all 237 raw/physical pairs and the complete six-package vendor census before exposing either old manifest. Each of the six current manifests permits only `disallowed_types`, `disallowed_methods`, and `disallowed_macros`; every Rust source, published runtime, public alias and ordinary Clippy check remains required. All six policy bodies qualify atomically; mixed, missing, foreign or unknown bodies refuse. Package staging uses the actual current manifest, complete normalized dependency graph and current VCS identity, without historical package substitution.

The publisher needs six explicit vendor paths in its existing manifest resolver. Four additional published VCS JSON files retain their byte-exact formatter exclusion and mandatory source checksum. Ten required native package-input triggers extend the original ordered 34 positives; the original three exclusions, complete consumer closure, negative controls and full suites remain unchanged. The original 372-row authority file is preserved, and the formatted guard stays at 350 rows.

Actual main contexts are separately bound by complete raw commit/tree identities and every original before cell. The NoUnused integration needs both complete current 111-body source views; a common 112-body context authenticates them before either historical projection, both refusal sets run, and the unchanged 113-body five-law replay runs once. Historical parser and original 206-body fixture views remain test-only. No source constructor, historical replay, Clippy policy or metadata inverse grants native, timing, package or public-registry acceptance. Fresh complete Clippy, independent default/distributed consumers, the real CSS kernel, native lifecycle/refusal suites, original400 and protected merge remain required.

Committed source `86b57` ran 22 source laws: 21 passed and the retained old-manifest checksum assertion failed against the actual six-policy manifests. The corrected counterfactual retains the complete old six-manifest refusal, adds the complete current six-manifest refusal, and passes all three vendor source laws before the full successor report is accepted. No product checksum or refusal is weakened; these source results grant no hosted native or package acceptance.

The actual `a93e7814` main integration preserves all 237 before cells and the complete incoming canonical record. Both original 111-body source families, their refusal sets and the shared 113-body five-law replay execute through a separate complete 112-body context. Its complete parsed metadata is immutable: the earlier mutable view admitted a missing source while claiming completeness; the new authority law reproduces that refusal and passes. The unchanged current-frame suite passes all five source laws, zero skipped.

The historical parser tests retain their complete original 117-body table, four raw modules and 206-body closure. Exact incoming `pnpm-lock.yaml` is a third named after-frame successor; the whole current lock remains visible through the actual reader, and only the independently authenticated old whole lock is loaned to historical tests after the 119-frame, 237-source and two-caller checks. A new test-only ancestry assertion is replaced by raw commit/tree and complete body checks, with explicit zero ancestry, squash, dependency and toolchain credit. Original runtime and NoDeep ancestry refusals remain unchanged. All 50 current/historical source laws pass on the integrated source, zero skipped. Hosted acquisition, native lifecycle, original400, default/distributed package consumers, full Clippy and protected merge still require fresh Actions.

Source `437c7f8` preserves the first complete hosted current-family failures. Check-JS refused one unformatted passive historical JSON capsule; move its complete original bytes into the existing fixture-data location in a separate move-only commit, then format it and update only exact data pins. No line-cap or formatter policy changes apply. Current Rust 1.99 Clippy additionally rejects four unnecessary closure borrows in the owned Corsa handle parser. Remove only those four `&` bytes, authenticate the complete original/current body and four-clause inverse, and stage the actual corrected source in package archives; standard lint and all original 237/95 source/provenance gates remain required.

The original400 admission completed the actual 237-body graph and seven fixture qualification but refused the incoming unchanged `pnpm-lock.yaml` source cell. Both actual before/after locks are the same complete 950,541-byte body; the original preserved-source table names the retained 946,627-byte body. Authenticate both complete actual cells before the named source counterfactual, leave the entire original 121/122/23/2 qualifier untouched, and record both identities. Runtime builds and lock receipts continue using the actual current body on both sides. This source transition grants no dependency, toolchain, runtime or performance credit. Current consumer, full Rust/native/original400 and protected qualification must rerun; prior failed jobs remain failed.

The first complete committed `d0ea028` local source run retained 28 passing and six failing laws. Three failures are bounded current-source bindings: the positive census must name the six manifests plus exactly the corrected handle file, the complete Clippy metadata checksum must name its reviewed successor, and the CSS checker must bind its reviewed current custody helper. Each test successor recovers the complete previous body by its declared literal inverse; every other production identity and refusal remains required. The two focused actual237/mixed-manifest controls and all four CSS context laws now pass, including all 32 source refusals. The remaining three official metadata-kernel laws reported `rust-script` ENOENT in that local invocation; their complete failures remain retained, and actual hosted kernel execution stays mandatory. No native, package, original400 or protected success is inferred from these source tests.

The exact `0d2ec40b` hosted source failed before native/package qualification: the two actual original400/native jobs refused unrecognized main `db258f5e`, Rust 1.99 rejected a single-element benchmark loop, and Check-JS required void test registrations. Replace only that loop with its unchanged single body and bind the independently authenticated raw main contexts `7d6e0350` and `3601d681`; all 237 before/after cells stay mandatory. The NoMutating integration preserves the current main caller/helper bodies and authenticates a separate complete SDK caller frame before the whole historical replay. Seven new source-only helper/law files extend the immutable original117/119 authority to current126; every original runtime, refusal, lock and package check remains literal. The six complete current join source laws pass, zero skipped; fresh hosted default/distributed compilation, metadata kernel, native lifecycle, original400 and protected qualification remain pending on the published successor. These source repairs give no semantic, timing or release credit.
