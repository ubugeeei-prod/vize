# Native JSX and TSX local navigation preview

Related issues: #6850, #6871 and #6883. This extends the optional native
source-navigation preview without changing standard LSP request selection.

## Genuine delivered prerequisites

PR #7476 actually merged through the protected queue on 2026-10-03 at
05:39:06 UTC as signed `104e8e87984655e06447845d2a9f16ea54ee38c1`.
Its source and fresh protected candidate independently passed the original
fifteen navigation laws, whole response tests, minimal feature compilation,
warning-denying feature Clippy, instruction counts and full recipe.

The current provider baseline is signed
`d36c4874739bd8e6e342c4d0aeb9ede1c8c6eb0b`, including genuine L2 JSX
reference facts from #7479/#7480. Its existing original-Program API supports
component reads, intrinsic tags, static member roots, expression containers,
fragments and spreads in the same bounded resolver walk. No private owning
Program or JSX body provider is copied into this consumer change.

## Language and semantic authority

`experimental-source-navigation` continues to register `vize/nativeDefinition`
and `vize/nativeReferences`. The actual Document language captured with the
immutable source snapshot selects the original module parse profile:

| Document language | Original Program profile |
| ----------------- | ------------------------ |
| `javascript`      | JS module, JSX disabled  |
| `typescript`      | TS module, JSX disabled  |
| `javascriptreact` | JS module, JSX enabled   |
| `typescriptreact` | TS module, JSX enabled   |

A URI extension, source resemblance, framework name or query argument cannot
change that profile. Unsupported languages remain explicit refusals. Plain
JS/TS never retries a refused input as JSX/TSX.

Each genuine physical snapshot is parsed once through L1. Its admitted
original Program and checked whole source block enter the existing L2
FileProducer. Structural construction is followed by `File.is_complete()`;
all original issues and interruptions survive refusal. Positive locations
come only from bindings and resolved occurrences in that same actual File.
No caller-supplied AST/File pair, external binding ID or legacy query is used.

Opening component identifiers and static member roots yield actual reference
locations. Static properties, intrinsic names, attributes and closing tags
do not acquire invented occurrences. Imports point to the current file's
local import binding; external definitions and property resolution remain
unfinished. Unsupported `this` tags, namespaces, generic arguments and TS
subtrees retain the provider's typed refusal.

## Reuse, coordinates and lifecycle

The original arena-owned Program and File coexist during synchronous
construction. Private consumer summaries retain only validated response rows
and the actual SourceSnapshot Arc. They are reused by both endpoints without
a query parse, new level stage or level serialization. Retaining all level
AST/artifacts across asynchronous requests remains unfinished in #6872.

Existing URI/global document revision/client version/language guards,
UTF-8/UTF-16 projection and cancellation remain mandatory. A real host
change, close/reopen or equal-version language change cannot publish an old
response or reuse its physical snapshot. This adapter does not apply host
edits or grant native Vue admission.

## New regression scope and delivery gate

Seven new project laws cover original JSX/TSX component/container locations,
Unicode and CRLF, genuine shadowed bindings, runtime fragment/spread reads,
actual language selection, complete typed refusals, foreign equal snapshots
and stale/cancelled lifecycle publication. Four new production-service laws
compare complete JSON-RPC Value envelopes for success, empty results,
refusals, language changes, close/reopen and pending request cancellation.
They do not claim serialized byte-format equivalence.

The existing shared full-recipe commands select all new modules:

```sh
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib source_project:: -- --nocapture
cargo test --locked -p vize_maestro --features experimental-source-navigation --lib server::native_navigation:: -- --nocapture
cargo check --locked -p vize_maestro --no-default-features --features experimental-source-navigation
cargo clippy --locked -p vize_maestro --features experimental-source-navigation --all-targets -- -D warnings
```

These eleven new functions are source-authored and unexecuted at preparation.
No earlier head's runtime acceptance transfers to them. Fresh exact-head
Actions, original full logs, unchanged instruction ceilings and protected
terminal merge are required. New PR publication is held while the existing
families drain; private source preparation does not count as delivery.

Whole JS/TS/JSX/TSX syntax and semantic coverage, workspace/external queries,
hover/type definitions, native Vue navigation, standard route migration,
all-level retention, atomic actual host edits and complete LSP fix-history
admission remain unfinished. #6850/#6871/#6883 stay open.
