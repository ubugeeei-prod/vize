# Source-qualified Nuxt 3 fixture

Decision for [#7838](https://github.com/ubugeeei-prod/vize/issues/7838), paired
with [the issue comment](https://github.com/ubugeeei-prod/vize/issues/7838#issuecomment-5983122808).
The immutable audit source is `61c975f8889a002a1b53265d86d0985447ffab63`.

## Existing coverage and gap

The existing Nuxt 3 fixture pins Nuxt 3.19.3 and Vue 3.5.43. Its workflow
packs the candidate Nuxt module and Vite adapter, but leaves the installed
native 0.429.1 dependency in place. Successful SSR routes, scoped CSS,
Chromium counter updates and navigation therefore did not prove that the
current Rust compiler produced the application.

The authentic earlier run 37200408329 used checkout
`5171b3d4c1bb12839a3de23efd6e946cd7792fd4`. Its artifact 11302622537 has ZIP
SHA-256 `8eb3d01e18f217ebedcd2e19dd4a496a7552df373e176e62edf67a9cef1b7dc1`.
The logged client 727 ms and server 144 ms are internal stage messages,
not a total build wall time or a current-source performance baseline.

## Quality witness

Build the actual source addon on Actions with the existing locked debug
NAPI capture. Reuse its validator without altering its schema or production
API. Preserve the source head/tree, lock hashes, actual Cargo completion,
declared `napi,legacy` features, original build command, raw stdout/stderr,
toolchains, generated artifact identity and frozen binary SHA.

Copy that authenticated binary into the fixture artifact. A test-only
preload sends only Vize addon requests to that binary through the original
physical native loader; unrelated addon requests retain their original
filenames and exports. Do not set the global `NAPI_RS_NATIVE_LIBRARY_PATH`:
Rolldown also consumes it. The preload records both requested and actual
Vize targets, so published target resolution cannot load published bytes. It forwards
the original arguments to the original functions and returns the same result
or exception object. It records complete arguments and results, including
maps, after the native call; no production stage or serialization is added.

Require actual load-before-call records from each process. Every original
tracked fixture SFC must reach the genuine compiler with its complete bytes
and original physical filename in both client and SSR modes. The receipt
must still match the checkout and copied binary after the entire fixture
passes. The existing fixture sources, npm lock, SSR/CSS assertions and real
Chromium interaction assertions remain unchanged.

The workflow also runs for protected merge groups and Rust/native changes.
Its extra job is not one of the four ruleset-required contexts. Protected
delivery must inspect its actual compiled checkout and terminal result in
addition to those required contexts. Local guard controls exercise rejection
and call identity with explicitly synthetic bytes; they grant no native or
Nuxt execution credit. Source Actions and protected delivery are pending.

This debug-profile lane proves current legacy compiler quality only. It is
not a production speed measurement, a default Davinci migration, full Vue
parity, or direct Vapor SSR support. Requested SFC Vapor SSR still emits
`VAPOR_SSR_FALLBACK` and uses standard SSR. Existing UI hydration interop and
its known-failure ledger are separate from direct Vapor SSR support.

## Proposed finite build measurement: not authorized to execute

Use this existing four-SFC Nuxt 3 fixture for a first bounded experiment.
Freeze two GNU x64 release-profile compiler binaries: a verified published
baseline (currently 0.432.0) and a fresh exact-source candidate. Use identical
current Nuxt/Vite adapter bytes, Nuxt/Vue versions, fixture source and lock,
Node/Rust/NAPI toolchains, runner class and declared features in both arms.
Only the native compiler binary differs. Preserve both package and binary
identities; an obsolete unpublished release artifact cannot qualify.

The fixed matrix is two arms times three pairs times cold/warm: twelve
builds. Pair order is baseline/current, current/baseline, baseline/current.
Each arm runs cold then warm. Preserve every exit, signal, timeout, stdout,
stderr and output; no retries, replacement trials or best-of selection.

Materialize independent writable fixture and dependency trees for each arm.
Installation, adapter packing and addon construction are outside timing.
Before each cold build, remove and enumerate `.nuxt`, `.output`,
`node_modules/.vize`, `node_modules/.cache` and `node_modules/.vite` under that
arm only. Cold describes application caches, not OS page caches. Warm is a
fresh Node process immediately following that arm's cold build, retaining
only its own caches and outputs. Record cache inventories before and after;
an inventory alone is not proof of a cache hit.

Measure monotonic wall time immediately before spawning the existing Nuxt
build command until its process exits. Include startup, compiler/precompile,
Vite/Nitro and output writes. Exclude installation, cleanup, source addon
builds and subsequent SSR/browser checks. Keep internal client/server stage
messages separate. Apply the same 300-second build deadline to every cell.
Retain all twelve individual values, failures and predeclared pair ratios;
aggregate only comparable successful pairs, with missing pairs explicit.

Validate the existing complete SSR/CSS/Chromium assertions after each
successful build outside its timed interval. Archive complete emitted code
and available maps, their source content and output inventories. Compare
those outputs across arms and explain any actual fix-related differences;
do not silently normalize or claim byte parity from file counts.

The full per-call JSON observer in this quality lane must not be timed as
production performance. A concrete matching load/count-only observer (or a
qualified immutable binding selection proof) still needs independent
review, preserving the same behavior in both arms. No timing runner or
dispatch is added here. Root review of the frozen binaries, observer,
cache/timeout laws and complete twelve-cell packet is required before any
measurement. Nuxt 2's `compiler: false` fixture remains a compatibility
control; standalone Vite and Vapor SSR need separately qualified workloads.

## Follow-up

The separately owned #7831 conditional whitespace fix adds the preserve
option to this fixture. Its original installed-native lane failed the real
Chromium heading wait in run 37224764561 while build and SSR passed. That
result does not qualify the new Rust source. Inspect the actual union of
the source-binding lane, preserve option and compiler fix with all existing
assertions before claiming Nuxt delivery. This change does not overwrite
that owner's fixture or compiler files.

Keep #7838 open until exact-source Actions and protected actual delivery
are verified. Release identity, a reviewed measurement observer, a concrete
runner and explicit measurement authorization remain TODOs. No performance
improvement is claimed.

The initial source Check 37225561805 at `686656ca90dc3d6aeb23c2a9ed6a63953176550a`
failed only the zero-warning JS gate: the new proof target sort required an
explicit comparator. Preserve its original UTF-16 ordering with that
comparator; no fixture, compiler, runtime assertion or performance change.
The initial source-native build was still running and grants no execution
credit. Fresh exact-head Actions are required after this correction.

At `fe3d0d1343fe766f39ecc8fdbef0ebf12e02b6bb`, Nuxt run 37225695959 failed
before compilation: Nuxt reports only `Could not load @vizejs/nuxt` and
discards the import exception. Authenticate artifact 11311937358, ZIP
SHA-256 `2fd47c55889de5704b14c978a473db2b7cb916403424a532ce63dcd24b0a8390`.
Its Cargo exit is zero at checkout
`d9695d2044598ae14daa69dd5cde8f93e4e02726`; actual addon SHA-256 is
`0cac8e35fab9cbba41cf09dcf87ea94af9a5a24718e14f842357311728a38fba` with
`default,glyph,legacy,napi` features. The packet records one physical load
and no compiler calls. This is not a compilation, SSR or browser pass.

Retain a bounded separate module import under the same source-binding
environment before the unchanged build command. Preserve its original
exception in `module-import.log` instead of diagnosing from Nuxt's generic
error. This import gives no workload or performance credit. The underlying
cause and quality result remain pending; do not bypass the binding guard,
weaken fixture assertions or retry the unchanged failed source.

The diagnostic source `8f77de18405ce4bef8cc795bfe980ff45ed737fc` isolated the
actual import failure in Nuxt run 37226395369. Authenticated artifact
11311714217 (181,867,326 bytes) has ZIP SHA-256
`13156e6dbfd7c8492db58e4fa91a2e502c79919f9e10a153c288ca35c9022d4e`.
Rolldown's `TsconfigCache` extends an undefined native export because its
loader consumed the same global NAPI override and loaded the Vize addon.
The original fixture lock pins Rolldown 1.2.11; its npm tarball SHA-512
matches that lock and SHA-256 is
`3acd96aab699f4936c223ebe60a52882a22aaa7dba22393f97b30729dd26516b`.
Its original `dist/shared/binding-BY0qR5iS.mjs` explicitly reads that override.

Remove the global override and scope the physical selection to Vize in the
test preload. Preserve the actual native loader for all other addons. The
extended control requires original Rolldown-like export/constructor identity
and physical filename, exact selected Vize binary, unchanged compiler
arguments/results/exceptions, and rejection of ambient global overrides or
stale selected bytes. These synthetic controls are not runtime credit.
Retain the original failure and require fresh exact-source Nuxt/SSR/browser
execution; successful quality and protected delivery are still pending.

Source `e2b59f21194f11b5caf5d7e1e535d261a232d428` passed genuine Nuxt run 37227242892. Artifact 11312995256 (181,890,926 bytes), ZIP SHA-256
`746f9a0420b70bdf6fb4d5b0570c70183f1914de983b3913040bfa923bd0b033`,
authenticates checkout `07f5981245629080aa5b3d9d3b859f742e2e1b77` and actual
binary `8d5d02e10b6a9faa767c586a2dace12a4ead425a41458a9d737c1a41ce350965`.
Its one compiler process made eighteen calls. All original four SFCs reached
both backends with exact complete bytes, empty native errors and nonempty
generated code; the original SSR/CSS/Chromium assertions passed. Only the
index SSR result supplies a map, so this does not claim all-map coverage.

Close a future acceptance hole found while auditing those actual results:
require one matching original native result, empty errors and nonempty code
for each original fixture compilation. An adapter fallback cannot turn a
returned native error into source-compiler quality credit. Preserve other
generated Nuxt inputs' existing adapter policy. Extend rejection controls
for errors, missing results, empty code and wrong result paths. This changes
the acceptance law, so the earlier pass stays source-qualified and fresh
exact-head Actions remain required before readiness or protected delivery.
