# L1 checked embedded source provider

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836), following
the [typed-embed design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929)
and its [exact entity projection decision](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5873677772).

`prepare_attribute_value` now returns `Result<EmbedSource, SourceError>`. It
prepares source independently of a grammar or an embedded syntax tree; an
invalid span does not manufacture an empty successful artifact. Ordinary
non-attribute text uses the checked `EmbedSource::authored` constructor and
retains its authored bytes without HTML interpretation.

Attribute preparation invokes the ordinary native `decode_one` provider in
attribute context on authored bytes. It retains every scalar of a reference,
and never scans decoded output again: `&amp;#40;` becomes `&#40;`, not `(`.
Unchanged values, including invalid or unknown references, borrow their exact
authored slice without arena allocation. Authored HTML spelling remains the
authoritative source for later edits.

Decoded text carries ordered, contiguous segments covering both complete
streams. Decoded coordinates are embed-relative and authored coordinates are
file-absolute. The constructor checks representable u32 lengths, UTF-8 spans,
nonempty segments, coverage and byte-equal Identity segments. Entity segments
are explicitly marked even when decoded and authored lengths match, as with
`&acE;`. Their semantic contents come from the native decoder rather than a
second decoding pass. Maps and segment constructors are private, preventing
consumers from pairing arbitrary text with unvalidated segments.

Exact `authored_span` preserves the earlier #6836 decision: a partial entity
selection or a zero-width point strictly inside its expansion returns
`PartialEntityBoundary`. This API is suitable for selecting rewrite bytes.
The separate `authored_covering_span` is a diagnostic policy change: partial
intersections and interior points conservatively highlight the whole authored
reference, because no authored coordinate inside an expanded scalar can be
invented. Diagnostic covering spans must not select rewrite bytes. Points at
an entity start or end remain exact points; contiguous neighboring segments
agree at shared endpoints. Reversed, out-of-range and non-UTF-8-boundary spans
are errors under both APIs.

This provider consumes an already implemented native L1 API. It introduces no
production lexer, parse stage, serialization, product route, language switch
or legacy normal/build dependency. The independent structural slice does not
resume the [held v-pre integration](./2026-09-29-l1-v-pre-resume.md). Publication
awaits source review and the root agent's audit of current provider ownership
and outstanding Stack delivery; open issue numbers alone do not block an
independently ready provider API.

TODO before typed embedded syntax is integrated:

- Retain actual JS/TS syntax trees, comments and typed holes in a consumer of
  this checked source, with corrected decoded and authored diagnostics.
- Resolve host language once per container and diagnose script/setup mismatch.
- Attach artifacts to existing L0 identities, without a competing allocator.
- Select shapes using real dialect policy and lexical diagnostics, and build
  composite shapes from real language pieces. Consume trees without reparsing.
- Complete native lexer and Vue v-pre ownership before production integration;
  preserve product fix-history gates and legacy byte-exact output meanwhile.

Twelve laws cover unchanged borrowing/allocation, native full-value and
once-only decoding, complete coverage, exact/covering distinction, equal-byte
entities, adjacent endpoints, UTF-8/range validation and overflow. Five map
laws initially passed under a lightweight rustc harness using the actual map
source and tests with a minimal Span shell. Exact-head Actions subsequently
passed all twelve native laws, crate compile, Clippy, wasm32-wasip2 default
and no-default-feature libraries, full Check and strict100. Fresh-main replay
after the actual directive-provider merge requires new exact-head Actions
before protected queue validation. No instruction ceilings change. The combined
L1 skeleton count is now zero, so its baseline entry is removed; eliminating
panicking skeletons does not complete the unfinished integration listed above.

The initial published [full Check](https://github.com/ubugeeei-prod/vize/actions/runs/36852526925)
failed its exact storage-inventory preflight: new source files were absent from
the ledger. Review records one arena map buffer and temporary L0 String (each
one import/two bound uses), plus the test-only owned UTF-8-boundary list (one
import/one bound use, analysis category). The existing file-based scanner also
counts external unit-test files; no classifier, storage types or gates change.
