# Public npm installation custody

This producer prepares the installed-package authority used by the prospective
original LSP replays in [#6239](https://github.com/ubugeeei-prod/vize/issues/6239)
and the CI work in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
It replaces the private collector's AST extraction with ordinary, readable
repository modules. Its shared schema is
[authority-schema.ts](../../tools/support/release/public_install/authority-schema.ts).
The existing `vize-public-registry-install-v1` required fields, including
`registry.provenanceSourceH` and `registry.provenanceR`, retain their literal
meaning and spelling.

This change grants no public installation, native execution, Corsa execution or
release credit. The retired v0.436 candidate remains unpublished. The next
v0.437 candidate remains held with C/H/R unassigned. The actual
[#8284 guard](https://github.com/ubugeeei-prod/vize/pull/8284),
and [#8290 BOM fix](https://github.com/ubugeeei-prod/vize/pull/8290) have merged,
but demonstrated ELOOP cause resolution and the complete original corpus still
must qualify before the release cut is assigned. The original
public runners remain prospective.

## Boundaries

The official entry is the non-Node
[Rust command](../../tools/commands/release/npm/collect-public-install.rs). It
rejects nonempty `NODE_OPTIONS`, `VIZE_PREFER_WORKSPACE_BINDING`,
`NAPI_RS_NATIVE_LIBRARY_PATH`, `NAPI_RS_FORCE_WASI`,
`VIZE_ALLOW_NATIVE_VERSION_MISMATCH`, `CORSA_PATH`, `CORSA_EXECUTABLE`,
`TSGO_PATH` and `TSGO_EXECUTABLE` before launching Python or any Node process.
Existing custody configurations are also rejected. Python is started with `-I`
and its owned sibling modules are compiled directly from reviewed source bytes;
matching timestamp/size `.pyc` caches cannot substitute executed code. The Python producer and
the hook independently reject their initial overrides. Nonempty overrides are
never silently removed to manufacture evidence.

The collector requires full C/H, a stable 0.x minor tag and a positive exact
Release run R. H must be exactly one commit with the sole parent C, read from
raw commit headers. All source reads disable Git replacement objects, and
neither replacement refs nor legacy grafts can substitute source authority. Frozen H
must declare the requested public CLI, plugin and native versions. Since
platform npm manifests are generated during release, the frozen H native
catalog's Darwin ARM64 version and native target declaration bind that platform
source authority. The CLI's real `@vizejs/native` workspace edge resolves to
the requested H native version; the public CLI native dependency and exact
Corsa optional dependency must equal H's real native and Corsa catalog
declarations. These source files are hashed separately in the receipt.

The caller must separately authenticate the actual public annotated tag,
GitHub release, Release run and publication channels. The collector does not
prove the tag's signature, GitHub publication, Marketplace or Open VSX status.
It does not install packages or discover a candidate from current main, latest
tags or npm dist-tags.

## Prospective invocation

Use a reviewed checkout. First obtain the collector snapshot for independent
review; this command reads owned tool files and executes no Vize/native/Corsa:

```sh
rust-script tools/commands/release/npm/collect-public-install.rs --print-collector-authority
```

The snapshot SHA256 hashes the sorted relative file path, a NUL, its SHA256 and
a newline for every fixed producer file. The list includes the Rust entry,
all five Python modules, the hook and the schema. A reviewer pins this value
before any later collection. The receipt's `collectorAuthority` identifies
these reviewed tool bytes; it does not claim those tools are included in H.

Only after the release hold clears, a genuine independently authenticated
public source/run exists and the caller has made a fresh npm installation:

```sh
rust-script tools/commands/release/npm/collect-public-install.rs \
  --root /absolute/canonical/source-checkout \
  --cut FULL_C --head FULL_H --tag v0.MINOR.0 --run EXACT_R \
  --install-root /absolute/canonical/fresh-install \
  --node /absolute/canonical/node \
  --output /absolute/canonical/receipts/unique-install.json \
  --collector-sha256 REVIEWED_COLLECTOR_SHA256
```

The fresh consumer must be private, outside the source workspace, and declare
exact versions of `vize`, `oxlint-plugin-vize`, `@vizejs/native` and
`@vizejs/native-darwin-arm64`. Its npm lock must be v3 and repeat those exact
dependencies. The CLI must declare an exact public optional
`@typescript/typescript-darwin-arm64` version of TypeScript 7 or newer. That
package's `lib/tsc` must be the executable public launcher. There is no source
executable, workspace binding, external Corsa or fallback route.

## What the receipt establishes

For all four Vize packages and the declared Corsa package, the producer checks
exact manifest/lock/public registry name and version, resolved URL and SHA512
SRI. It downloads the public archives, verifies SRI and compares every installed
file byte and the entire file set. This includes JavaScript routing,
`native-targets.js`, native bytes and the Corsa launcher. Wrong archive roots,
traversal, links, duplicate files, missing files and extra installed files fail.
The complete file digest manifest is retained and rechecked after the probe.

For Vize packages, `provenancePayloadBinding` checks the decoded npm SLSA v1
payload's exact H, release branch/workflow/repository, R invocation and SHA512
subject. It rejects missing or duplicate qualifying payloads. This is metadata
and payload binding; **cryptographic signature verification is not performed**.
The legacy `provenanceSourceH`/`provenanceR` fields record that bounded payload
binding, and do not claim authenticated certificate/DSSE verification. Corsa's
receipt is third-party public archive custody, not Vize source/run provenance.

Prospective cryptographic verification is separate. The
[official npm verification guide](https://docs.npmjs.com/verifying-registry-signatures/)
and [audit command](https://docs.npmjs.com/cli/v12/commands/npm-audit/)
describe `npm audit signatures --json --include-attestations`. Require retained,
per-package verified signed-bundle coverage for the same four exact Vize versions,
H/R/workflow/ref and SHA512 subject/SRI. Exit zero alone permits missing
provenance and does not prove that coverage. The reviewed
[npm 12.1 implementation](https://github.com/npm/cli/blob/v12.1.0/lib/commands/audit.js)
loads the actual installed tree; a lock-only consumer cannot supply this coverage.
Full installed-file equality remains a separate check. This producer has performed
none of that cryptographic verification.

Only the genuine Darwin ARM64 host is supported. The absolute Node executable
and CLI wrapper are checked, and the wrapper must equal frozen H. The Node hook
observes the original `.node` loader with the same `this` and arguments, and
preserves its loaded exports, thrown exception and return value. Resolution or
an attempted load earns no success credit. Exactly one successful genuine
expected loader return with matching path/digest and the CLI-selected public
bundled Corsa route is required. The process PID, complete journal, zero exit,
exact version stdout and empty stderr are checked. Node, lock, producer and all
installed package bytes are rechecked before the final receipt is written.

Receipt, payload manifest and native journal names must all be new, canonical
paths outside the installation. Exclusive creation rejects overwrites, including
dangling symlinks. A failed run can leave a payload manifest and failed journal;
preserve these observations and use a new output name. Such files are never a
successful authority receipt.

## Inert rejection qualification

```sh
node --test tests/tooling/public-install-custody.test.ts
python3 -I tests/tooling/support/public-install/controls.py
python3 -I tests/tooling/support/public-install/source_controls.py
rust-script tools/commands/ci/verify-tool-layout.rs
rust-script tools/commands/ci/source-file-lengths.rs --check --base-ref origin/main
node tools/benchmarks/scripts/test-inventory.mjs --json /tmp/public-install-test-inventory.json
```

The tests use inert archive files and parser packets, synthetic failed journal
controls and text pretending only to be a `.node` filename. The real original
Node loader rejects that text, and its journal must contain `failed` with no
`returned` event. Additional controls reject tampered JS/native/Corsa, archive
file-set changes, genuine committed H manifest shapes, wrong H versions/edges/catalogs,
replacement/graft substitution, wrong source/run/workflow/subject,
ambient overrides before startup, path escapes and overwrites. They include a
matching same-size/same-mtime stale Python cache that ordinary import
really executes, while the official source-only producer ignores it. Network
access is denied throughout the inert parser controls. These tests do
not simulate a successful native load and establish no public product credit.
The ordinary tooling selector discovers the test in its existing broad PR and
full merge suites; no gate or instruction budget is relaxed.

The first draft's [tooling shard 3](https://github.com/ubugeeei-prod/vize/actions/runs/37750957115/job/113223849641)
failed because its wrong-version fixture assumed the Actions checkout had one
raw parent; the PR checkout was a two-parent merge. That law now copies the
committed checkout's exact manifest/catalog bytes using raw Git reads into an
owned scratch repository, commits an actual C-to-single-parent-H relation, and
requires wrong-version refusal before the fake Node or receipt output. A
separate actual two-parent scratch checkout must refuse before any probe.
This copied committed shape is inert control data, not a publication H. No
parent objects are fetched or inferred from shallow checkout history; the
production requirement for complete raw C/H and H's sole parent C is unchanged.

Remaining work: actual held-source qualification and owner-authenticated public
publication, cryptographic npm provenance verification, genuine fresh public
installation and complete installed original-case replay. None is replaced by
these producer preparation laws.
