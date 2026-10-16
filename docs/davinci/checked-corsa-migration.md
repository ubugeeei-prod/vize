# Checked Corsa compatibility graph

Vize needs observable native process termination and a snapshot owner that keeps
configuration, sources and protocol handles together. The stable Corsa 1.14.2
binding does not expose the process observer required for those checks. The six
packages under `vendor/corsa*` provide a compatibility preview with the original
Rust library names and existing default/distributed features.

This migration does not claim that upstream Corsa, registry consumers, release
artifacts or all real applications have passed. Source checks, native behavior,
independent package consumers, controlled performance and public release
acceptance are separate results.

## Source and dependency identity

The starting lockfile uses Corsa, LSP and orchestrator 1.14.0, with core, JSON-RPC,
client and runtime 1.14.2. Preserve that mixed graph: replacing every package with
1.14.0 would remove current client APIs. Runtime remains the unmodified registry
`corsa_runtime` 1.14.2. All six owned packages share the Vize workspace version and
keep the `corsa*` library names. Their package names become `vize_corsa*`.

`vendor/corsa-source.tsv` records every imported member, published archive SHA256,
upstream version, exact member size and SHA256, and whether it is unchanged
upstream source, a checked overlay, an owned manifest or a retained license. The
source verifier checks the complete inventory and six full MIT notices. It does
not constrain unrelated Vize source or require a finite Git history.

The facade's upstream allocation policy originally refers to its monorepo. Its
complete original test remains in `tests/data/upstream_policy.rs.txt`. The active
test checks all 13 imported fast-module paths and the actual Cargo-resolved
registry runtime's broadcast module. The five upstream `corsa_ref` CLI-tool
paths are outside this six-package dependency graph and are not shipped; their
historical policy remains in that original file. The original native baseline
JSON and optional upstream dataset/backend conditions remain unchanged; only
its packaged fixture path is relative to the actual facade manifest. Separate
required Vize native laws provide actual product acceptance.

The overlays add owned-process observations, acknowledged shutdown and bounded
reaping. Only an authenticated native 7.0.2 executable and complete initialize
contract select shutdown followed by stdin EOF. Foreign servers retain standard
shutdown/exit. No stderr bytes, including the historical EOF line, are discarded.
A clean result requires one completed successful terminal receipt and an
acknowledged graceful shutdown. Startup failure and discard retain raw evidence.
Final LSP shutdown freezes cache admission and waits for the actual queued client
close, including unfinished startup and owners removed by configuration changes.
Typed cancellation handles are published before synchronous initialization, so
finalization can terminate and reap the exact owned child without acquiring a
handshake lock. The same finalizer runs after Exit or stdin EOF. An empty cache
slot or a timeout cannot produce a successful shutdown response.

Ordinary configuration retirement stays asynchronous. Context removal transfers
its native owner to a shared retirement ledger before releasing the registry
lock; final shutdown joins current and retired owners concurrently. A past
protocol failure remains visible with its raw status, signal and error, while
complete terminal custody allows a recovered healthy server to close. Missing
reap acknowledgments retain cleanup handles and fail final shutdown. The first
failure of the active final shutdown attempt remains sticky.

Non-owning observations survive failed constructors and process-owner Drop.
Each receipt snapshot copies terminal and stderr evidence, and completed records
are pruned only after their exact snapshot is handed to a typed caller or
offered to the diagnostic event. Socket attachments own no separate child; `attachedBindings` records
the bindings retained at cleanup, rather than a history of every earlier socket
attachment.
Ordinary recovery offers completed receipts to the existing INFO diagnostic
event before pruning them. Disabled logging follows normal log semantics and
does not promise durable terminal history; it cannot change metadata growth or
process lifetime. Live and incomplete owners always remain tracked, regardless
of logging configuration.

Package identities are nominal Rust types: applications exchanging Corsa types
must consistently use the owned graph. Mixing upstream and owned providers is
not a supported migration. No consumer-side Cargo patch is required.

## Independent acceptance

