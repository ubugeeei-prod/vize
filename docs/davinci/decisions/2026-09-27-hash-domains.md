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
  its current `[disegno-folio]` grammar. L3 dump grammar is unchanged.
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
- The existing TS-43 job also runs on PRs with hash-domain/SFC/global targets
  on its existing Linux/macOS matrix. No new workflow or dependency was added.

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
