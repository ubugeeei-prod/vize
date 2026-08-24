# TS-40 Current-Projection Differential Baseline

This lane freezes the current Canon and Maestro virtual-language behavior before
P4-5 replaces both generators. It is a migration oracle, not evidence that
Davinci or S2 already owns the projection.

## Lanes

- **Canon editor/socket projection** records exact digests for both pre-rewrite
  and consumer-visible rewritten virtual text, `VizeMapping` rows and
  sub-spans, semantic links, import-source-map adjustments and offset probes,
  and the diagnostics emitted by Canon's Content Mapper projection where that
  producer is compatible.
- **Content Mapper protocol** separately records its generated text, protocol
  span tuples, semantic links, diagnostics, and authored-anchor hits.
- **Maestro editor projection** separately records its virtual documents,
  `SourceMap` rows, and every mapping that resolves a declared authored anchor.
- **`vize check --show-virtual-ts` presentation** uses the same fixture matrix
  through a deterministic socket oracle and records the exact sorted JSON
  surface. This verifies CLI framing separately; it does not substitute the
  socket oracle for corpus diagnostic parity.

All hashes use fixture-relative input names. Consumer-visible virtual-document,
mapping, sub-span, semantic-link, diagnostic, and authored-hit sequences retain
generation order so an order-only regression fails closed. The CLI JSON lane
keeps its production file-order normalization. Timestamps, absolute paths and
process ids are absent.

## Exact fixture scope

The machine-readable matrix is
`tests/_fixtures/davinci-ts40-projection/matrix.json`. It covers:

- UTF-8 identifiers and materialized CRLF input;
- malformed-SFC recovery as an explicit diagnostic/error outcome;
- dual normal/setup scripts and emits;
- Options API props and slots;
- generic SFCs;
- JSX and TSX script blocks;
- a parent and child SFC with a local `.vue` import, including rewritten code
  and a non-empty Canon import source map;
- Vue 2 native-event syntax behind the Canon/Maestro `legacy` feature;
- authored mapping anchors for props, emits, slots, and navigation ranges.

The Vue 2 fixture is intentionally a mixed-lane record. Content Mapper's
production entry point is fixed to `legacy=false`, so its Vue 3/default
projection is always captured, with or without the Rust `legacy` feature. The
default build marks only Canon and Maestro's legacy generators as disabled; the
focused legacy build enables those two lanes. Snapshot names say
`mixed-vue2` and never describe the Content Mapper record as a legacy
projection. Canon's legacy lane does not borrow Vue 3-only Content Mapper
diagnostics.

## Fail-closed contract

Rust self-tests first run the real local-import mapping fixture and malformed-SFC
recovery fixture through the capture path, then inject mapping or diagnostic
corruption into those captured records. Node also tests the digest verifier.
Both require rejection with the matching drift class.
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
