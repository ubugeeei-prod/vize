# Checked Corsa package consumer

Run `python3 tools/support/compat/github/checked-corsa-package-consumer.py` from a
clean checkout with Python 3.11+, `SOURCE_SHA` set to that checkout's commit,
`CARGO_TARGET_DIR` set to the native CI worker's absolute target directory, and
the workspace dependencies already fetched with `cargo fetch --locked`.

The harness substitutes the current workspace version into this manifest and
creates independent workspaces outside the source checkout. It builds and runs
the fixture with default features and `experimental-distributed` through two
providers:

- The six checked source packages through ordinary direct path dependencies.
- Cargo-generated `.crate` archives through an offline directory source, with
  normalized version-only dependencies and no consumer patches.

Both providers use the native worker's `ci` profile (`dev`, no debug information
or incremental compilation, 256 codegen units) and shared target directory to
reuse dependency builds. Archives must retain every source member and the exact
`LICENSE` bytes; only Cargo's generated manifest/VCS files and the explicitly
omitted package lock are exempt from source-byte comparison.

Generated consumers have distinct package names. Before each build, the harness
cleans only that generated root package and requires a `fresh: false` compiler
artifact identifying its exact source file. Provider/dependency artifacts remain
shared. This prevents Cargo from treating a sibling consumer's earlier output
as proof that a different source compiled, including the default-feature refusal.

The fixture checks shared errors, process receipts and command ownership across
all six libraries. It also preserves current client source-file/content-mapper/
semantic-query APIs and checks local/default versus experimental orchestration.
A separate default consumer must reject an import of the gated `RaftConfig`.

Archive generation uses `cargo package --no-verify --exclude-lockfile` because
the owned registry names are not yet published. Compilation proof comes from
the subsequent archive-consumer builds and tests. No publishing occurs, and a
successful receipt explicitly records `publicRegistryAcceptance: false`.

An independent Canon-only root must compile its default native public APIs.
CLI default, minimal, glyph, maestro and all-feature configurations also compile
through separate `vize` library consumers. They reference the public CLI
entrypoint, native Canon exports, and enabled glyph/maestro/profiling exports.
Their metadata must preserve the selected features without workspace-wide
unification. These builds share the CI dependency cache and avoid repeated CLI
binary linking; CLI runtime behavior is qualified by the surrounding native
acceptance workflow. The receipt records each actual root compilation before
setting `canonDefaultNativeCompiled` and `cliVariantsCompiled` to true.
It then runs the one exact Musea story-discovery regression with CLI default
features disabled and records `minimalStoryCollectionTestPassed` only after
confirming that the named test executed and passed.

`--output` selects a new evidence directory. Preserve its top-level `*.json`,
`*.stdout`, `*.stderr` and `package-target/package/*.crate` files as CI artifacts;
the other directories contain disposable copied/vendored dependencies. A
`receipt.json` is written only after every required command and assertion passes.

Fast harness refusal tests require no Rust builds:

```sh
python3 tools/support/compat/github/checked-corsa-package-consumer-test.py
```

## Separate minor publication and public acceptance

The release coordinator must first clear the existing shipping hold and publish
the six owned packages in dependency order (core, JSON-RPC, client, LSP,
orchestrator, facade), followed by Canon and its Vize dependencies in the new
minor. This helper does not publish, register names, edit a catalog, or inspect
ownership. Keep the existing `corsa_runtime` 1.14.2 registry package.

After those exact versions are public, run from the clean checkout of the
immutable release source. Supply the explicit version and source SHA rather
than inferring them from current registry state:

```sh
python3 tools/support/compat/github/checked-corsa-public-consumer.py \
  --version "${VIZE_RELEASE_VERSION:?exact published version required}" \
  --source-sha "${VIZE_RELEASE_SOURCE_SHA:?immutable release commit required}" \
  --output "${RUNNER_TEMP:?}/checked-corsa-public-${VIZE_RELEASE_VERSION}"
```

The output must be new and outside the repository, without ancestor Cargo
configuration. The helper creates fresh Cargo home/target directories, strips
Cargo source/compiler overrides, and uses version-only crates.io dependencies.
It builds and tests default/distributed six-package consumers, then compiles an
independent published Canon consumer through its default native feature. Every
owned provider/version, the single runtime, complete archive membership,
compiled source bytes, registry lock checksum, exact release VCS SHA and MIT
notice must agree. Preserve the entire `evidence` directory plus the seven
downloaded `.crate` files identified by its receipt.

Publishing strips development dependencies with `--allow-dirty`; the public
archive evidence preserves the actual normalized/original manifests and VCS
receipt, including truthful `dirty: true`. Canon now includes the complete root
MIT notice, addressing its prior missing license file. CLI is `publish = false`:
its five feature variants remain source acceptance, and the public receipt
does not claim they were installed from crates.io. No public acceptance is
established until this optional helper completes successfully after publication.

Offline refusal controls make no registry requests:

```sh
python3 tools/support/compat/github/checked-corsa-public-consumer-test.py
```
