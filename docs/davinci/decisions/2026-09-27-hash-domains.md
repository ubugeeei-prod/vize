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

## Witnesses and pending capture

- Existing TS-43 `base.keys` is archived unchanged as `base-v1.keys`; SHA-256
  `25c449cf36249bf7fabc8de9c2cf41ce3a491463a5d23b336cbdcc81b874ec5a`.
- Current seven keys stay pending until actual frozen source execution.
- `hash_domains` pins fixed preimages for all four domains, all six SFC and
  three global facets, and isolated domain/stage/recipe negatives. Every
  expected digest is null until actual producer capture and independent
  one-shot XXH3 verification. Normal tests fail closed while capture is pending.
- Four real store tests and a real native provider/consumer cache test use
  an old-source key fixture whose fields are also pending actual capture.
  Ignored observation tests print actual values only when explicitly invoked;
  they are not acceptance gates and do not turn missing references into pass.
- Production provider dependency assembly is exercised in its NAPI-gated child
  test; default Rust tests do not imply this NAPI-only path executed.
- Existing TS-42, TS-43, actual plugin/transform/output snapshots, native addon
  contracts, workspace differentials and WASM gates remain required.
  The existing TS-43 job now runs on PRs and includes the four-domain target
  plus SFC/global summary tests on its existing Linux/macOS matrix.

No actual Cargo, ABI, compiler output, WASM, vector or Actions pass is recorded
by this source preparation. Capture receipts must bind immutable source/binary,
inputs, profile/build stamps, commands, raw outputs and exact exit status.
Old archives and source corpus membership remain unchanged; metadata and
historical `.keys` files do not add product inputs.

## Source budget and record exception

Six product files own the identity delta. Small existing cache modules register
focused test children; the existing 350-line NAPI SDK test changes its prefix
assertion in place. Summary.rs stays at 347 lines; touched sources stay <=350.
The central decision record receives one bounded companion link with no line
count growth. This active documentation file is the explicit exception to the
2,936-file protection inventory: 2,935 records remain byte exact, as do all
245 captured differential archive files and the original TS-43 v1 golden.

Whole #6832 and history/native acceptance remain open.
