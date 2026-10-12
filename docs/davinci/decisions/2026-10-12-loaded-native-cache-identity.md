# Precompile caches identify the loaded native body

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
This follows the source-only [#8623 observation](./2026-10-11-8618-vite-cache-runtime.md)
and does not replace its pending installed-release acceptance.

## Observed failure

The original reporter project stays at commit
`6cb621553b40c6af751c2d4bf2dc32ae09bc5790`, with all fourteen SFCs,
154 non-Vize package and snapshot entries, original configuration and eleven
browser assertions unchanged. Vite remains 8.3.2, Playwright remains 1.63.0 and
Vue/Vapor remains 3.6.0-rc.10. Only the candidate Vize packages may change.

The original wrapper-directory scan finds no binaries when pnpm installs the
platform package beside the wrapper. An independent warm control first loads
the published 0.443.0 native body, records all fourteen complete modules, and
then atomically replaces only its owned platform slot with the qualified source
body at the same package version. Both Vite instances restore fourteen modules
without recompilation: six complete modules remain stale and the unchanged
browser suite records six passes and five failures. Its manifest key stays
`6aa619395d73b1c6c87f70ffa8acb5a8`; the whole manifest SHA-256 stays
`58773810e9e5562e5ba65987fcffcf7e26dc5099f9558a00987448192eaa3a14`.
The original cold source control passes all eleven assertions.

The independent same-process control also preserves Node's already-loaded
native object after that on-disk replacement. All fourteen complete outputs
remain from the old body. A fresh process loads the successor and changes
exactly six whole modules. Hashing the replacement file after loading would
therefore incorrectly attribute old in-memory code to the new compiler.

The complete negative and memory control index is retained as
`/tmp/vize-native-cache-rollover-independent-red-and-memory-proof-20261012.json`,
SHA-256 `3f0cca4c330ef9295bca944b4b5f41455a6882f0bfafdac0f34c3f3a62cb09c9`.
It retains original inputs, full modules, complete results, load-time native
body/stat observations and harnesses; no reference packet is rewritten.

## Initial attempt, superseded by the image-token decision below

Capture identity at the existing common successful native require boundary.
Stream SHA-256 once before the first `.node` load; compare device, inode, size,
mtime and ctime before and after the read and after successful loading. Keep
that immutable identity in a WeakMap keyed by the actual returned native
binding. The public entry point exports that same binding.

The Vite manifest key includes its package version, captured body SHA-256 and
byte count, alongside the unchanged format and complete compile options.
Paths and file timestamps do not define compiler equivalence. Reading the
identity later never rehashes a file or relabels already-loaded memory.

Path overrides, local binaries, platform packages and universal fallback use
the same boundary. Preserve existing target selection, version validation,
fallback order, errors and static named exports. An already-cached binding
without capture, unknown WASI/custom JavaScript provider, unreadable file or
identity race receives a stable process nonce: it cannot reuse another
process's persistent cache. The helper is shipped in the ordinary CJS package;
there is no Rust rebuild, provider replacement or new compiler pipeline stage.

## Qualification and boundaries

Ten focused controls pass: loaded-memory stability, one streamed hash per
initial load, same-size/time replacement, fresh-process body changes,
preloaded/custom providers, replacement during loading, actual Vite key
wiring, unchanged-body disk reuse, older-wrapper refusal and existing packed
loader/version/failure/export contracts. The isolated extension-handler
controls model Node's require cache; they do not claim real native execution.

Normal `vp pack` and `pnpm pack` produce the candidate Vite plugin and native
JavaScript wrapper. Independent real native warm-rollover qualification must
retain the frozen 154-entry dependency plane and all fourteen whole modules:
old-body warm cache, successor-body recompilation without manually clearing
Vize or optimizer caches, all eleven original browser assertions and unchanged
successor reuse. Load-time body capture must remain distinct from the file at
process exit. Preserve every original negative receipt.

## Independent packed-candidate result

The complete eight-phase receipt is
`/tmp/vize-native-cache-rollover-candidate-independent-acceptance-20261012.json`,
SHA-256 `679c3c736c146a1bc6950d29e2aaea707e05382daa80e86128387623e0ebdfbc`.
Its 99 complete evidence files and 36 original committed source blobs were
independently rehashed. Both default and configured-cache consumers preserve
the exact original 154 package/snapshot entries and eleven browser assertions.

For each cache location, the old published native body records six passes and
five failures. Without deleting either old Vize or optimizer cache, the fresh
successor process recompiles fourteen modules, then restores all fourteen on
its second instance: all eleven assertions pass. A repeated unchanged-successor
run recompiles none and passes all eleven again. Only after those receipts are
frozen, a separate same-path cold authority passes all eleven and matches every
field of all fourteen complete modules and the entire compressed manifest
byte-for-byte, without normalization.

The real same-process control retains its frozen old-body identity and all
fourteen old complete results after an atomic disk replacement. A fresh process
captures the successor body. Two genuine providers loaded before capture get
stable-within-process, distinct process nonces and retain all fourteen complete
successor outputs. The ordinary Vite tarball is SHA-256
`98a3e0aa3c60802ac022e9ae8d836e25e1344958e1001b757ec4fb959e3698c3`;
the native JavaScript wrapper is
`6d74b844240fd58c6e1b7fbbcf51ca9f78aadc34a3a59353950d7d7c4e01b856`.
The retained native bodies and all original observers are unchanged.

The first source Actions retain an owned type-aware `unbound-method` warning,
corrected by declaring the getter as a function property; runtime output is
unchanged. The separate existing custom-directive source-custody guard rejects
composed current-main files before runtime. Preserve both full failures and
require the actual shared-guard repair plus fresh exact-source qualification;
neither failure is waived by these packed-candidate controls.

Fresh exact-source Actions, full protected integration, actual merge and
installed official-release checks remain required. This bounded repair does
not block the first fresh official release's original #8623/#8631 acceptance;
a normal version bump already separates their manifest namespace. Neither the
source controls nor cache qualification establish publication or broad issue
completion.

## Newly observed same-path operating-system reload

The preceding eight phases qualify ordinary fresh-process warm rollover only.
A separate real-native control loads the old public body, atomically replaces
its owned native slot with the unchanged source body, removes the `.node`
require-cache entry and loads the same filename again. Node returns a new
JavaScript binding, but all fourteen complete results remain from the old
mapped image. The first implementation incorrectly assigns the replacement
file's SHA-256 to that binding. A fresh process returns the complete successor
authority. This was a correctness failure in the disk-hash attempt. Keep that failure
packet; only the reviewed image-token successor below may qualify PR #8704.

The full original twenty-eight results and fresh successor authority are
retained under `/tmp/vize-native-cache-os-reload-diagnostic-20261012`.
Its custody index is
`/tmp/vize-native-cache-os-reload-diagnostic-custody-20261012.json`, SHA-256
`4f5fd66223ed55ac84985fb627eab8583972b1211a74859cbe51e3cbd1b5d07a`.
Independent new slots reproduce the failure after helperless preload and
cache eviction, across two helper copies, and in a fresh Worker isolate.
Only their `sharedObjects` arrays are recorded, never whole diagnostic reports
or environment contents. Every case positively exposes the old mapped path.

Known prior image loads and unavailable process-wide identity must refuse
persistent reuse and receive a stable process identity. A Symbol registry can
share path history among helper copies in one isolate; it is not proof of
process-wide ownership. A path absent from an OS report does not by itself
prove which native body executes. Preserve all previous scoped receipts.

The bounded-refusal proposal was superseded by the image-token decision below:
retain the healthy cross-process restore while refusing unknown images.
Avoid adding loader waits: [Node 24.14.0's report implementation](https://github.com/nodejs/node/blob/v24.14.0/src/node_report.cc)
synchronously collects child-worker reports, while its
[loaded-library implementation](https://github.com/nodejs/node/blob/v24.14.0/src/debug_utils.cc)
uses platform image enumeration. These APIs require explicit visibility and
worker-boundary evidence before being used as production identity authority.

## Reviewed image-token successor

An additive `getNativeBuildToken()` getter returns a token embedded in the
actually linked image. Each existing native build invocation creates a fresh
128-bit token, overrides an inherited token, and passes it to Rust. The build
script watches that environment value so a warm Cargo cache relinks this crate
with the new token. Direct uninstrumented builds return no token. Keep the
separate partial `VIZE_PLUGIN_HOST_BUILD_ID` contract unchanged: its watched
source closure and revision are not full native-image authority.

The common successful require boundary reads the actual image getter once and
keeps an immutable identity for its returned binding. File replacement and
same-path OS reload cannot replace the token of already mapped code. Execute
require exactly once before optional path resolution; preserve its original
failure, and permit cached bindings whose disk path no longer resolves.
Unknown, old, WASI, custom JavaScript, inaccessible or invalid-token providers
receive a stable process nonce. No production disk hash, report or startup
image scan is required. Vite keys include the actual image token alongside the
existing package version, complete options and unchanged manifest format.

Independent producer inventory covers local release/debug, CI, and the Moon
release helper's Apple Cargo and non-Apple NAPI paths. Its complete fourteen
source blobs and twenty-nine caller rows are retained in
`/tmp/vize-native-build-producer-inventory-20261012.json`, SHA-256
`b562858d4642d171b25c5bf1475e75ca2a5ba10aca98d1e1f9e793fee049beb7`.
Seventeen isolated controls pass, including same-path image reuse, independent
helper copies, deleted-path cached loading, original errors, old-build refusal,
same-image persistence and fresh build-environment identity. These modeled
image tests do not replace actual native qualification.

Three MoonBit native runtime controls execute byte-exact extracts of the two
production functions: every nibble including zero padding, actual platform
entropy and complete inherited-environment preservation with token override.
The complete fifty-five-file custody index is
`/tmp/vize-native-build-token-moon-whitebox-custody-20261012.json`, SHA-256
`d24e7ed2e4305596121ae0a093aa53552d8c808e5020366f3300c435a6f191c5`.
Local full-package execution remains unqualified because the existing `x`
dependency uses syntax rejected by the available local toolchains. The durable
whitebox tests must execute in the selected Actions producer before native
building; extracting functions is separate evidence, never full-package credit.

The existing Native Smoke default/schedule retains all six host and twelve
fresh-install cases. An explicit Darwin ARM64 host-only dispatch reuses all
original host steps and retains a source/token/binary custody artifact; it
establishes selected producer evidence only. Obtain that one necessary fresh
Actions image and reuse it for the frozen fourteen-module, 154-entry, eleven
browser-assertion controls and all four actual OS reload cases. Preserve the
ordinary healthy cross-process fourteen-module restore. No local duplicate
native builder is authorized for this qualification. Source Actions, exact
protected integration, actual merge and public acceptance remain pending.

### Actual selected image and missing producer receipt

Selected source `9c1acd36e2e2b6484f553c248931021cfb2cd2be`, run
`38182345080`, attempt 1, passes all three real full-package Moon laws before
the existing CLI/native builds. Artifact `11718247253` has complete ZIP SHA-256
`34dbd8080f81e0c4e7cf2ac68d0a536fd10125631c97467cdac678db5bc84040`.
The uploader excludes the hidden producer JSON despite a successful capture
step; preserve the original strict authentication failure. Future explicit
uploads include hidden files only from the existing finite path list.

An independent local observer authenticates every member CRC and complete
native body, actual getter token, unchanged generated JavaScript/helpers and
genuine source bindings against the successful run and ZIP. This is distinct
observer authority; the absent Actions producer JSON is never reconstructed or
claimed available. The partial embedded plugin-host revision corroborates
`9c1` but cannot establish full image authority. Preserve raw generated and
committed declarations and their complete difference independently. Reuse this
one actual image; source-only successors retain their separate exact Actions
requirements. No duplicate native builder is required.

### Actual OS image-reuse acceptance

The same selected image passes wrapper reload, helperless preload followed by
cache eviction, two independent helper copies and a fresh Worker isolate.
Each new owned APFS slot initially loads the authenticated image, then swaps
only its disk file to the old published body and evicts only that `.node`
require-cache entry. The mapped image retains its embedded token and every
field of all fourteen complete successor results. A fresh child of the same
path loads the old body, receives a process nonce and returns all fourteen
complete old results. The Worker exit is awaited with code zero; these
controls invoke no process-report API. Both old uninstrumented bodies retain
stable frozen nonces within a binding and cannot claim persistent identity.

The forty-one-file complete index is
`actual-image-token-os-controls-index.json`, SHA-256
`a1e63216752fd01fe58f6164c87b291faea5b49ba1fff5da3af4451babda9c68`,
under the separate controller root recorded by
`/tmp/vize-native-image-token-acceptance-controller-index-20261012.json`.
The independent parent readback rehashes every file and compares all complete
result arrays, sources and options to the retained old/successor authorities:
`/tmp/vize-native-image-token-os-independent-parent-readback-20261012.json`,
SHA-256 `d67343110800e48721289da077f231b6c27ac2484b2b428b3d915b6750f3523b`.
Nothing is normalized and every original negative packet remains unchanged.

### Finite declaration boundary for runtime qualification

Preserve both complete generated and curated declarations. All 199 names
exist in both; 196 shared structures match exactly. The three preexisting
differences are `EndOfLineNapi` enum versus the same four string literals,
`OxlintHtmlRule.authoredOptions` and `OxlintHtmlDiagnostic.fix` curated unknown
versus raw any types. Four preexisting documentation groups and declaration order also
differ. The owned getter documentation and nullable return type match exactly;
a nonnullable mutation is rejected. Every old declaration and documentation
body is unchanged from the original source; only the getter is added.

The complete two-declaration AST review is
`complete-declaration-authority-review.json`, SHA-256
`ccff5d6a707b0a9fcda948500cef398d285ffba4973d557ec721b3c2c474c516`,
in the same controller receipts. This explicit existing boundary permits
only the bounded runtime controls using the unchanged curated-source package
and authenticated image. It does not establish broad declaration or shipping
API equivalence and authorizes no declaration rewrite.

### Complete unchanged browser and cache acceptance

Both default and configured-cache consumers preserve all twenty-five original
committed blobs, including the twenty-three unchanged runtime/config/test
files. The candidate plugin pin and Vize lock entries are the only dependency
changes. Each physical pnpm store retains all 154 original non-Vize package,
snapshot and importer entries, with the same actual runtime versions.

All eight original eleven-assertion browser runs are retained. The old public
body produces six passes and five known failures in both consumers. Without
clearing either cache, replacing only the owned native slot with the selected
image causes fourteen recompilations on the first server, then fourteen disk
restores on the second; all eleven assertions pass. A fresh unchanged-image
process restores fourteen modules without recompilation on both servers and
again passes all eleven. Only after these results, move both owned caches
intact to receipts and run an independent same-path cold control. All eleven
assertions pass; every field of all fourteen complete modules and the entire
compressed manifest match automatic and warm results exactly. Both original
old manifests remain byte-exact. No source, assertion or output is normalized.

The complete 291-file index is
`/tmp/vize-native-image-token-complete-independent-controls-20261012.json`,
SHA-256 `d8a12e6b704349c465a27b0709ca264af42d3dd9836250f3758ca0c96b4f5718`.
Independent readback verifies every file, committed input, physical lock plane,
actual native load/getter, complete result and cache byte:
`/tmp/vize-native-image-token-complete-independent-parent-readback-20261012.json`,
SHA-256 `a7b65661c99be23209daf379cc966762cd1b364d09c8332472a0986fc627add4`.

The ordinary source plugin tar is
`3025bfef5c958fff15336d93e166dfd1565442044df693449a5b5c2b88116919`;
the unchanged curated-source wrapper/declaration tar is
`de90c4607cd9df89afc255079ab2c310d560cf16345a76adca2b20eca663b587`.
The image remains the genuine selected `9c1` artifact, never a rebuilt
successor image. Native Rust, loaders, token producers and Vite runtime retain
those qualified bytes. Exact
source `3b7ea1dc16127853f34ef6c6adeff9b501bf0502` passes 41 ordinary Actions
checks with no failures; eighteen normally skipped lanes remain skipped.
Rebasing this evidence record requires fresh exact-source Actions. Full
protected integration, actual merge and fresh official installed acceptance
remain separate required gates. The original #8623/#8631 release observers
continue independently; this same-version follow-up does not delay them.
