# Static root component diagnostic ownership

Issue: [#6258](https://github.com/ubugeeei-prod/vize/issues/6258).

The standard Content Mapper backend tracks one `used` flag per expectation
interval. Bounding several parent ranges can swallow a child's error; splitting
one expectation into several tuples can produce false unused expectations. This
change keeps the existing protocol and groups one bounded class of parent checks
in generated code instead.

## Admitted owners

An existing direct root `ElementNode` must match its actual Croquis component
usage. Its component and every bare event-handler reference must have an actual
`SetupConst` binding. Every prop is a static attribute, with a non-key/ref prop
present. Other directives, dynamic events/props, spreads, explicit generics,
instance-global fallback and strict slot-child checking are excluded. A slot
navigation check whose source belongs to a child also excludes the owner.

The marker search is only an activation gate. The shared canonical directive
walker consumes the existing comment-retaining RootNode and admits only actual
Expect target ranges, including its existing precedence and orphan semantics.
Prose comments, text and attribute strings cannot authorize an owner packet.
There is no extra parse or second directive grammar.

The owner keeps its original component usage index and exact event scope spans.
Its event references and existing handler scopes are emitted once into a local
buffer. Per-usage declarations, navigation, prop checks and required-slot checks
complete that buffer at the existing late prop-check phase. Shared helpers are
emitted once. Appending translates every generated mapping/subspan and both ends
of every semantic link; authored ranges remain unchanged.

Static prop literals and bare immutable handler references do not depend on a
child's assignments. The existing emitter bodies remain responsible for generic
inference, handler signatures and function boundaries. There is no second parser,
parallel scope graph, backend change or new directive representation.

## Why dynamic owners retain their old layout

Moving a handler is not generally neutral to TypeScript flow analysis:

```ts
function render(value: string | number) {
  void (() => {
    value.toFixed();
  });
  value = 1;
}
```

The existing TypeScript 6.0.3 compiler reports TS2339 here. Moving that arrow after
`value = 1` removes the diagnostic because the later closure captures the final
assignment's narrowing. A dynamic template handler must therefore not enter this
packet merely because it has a root scope. Component event inference also emits
value-level calls with the authored props; those calls are not inert aliases.

Unknown names, dynamic props, structural scopes, generic directives and named
slots remain #6258 frontiers. This bounded change does not complete that issue or
prove the original `outer`/`child` reproduction fixed.

## Qualification

The authored unit controls compare the complete original-order and relocated
emissions: every generated line, every complete mapping/subspan payload and every
semantic link with its authored owner and full destination statement. They also
require old parent ranges to contain the child and the new owner interval to
exclude it. Unsupported inputs compare whole text, mapping vectors and semantic
link vectors byte for byte, including the closure mutation counterexample,
conditions, loops, slots, undefined handlers and missing setup bindings.

The standard tsgo CLI control retains 16 complete sources, stdout/stderr byte
streams, process outcomes and expected whole output before asserting all cases.
It covers independent prop and known-event handler repairs, one unused
expectation only after both are fixed, LF/CRLF and Unicode prefixes. `@missing`
is retained as a mapping stressor: the standard mapper's existing default
`checkUnknownEvents=false` is unchanged, so it is not counted as a second active
parent error. A wrong immutable handler on the known `save` event supplies that
independent parent error.

The optional `directive-owners-only` mode in the existing Content Mapper
workflow builds the exact pinned upstream backend once. In one disposable runner
checkout, it builds a genuine old producer with only the same integration tests
overlaid, retains every old observation, then restores the candidate and builds
it using the same warmed Cargo directory. It requires an actual 16-cell old RED,
16 complete candidate CLI outputs, four byte-identical no-directive calibration
outputs, 16 whole mapper transforms per operand, and four editor sessions with
44 complete reports per operand. Extra unmapped diagnostic aggregates fail.

Retired 2026-10-19 (#8840): the comparison was pinned to one old producer and
could no longer pass on main, so the mode and its `owner-*.mjs` drivers were
removed.

The mapper comparison retains every span's authored coordinates, generated
payload, kind and feature bits, both semantic-link endpoint payloads and every
generated line multiplicity. Candidate Canon controls additionally execute the
complete internal mapping/subspan and semantic-link assertions, exact original
fallbacks for unrelated markers and unsupported shapes, and the borrowed-root
policy equivalence with offsets. All observations and binaries upload before the
strict result gate; failed comparisons retain their own final receipt.

The candidate phase also runs the existing five stock LSP selectors with explicit
provider environment variables. Their complete original stdout/stderr, test lists,
source query sites and actual test counts are retained. Pure helper laws are
counted separately from backend-dependent tests; no invented "41 tests" count or
new stock wire capture is claimed. Neither default full/event commands nor their
provider pins, deadlines or SDK are changed.

The controls were initially authored without backend execution. The observed
run and corrected qualification boundary are recorded below; protected checks
and actual delivery remain required. Native or standard CLI ownership is not inferred from a range
unit test, a skipped backend test or a successful process with missing packets.

## Observed qualification and corrected boundary

The first exact-backend run [38989978044](https://github.com/ubugeeei-prod/vize/actions/runs/38989978044)
used old `8d3ca7bbc544ec1d8cf8e275cc758037c5289237` and candidate
`9d1e13d7053c89361f6a0e4a837964d3bd3fa795`, with unchanged upstream
`d6c4afddb2c55f4a9dea7b59293a99a8fdea1799`. It failed its original strict
qualification. Preserve that failure; a revised contract is not a retroactive
passing execution. Complete observations are in artifact `12084234305` (ZIP
SHA-256 `a8096542d10c9833b8a43ad685395d7bdca0d1e39e429ab3067fa1e642b9286b`).

All 16 old CLI cases actually lost the child; all 16 candidate CLI cases matched
the authored whole output. All wire spans, kinds/features and generated-line
multiplicities survived relocation. These fixtures serialize no semantic links;
the six executed Canon tests separately exercise nonempty internal link/subspan
controls. The five existing stock selectors actually ran 14 named Rust tests
(10 backend-dependent and four helper/fake-peer laws), with authenticated provider
aliases and no environment-based skips.

The no-directive calibration premise of one diagnostic per error intent was
wrong. The complete unchanged fallback counts are **6 / 2 / 5 / 1**, rather than
3 / 2 / 2 / 1. Bad static `id` is checked by two real event-inference calls (for
`save` and `missing`) at virtual lines 157 and 190, plus two mapped prop checks.
The inference errors are unmapped. The wrong known `save` handler and child each
supply their independent errors. Every calibration message, continuation, unmapped
note and ordering is frozen in full, and must remain byte-identical OLD/NEW.

Every editor report also contains two legitimate TS6133 severity-4 suggestions
for the empty handler parameters. The full expected packets explicitly retain
these source ranges, messages and severity; diagnostics are never filtered.
Across four newline/prefix sessions, states 1–8 provide **32 bounded owner/repair
reports**: full exact packets, child/unused behavior and no TS100036 aggregate.
States 0 (no directive), 9 (ignore) and 10 (skip) provide **12 known-failing
fallback reports**, not semantic acceptance. Freeze their complete OLD/NEW
packets, including leaked bad-id diagnostics and the real zero-range TS100036
aggregate. These inherited inference/mapping fronts remain open under #6258.

The result receipt therefore distinguishes `boundedOwnerAcceptance: 32` and
`unchangedKnownFrontiers: 12`, with `overall6258SemanticAcceptance: false`.
The original plan, raw failed receipt and observations remain evidence; only the
reviewed qualification expectations change. A further exact execution and normal
protected delivery are required before this bounded fix is accepted.
