# Canonical functional hash identity — #6832

Status: source prepared; actual compilation, capture and acceptance pending.

## Scope

The owned hash domains become `vize.artifact-key.v2\0`,
`vize.key-manifest.v2\0`, `vize.sfc-summary.v2\0`, and
`vize.global-summary.v2\0`. The final `\0` denotes an actual NUL byte.
The three current L0/L1/L2 artifact key recipes become 2. The first three
CachedArtifact role names become `l0.source-block`, `l1.surface-page`, `l2.page`.

Stage::wire_id now names logical l0/l1/l2/l3/l4 boundaries. Its current
production callers are the hash prefix and key Display. Emit/l4 is a logical
backend boundary; there is no physical L4 crate or current L4 key producer.
No L3/L4 artifact recipe or native corpus acceptance is added.

This is an intentional experimental key identity break, not a compiler output
change. Types/signatures, explicit byte encodings, relative spans, ambient
input declarations and current semantic payloads remain unchanged.

## Contracts kept separate

- α/SFC/global fact schemas remain 1. Summary dump grammar remains unchanged.
- L1 remains lossless syntax without a new owned Page; L2 Full dump still uses
  its current `[disegno]`/`[disegno.ops]` grammar. L3 dump grammar is unchanged.
- CLI pipeline labels/maps, Stage diagnostic strings, StageFeed, counters,
  profile hashes, DOM receipts and production captures keep their own contracts.
- Doctor capability cache, resident source/config hashes and batch compile
  cache do not share these four domains and are unchanged.

## Persistent caches

The diagnostic, provider-fact, output-plugin and transform-plugin stores keep
existing envelopes, paths, size limits and validators. Every store checks the
complete saved key. New hash identities must miss old records, including
old records copied to the current key's path. A new record must hit through
a fresh instance/disk read. Provider facts also re-audit result fingerprints;
changing their stamp must invalidate dependent rule caches with equal fact bytes.
No old-key alias, fallback, cleanup policy or historical key reader is added.

Plugin cost contentKey, provider result fingerprints, cache-hit flags and timing
are permitted telemetry changes. Actual compiler code/preamble/map/helpers/
templates, facts and diagnostic messages/spans must remain byte equal. Testing
the complete Result object while silently removing fields is not a parity claim.
The new public ABI regression uses the real Node addon; all other unknown or
external callers remain outside the measured scope until verified.

## Actual captures and pending acceptance

- The original TS-43 seven keys remain byte exact in `base-v1.keys`; SHA-256
  `25c449cf36249bf7fabc8de9c2cf41ce3a491463a5d23b336cbdcc81b874ec5a`.
  Actual source `de5e656` generated seven current keys, SHA-256
  `3d1676cdf640de111723e5a3b3fbd546c42ecee9e9392ee363a13a42320466a4`.
- `hash_domains` captured 24 fixed preimages/digests: 11 real producers and
  13 isolated domain/stage/recipe negatives with no production result.
  A separate direct one-shot call to the existing XXH3 primitive verified all
  24 under the same installed Rust 1.98.0. This is not a second hash algorithm.
- Old probe `7298647` keeps production identity `3bf16a5` and captured five
  actual keys and provider values from four exact observers. Its real compiled
  transform build stamp differs from the three explicitly controlled helpers.
- The selected dev/default gates passed: key units 6, manifests 5, SFC 7,
  global 2. The initial reused library mismatch was refused and preserved;
  an authorized Davinci-only package clean and identical-source build resolved it.
  The initial one-shot compiler mismatch and exact-toolchain retry remain raw.
- `capture-v2/receipt.json` links actual source/binary/input/profile/commands,
  process status and unchanged raw outputs. Activated fixture tests and four
  real persistent-store migration tests require a subsequent source build.
- Production provider assembly is NAPI-gated; default tests did not execute it.
  Real addon, compiler output byte parity, TS-42, WASM, full CI and all normal
  TS-43/hash-domain acceptance remain pending. No native acceptance is credited.
- The existing push/manual TS-43 job includes hash-domain/SFC/global targets
  on its existing Linux/macOS matrix. It does not start on PRs; PR source checks
  retain the fast parent CI routing. No new workflow or dependency was added.

Capture source heads stay exact; later data commits are not relabeled as the
compiled source. Raw archive evidence is not executable source. Source corpus
membership and historical references remain unchanged.

