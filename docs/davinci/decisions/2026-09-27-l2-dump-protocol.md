# Current L2 dump version 2 and published version 1

Issue: #6832. This decision implements the bounded B1 codec change.

## Current core contract

`vize_l2::dump::Page` writes and reads only `[l2-dump-v2]` and
`[l2-dump-v2.ops]`. Provenance uses `[l2-provenance-dump-v2]` and its
`.records` section. The tree, operation vocabulary, field order, escaping,
normalization, optional values and spans are unchanged. Full and Display
retain their existing differences; provenance retains spans in both modes.

The codec is chosen by a typed constructor before input is inspected.
There is no header inference, fallback or replacement of serialized text.
Header-looking quoted payloads remain payloads. Direct core callers that
read the old headers must migrate intentionally.

## Published extension contract

Published `s2-page@1` still carries the historical `[disegno]` grammar.
`dump::historical::v1::Page` owns the same tree and selects that grammar
explicitly. The version 1 contract reader converts the accepted historical
wrapper into the current typed tree; the Vue version 1 writer selects the
historical wrapper. The original raw answer is retained exactly.

Schema checks precede parsing, and canonicality compares the complete old
wire text. SDK writers, WIT definitions, feature negotiation, diagnostics,
old parser messages and canonical-byte offsets remain unchanged. Shared
`DumpError` formatting is unchanged; a later formatting migration must
preserve the published version 1 envelope explicitly. The historical
wrapper has no artifact-key producer.

## Identity and evidence

Current L2 key recipe 3 feeds the version 2 Full grammar with the existing
block-relative spans. Before-base spans keep their absolute encoding. The
artifact-key domain, all other recipes, counters and feeds are unchanged.

The existing 24 hash vectors, seven version 2 TS43 rows and 35 raw capture
artifacts are historical evidence and remain immutable. Actual recipe 3
capture will create only `fixtures/keys/base-l2-recipe3.keys`; no digest is
inferred from the old fixture. Six L0/L1 rows must remain identical.

The inherited hash slice has actual normal-test and persistent-store
receipts (25 and 4 passes). Those receipts precede this codec change and do
not establish B1 runtime acceptance. Source formatting and preservation
checks likewise do not establish Rust, CLI, browser, Lean or native passes.

## Remaining work

- Run the source-built L2, published-host, identity and CLI gates and record
  the actual recipe 3 capture before claiming this slice verified.
- Integrate the separate owned collector with version 2 header declarations
  and `Some(2)` grammar metadata; its old standalone receipts remain intact.
- Run browser/WASM, old guest SDK/WIT, inherited Lean and complete product
  gates at the integrated source head through the existing Actions lanes.
- B2 current SDK/WIT capabilities and version policy remain a separate
  change. B1 does not establish current public native or SDK acceptance.
- Physical filename and module moves remain a separate approved migration.
  Historical archives and formal fixture bodies are never rewritten.

The central restructuring decision stays unchanged in this bounded slice.
This change alone does not close #6832 or the compiler fix-history gate.
