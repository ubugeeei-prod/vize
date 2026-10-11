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

## Decision

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

Fresh exact-source Actions, full protected integration, actual merge and
installed official-release checks remain required. This bounded repair does
not block the first fresh official release's original #8623/#8631 acceptance;
a normal version bump already separates their manifest namespace. Neither the
source controls nor cache qualification establish publication or broad issue
completion.