## Source budget and record exception

Six product files own the identity delta. Small existing cache modules register
focused test children; the existing 350-line NAPI SDK test changes its prefix
assertion in place. Summary.rs stays at 347 lines; touched sources stay <=350.
The central decision record receives one bounded companion link with no line
count growth. This active documentation file is the explicit exception to the
2,936-file protection inventory: 2,935 records remain byte exact, as do all
245 captured differential archive files and the original TS-43 v1 golden.

Whole #6832 and history/native acceptance remain open.

## Replay on the repaired source graph

The local source follows `e004e3050`. The original source patch retains its
stable patch id outside the central record
(`35522410b99f8e3c8cb2e2c18ace1bd81f2908b6`), which retains all earlier
decision links. Historical capture and header-document correction patches
replay unchanged. The current capture fixture still has 24 complete digest
vectors, 11 with real producer identities; source-bound raw receipts and the
v1 golden stay exact. These historical executions do not certify a new head.

Forty-two existing pure/source/metadata contracts pass. Three native addon
cache tests were attempted at the composed L2 preparation and refused before
execution because this clean worktree has no native binding. They are not
passed or skipped acceptance. The one TypeScript formatting correction adds
an optional trailing comma and preserves the parsed AST. Cargo formatting,
Markdown formatting and the actual repository source-growth gate pass.
Authored-source whitespace passes; twelve immutable raw stdout captures keep
their original final blank lines, reported by a whole-diff whitespace check.

Native cache/store, CLI, Rust, WASM and exact-head Actions remain required.
The subsequent current L2 grammar changes its recipe to 3 and selects a
separate actual historical golden. Decisions and explicit limits are mirrored
in the [#6832 record](https://github.com/ubugeeei-prod/vize/issues/6832#issuecomment-5854223807).

## #6946 fast-CI and assertion repair

The published hash preparation inadvertently restored `pull_request` in
`davinci-incremental.yml` from older source, while its source contract still
expected the old two-target cargo command. The #6947 tooling failure exposed both
conflicts, plus the transform migration test's two partial prefix assertions.
The macOS runner was already identical to the real `ab21c5ceb` parent; there was
no runner drift to repair.

Repair source `0acf9d9085b6498d42260d5e803b6a03414b7772` is based on published
hash head `746ec0a171f19eeb55fd670666c130d845b8f5c6`. It removes only the restored
PR event and retains the intended three added TS-43 targets. A parsed deep
comparison proves every other parent workflow value exact, including both
platforms, runner labels, cache steps, permissions, concurrency and TS-42.
The workflow source contract now requires the exact five-target command.

`tools/support/levels/extend-hash-key-workflow.mjs --apply` replays only that
known run scalar on the current workflow. It never reads a frozen workflow body.
`--check` verifies idempotence. The helper refuses stale PR-trigger workflows and
unknown commands; its laws verify that changed current runners, caches, events
and unrelated jobs survive the replay unchanged.

The transform migration test now pins the complete observed old key
`s0.v1:e99ab135a693b1e38e520b7918806d3b` from the unchanged legacy fixture/receipt.
The current key embeds the compiled `VIZE_PLUGIN_HOST_BUILD_ID`, so it has no
head-independent golden digest. Instead, its complete parsed identity must equal
`l0.v2`, and its digest must contain exactly 32 hexadecimal digits that roundtrip
to canonical lower-case 128-bit text. No public parser or pinned current digest
is invented; the roundtrip checks formatting only. The behavioral witnesses remain intact:
old records miss, old records copied to the current path miss, and a current
record hits through both the cache entry point and a fresh disk read.

Focused local source contracts and replay laws: 11 passed. The transform subtree
assertion lint passes with zero findings and no allowlist. Cargo/JS/YAML formatting,
whitespace and source350 checks pass (11,387 files scanned; no new/grown over-limit
files). The full assertion lint has exactly one
remaining finding in the separate #6945 L3 dump protocol test; its owner repairs
that oracle before the stack reruns the full gate. All 41 existing captured key,
receipt and legacy fixture blobs remain byte exact. No product or full Rust build
ran for this repair.

After restacking the earliest repairs, rerun the exact-head PR source check and
TS-43 push/manual job. The actual transform-store test still needs the Vitrine
test build on Actions; source lint is not runtime cache/native acceptance. This
preparation is committed without push, PR mutation or issue comments.
