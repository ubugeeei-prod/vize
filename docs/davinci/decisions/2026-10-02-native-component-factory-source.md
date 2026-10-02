# Source-owned native Component factories

Issues: #6836 and #6838. File-owned resolution remains dependent work.

`NativeComponent::parse_in` owns the genuine native `ComponentParse` paired
with its checked `SourceBlock` and allocator. Private fields and immutable
carrier access prevent a caller from substituting a foreign or modified tree.
`construct_in` consumes that owner and checks the factory's complete source
pointer and length before any construction or expression parse. Rejection
keeps the whole original owner in an ordinary cold box, available for a
source-correct retry. Equal bytes in a different allocation do not confer
admission, nor does a prefix sharing the same pointer with a different length.

Whole-file template construction uses the block's original source slices.
Surface error offsets and directive-admission spans are checked and rebased
during their existing loops; the original relative-coordinate observations
remain in the carrier. Token, operation, expression, diagnostic and provenance
spans are file-absolute. The sole retained-expression handoff still preserves
AST roots, children, comments, full diagnostics and once-decoded entity maps.

`ComponentFactory` is a monomorphized capability for source, checked leaf and
owner factories, named binding, retained `JsExpr`, and provenance. Its concrete
`ComponentBody::run` receives the factory's real borrowed child and minted
owner ID. This non-GAT callback accepts compile-local source and recorder
borrows without an implied-static higher-ranked requirement. Raw
`RegionBuilder` delegates to its existing checked methods; no factory mint,
owner phase, pending-frame guard, seal or layout is replaced.

The dialect table retains the same const function pointers, predicates and
lowering. Its generic const array is borrowed for iteration: a static slice
would require the captured source/factory types to outlive `static`, while a
source-lifetime slice would incorrectly extend the shorter recorder borrow.
Actual compiler failures and the short-borrow success law establish this
choice. The table allocates nothing and introduces no dynamic dispatch.

A public diagnostic factory cannot complete a file-resolution artifact. Only
an authoritative private file-construction route may resolve a real retained
expression before its own factory call and associate the resulting table at
that actual returned node ID. Caller-provided IDs, lookup tables and generic
callbacks do not confer completeness. The dependent file owner keeps the
recorder and raw region private. This slice implements the capability, not
that declaration registry or its resolution semantics.

Five meaningful law groups cover full-file Unicode/entity spans and identical
retained AST pointers; equal-byte and same-pointer/different-length rejection
before mint with retained-owner retry; every original error/admission fact and
rebased diagnostic; and recursive short-borrow recorder equivalence for actual
owner/expression IDs, complete tree output and provenance after ordinary
observations drop. All 40 actual native producer laws and 51 current L2 laws
(44 unit and seven resolver integration) pass; whole-current-L2 and
selected-native production Clippy pass.

Storage review moves four existing construction-observation buffer constructors
from the entry module into the block owner; its public observation contract adds
four field-type references without a new runtime buffer or stage. Recursive
callbacks reuse the existing directive arena list, with one field-type reference;
the trait's four arena paths are existing owner-argument types. Test observation
lists and strings stay explicitly inventoried. Every
new Rust file is below the unchanged 350-line limit. The classifier and all
pinned footprint/instruction/allocation ceilings are unchanged. This slice
adds no parse, AST walk, serialization, seal walk or product-route change.
The seven original ignored-head/entity regressions remain checked through
the same prepared-header early mode guard. Performance is unclaimed pending
fresh exact-head Actions and all-100 protected queue validation.

The proof compiles the whole actual L1 source with coherent cached dependency
metadata and the complete current L2 registration. It is scoped source evidence
rather than a whole fresh-L0/workspace build. The genuine source ancestry includes the
actually merged core/helper, Params/Dense/resolver providers on signed f634,
and the reviewed native opening-mode provider;
no provider is copied into this source prefix. Publication still requires
fresh exact-head Actions, native Stack
registration, protected queue checks and actual merges. Conditional and For
construction, HandlerBody resolution, complete file semantics, remaining
dialects and product migration remain unfinished.
