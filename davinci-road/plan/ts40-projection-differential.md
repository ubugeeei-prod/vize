# TS-40 Current-Projection Differential Baseline

This lane freezes the current Canon and Maestro virtual-language behavior before
P4-5 replaces both generators. It is a migration oracle, not evidence that
Davinci or S2 already owns the projection.

## Lanes

- **Canon editor/socket projection** records exact digests for pre-rewrite
  virtual text, `VizeMapping` rows, semantic links, and the diagnostics emitted
  by Canon's Content Mapper projection.
- **Content Mapper protocol** separately records its generated text, sorted
  protocol span tuples, semantic links, diagnostics, and authored-anchor hits.
- **Maestro editor projection** separately records its virtual documents,
  `SourceMap` rows, and every mapping that resolves a declared authored anchor.
- **`vize check --show-virtual-ts` presentation** uses the same fixture matrix
  through a deterministic socket oracle and records the exact sorted JSON
  surface. This verifies CLI framing separately; it does not substitute the
  socket oracle for corpus diagnostic parity.

All hashes use fixture-relative input names. Mapping rows, semantic links,
virtual documents, diagnostics, and authored hits are sorted before hashing;
timestamps, absolute paths, process ids, and iteration order are absent.

## Exact fixture scope

The machine-readable matrix is
`tests/_fixtures/davinci-ts40-projection/matrix.json`. It covers:

- UTF-8 identifiers and materialized CRLF input;
- malformed-SFC recovery as an explicit diagnostic/error outcome;
- dual normal/setup scripts and emits;
- Options API props and slots;
- generic SFCs;
- JSX and TSX script blocks;
- Vue 2 native-event syntax behind the `vize_maestro/legacy` feature;
- authored mapping anchors for props, emits, slots, and navigation ranges.

The default-feature snapshot records Vue 2 as feature-disabled rather than
silently treating it as Vue 3. The focused legacy run owns the enabled snapshot.

## Fail-closed contract

Rust and Node self-tests clone a known record, inject mapping or diagnostic hash
drift, and require the verifier to reject it with the matching drift class.
Every runtime lane also rejects an empty fixture matrix. Successful non-recovery
fixtures require non-empty Canon, Content Mapper, and Maestro mapping products,
plus authored anchor hits on both mapping models.

## Not proven

This baseline does not prove:

- Davinci/S2 projection parity or readiness;
- full-corpus `vize check` false-positive/false-negative parity;
- complete tsgo Content Mapper editor-feature coverage;
- incremental, watch, or multi-project invalidation;
- that either current generator can be switched or deleted.

Those remain P4-5/TS-40 exit work. The old-vs-new differential lane gains a new
side only after an S2 projection exists.
