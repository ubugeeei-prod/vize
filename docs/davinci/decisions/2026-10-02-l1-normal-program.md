# Ordinary native JS/TS/JSX/TSX Program provider

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836), following
the [retained AST design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929).
This revises the initial Program admission scaffold in
[the retained syntax record](./2026-10-01-l1-native-embed-syntax.md).
It does not revise wrapped Expr, HandlerBody, SlotParams or DenseFor admission.

`embed::syntax::parse_program_once` takes validated `EmbedSource` and explicit
`ProgramOptions { lang, jsx, goal }`. The caller chooses JS or TS, whether JSX
is admitted, and Module or Script before parsing. The exact OXC `SourceType`
is retained on `NativeSyntax`, including syntax/admission holes. Filename,
container language resolution and formatter profiles are not inferred here.
Existing `parse_once` dispatches Program to the same explicit Module provider.

The provider checks OXC's byte-length boundary before a single ordinary pinned
`Parser::parse`. It parses the original text without a generated wrapper,
Unambiguous retry, semantic pass, serialization or legacy Vize dependency.
Real OXC Program nodes, source, comments and full normally owned diagnostics
are retained together. Syntax and unsupported Flow remain local typed holes;
a hole hides the recovery AST from successful views while retaining its
observations. Exact AST edit locations and diagnostic covering keep the
existing once-decoded entity/UTF-8 projection contracts.

The initial 31 conservative word/punctuation unit limit is removed only for
Program. That scaffold bounded broad parser recursion by rejecting ordinary
valid programs; it was an initial API limitation, not a requested universal
resource-quota contract. Normal whole-Program support may reuse the mandated
production OXC language library. OXC syntax success does not establish semantic
validity, hostile-input memory/fuel/cancellation quotas or complete product
integration. Actual demonstrated parser defects require responsible fixes.

## Concrete PURE parser defect

The ordinary Program regression `f<>/((\nd=//#__PURE__0` reproduces an assertion
failure in pinned OXC `fc702c1` before this provider can return diagnostics.
The same annotation is marked `PureNotApplied` twice during error recovery.
Expanded native observations also reproduce duplicate-comment rewind pointing
at the final ordinary comment for `(/*#__PURE__*/\n/* other */\nf`.

Official [upstream PR #24161](https://github.com/oxc-project/oxc/pull/24161)
documents the idempotency cause. It was superseded by
[d5163d0f](https://github.com/oxc-project/oxc/commit/d5163d0faac7c936796dd5c0164d58ab5eae92ea),
whose exact-span duplicate-index lookup also repairs the rewind defect.
That complete upstream change rewrites annotation application more broadly.
The private reviewed candidate instead accepts the already-not-applied
terminal state and recovers the matching authored comment index. No comment
skip, input rewrite or panic catch belongs in native admission.

Dependency integration is pending root review. The initial provider with the
unchanged original debug parser is known red on the actual PURE regression.
Private parser proof is not a repository dependency repair or merge proof.

## Validation and remaining work

Nine focused laws inspect actual JS imports/functions/loops/exports, TS
interfaces/generics/return types, JSX elements/expression containers and TSX
typed arrows/children beyond the old scaffold. They verify explicit goals,
once-decoded entity maps, every malformed UTF-8 prefix, owned diagnostics and
the real PURE recovery/comment owner. Wrapped-shape laws remain unchanged.

The real current L1 source compiles against pinned cached dependencies. Before
the ninth added regression, all 253 unit laws passed with the private corrected
debug parser; all nine focused Program laws subsequently passed. Strict library
Clippy passes. The parser's 16,496 fixed input/profile observations pass in both
debug and optimized release. Complete original/fixed release captures localize
256 differences to corrected comment classification, with unchanged AST
structure/flags, spans, diagnostics and parse status in that bounded matrix.
These are scoped source checks; fresh exact-head Actions is still required.

The `l1_program` fuzz target calls this provider once in an explicit profile
selected by one corpus byte. It observes real source, comments, diagnostics,
covering labels and Program identity, and drops owned observations normally.
Fixture script blocks and committed valid/malformed/PURE seeds run in all
eight JS/TS/JSX/TSX Module/Script profiles. It has no expression guard or PURE
skip. Replay and campaign evidence remain pending; existing fuzz limits and
instruction ceilings are unchanged.

TODO:

- Integrate the reviewed narrow pinned-parser correction with license/source
  provenance, shared supporting OXC identity, real regression corpus and Actions.
- Add the dependent explicit standalone script dump/inspection consumer.
- Retire wrapped-shape admission only with its own actual context/lifecycle proof.
- Resolve file/container languages and script/setup mismatch, attach artifacts
  to existing identities and connect native markup/container consumers.
- Complete lossless typed script surface recovery and native product routes.
- Measure parser/consumer deep-input risks separately; hard resource quotas,
  bounded iterative consumers and upstream defenses are hardening follow-ups.

The issue remains open. Full Vue/dialect, JS/TS/JSX/TSX end-to-end product
completion is not established by this standalone Program provider.

The five frozen native Program fuzz sources use opaque `.input` filenames.
The seeder reads every file in this directory and prefixes each unchanged
source with all eight explicit profile selectors; filenames do not choose
the language. The scripted move preserves all five inputs byte for byte,
including malformed parser regressions, and adds no formatter exclusion.