The native Actions retain existing snapshot, projection, syntax/semantic and LSP
laws. They additionally run the frozen full diagnostic vector and repaired input
against the actual native process; the batch receipt borrows the same retained
snapshot attachment used for diagnostics. CLI controls retain unexpected stderr,
non-UTF8 bytes, nonzero status, signal termination and forced timeout evidence.
Concurrent rename sessions verify their actual child termination. A small real
stdio test separately checks shutdown/Exit, Exit with stdin still open, and EOF;
its proof requires the observed native PID/birth to disappear, and never uses a
kill fallback to obtain a passing result.
Additional actual native cases hold an old owner's response while a new owner
serves requests, and terminate an old owner before a healthy replacement closes.
Their packets retain exact API/editor identities, terminal results and the final
shutdown response. Controlled handshake tests also pause real context removal
and watcher replacement before deferred retirement, including the primary root;
the final acknowledgment requires the recorded child birth to disappear first.

`checked-corsa-package-consumer.py` assembles all six normalized Cargo archives
and checks complete source membership and licenses. Archive assembly with
`--no-verify` is explicitly not compilation acceptance. Separate independent
consumers build and test both path providers and sealed archive providers with
default and distributed features. They do not patch dependencies. An independent Canon-only
consumer compiles its default native public API. Five separate CLI library
consumers compile default, minimal, Glyph, Maestro and all features, isolated
from workspace-wide feature unification. Each root must actually compile from
its exact source; only that tiny generated root is cleaned between modes, while
provider and dependency compilation share the worker cache. All compilation uses the worker's shared
`ci` target. The archive consumer is offline and is not a public-registry test.

The original 400-module performance source-pair mode retains both full
manifests/locks and metadata. It admits only the structural six-provider
migration, preserves unrelated dependencies and the actual runtime, and demands
fresh compilation of all six provider libraries on each side. It keeps the
existing workload packets, invalidation controls and numerical ceilings. A source
pair does not replace public release qualification. Earlier baseline/candidate
runs exposed a final direct-child zombie after server exit. Those complete
packets and observations remain evidence, while the candidate and installed
public side now additionally require actual disappearance or a different birth
identity. A same-birth zombie cannot satisfy final owner reaping.

## Publication plan and current prerequisites

Read-only crates.io queries on 2026-10-16 returned HTTP 404 for all six owned
package names. There are no published versions or owners to inspect for those names. The
existing `vize_canon` owner query identifies the user `ubugeeei`; this does not
establish a future token's permission to create the new namespaces. This is
not proof that a future authenticated first publication will be permitted. No
names have been registered and no crates have been published by this change.

The shipping graph and publisher catalog stay deferred until the urgent next
minor carrying #9202 and #9199 is public and its source receipts are closed, as
confirmed by the release coordinator. Closing #9196 alone does not clear this
hold; merging a new six-package graph between pinned release sources would
invalidate that delivery. The manifest resolver is prepared for the six vendor
paths; adding the packages to the ordered publisher catalog is a separate release
coordination step. The
local workspace version is preparation, not a public-release promise. Per the
pre-1.0 release policy, ship the migration in a new minor release after the
urgent release and source-receipt closure finish.

After source and package/native Actions pass:

1. Confirm the publication identity and ability to create the six namespaces;
   report any actual registry authentication or ownership rejection verbatim.
2. Add the six packages to the publisher catalog before their consumers, in
   dependency order: core, JSON-RPC, client, LSP, orchestrator, facade. Preserve
   existing runtime 1.14.2 rather than republishing it.
3. Publish the new minor's six packages before Canon and other Vize consumers.
   Preserve full upstream notices and archive provenance. Verify each exact
   published package/version and owner configuration.
4. Build a fresh registry-only consumer with no paths, patches or workspace
   dependency source. Require default and distributed builds/tests, Canon's
   default native API and one runtime provider. The optional
   `checked-corsa-public-consumer.py` records downloaded archive checksums,
   complete source/license membership and release identity alongside those
   builds. It never publishes packages. The CLI package is not publishable;
   its five feature builds remain independent source acceptance, and the
   installed release binary needs its own qualification. Archive/path
   acceptance cannot satisfy public-registry acceptance.
5. Run the release's unchanged original performance and application/editor
   qualification against the immutable release identity, then verify public
   installations. Keep the original #8675 failure evidence until every claimed
   replacement result is independently available.

No upstream comments, changes or namespace operations are necessary to review
this source migration. An actual publication failure is a concrete release
blocker; the read-only 404 result alone is not an authorization request.
