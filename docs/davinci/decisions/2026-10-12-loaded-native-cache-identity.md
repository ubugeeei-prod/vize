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
authority. This is an unresolved correctness failure; PR #8704 stays off queue.

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

TODO: qualify a bounded refusal guard with the unchanged native bodies, all
fourteen original whole modules and the frozen dependency/browser plane.
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
