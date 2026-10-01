# Native canonical L2 ownership

Tracked in [#6838](https://github.com/ubugeeei-prod/vize/issues/6838), paired
with the L1→L2/L2 section of the central decision record. This independently
reviewable provider slice implements ownership over the native op family.
It does not complete #6836, bypass its consumer prerequisite, or replace a
product's legacy route.

## Owned fields and retained expressions

`vize_l2::artifact::Artifact` consumes `ArtifactParts`: the complete authored
source, native `Region`, producer-ordered `ProvenanceRecord` rows, and the
sparse `ScopeFacts` table. It derives its node count instead of accepting a
producer's unchecked count. The sealed artifact exposes immutable borrows;
consuming `into_parts` releases the seal so a producer can update the tree
and its facts together before checking them again.

Every existing `ExprRef` stays in place. In particular, a retained `JsExpr`
and its OXC expression AST keep their original pointers, source and spans.
Construction performs no parsing, expression rewriting, owned dump conversion,
or serialization. Decoded/synthesized expression source may differ from its
authored source slice. JS root coordinates must index the expression's own
source; authored coordinates must index the artifact's full source.

This does **not** adapt the unpublished L1 wrapped-expression provider. Its
coordinate-aware parsed carrier needs an explicit future handoff; copying it
into the current source-relative `JsExpr` would not establish that contract.
Identifier resolution, full binding declaration enumeration and broad native
embed admission remain unfinished.

## One shared numbered traversal

The existing `PageWalk` moves unchanged in a move-only commit from L1→L2 to
L2. The integration commit changes its crate-local import, exposes it, and
keeps a conversion reexport so existing mutable passes use the same provider.
`mint_attached` assigns attached binding ids without incrementing region-op
visits. `visit_ops` preserves its existing mutable traversal and accounting.

The borrowed `visit_nodes` and `visit_events` surface uses that same numbering:
owner op, attached bindings, then owned regions. Conditional branches remain
in authored order; branch and attribute rows have no ids. Each enter/leave
pair carries the same id, node and parent. An enter event also carries its
immediate source owner span, including an unnumbered conditional branch.
Bottom-up consumers can therefore aggregate children in one walk without a
second id allocator, pointer-to-id map or target-dependent artifact.

Unsealed traversal reports `NodeLimit` before dispatching an unnumbered
node. A successfully constructed artifact cannot reach that state. `NodeId`
remains stage-local and contains no artifact generation: downstream facts
must retain their artifact borrow and cannot be reused after page-order
mutation or across another artifact with coincidentally equal indices.

## Checked construction and failed results

The constructor checks and counts the native tree in one traversal. It checks
file bounds and UTF-8 boundaries for node, attribute, expression, branch and
authored scope-binding spans; nested spans stay inside their immediate owner.
Filter base/segment authored ranges are checked as well. Canonical conditionals
have a conditional leading branch and at most one trailing unconditional
branch. Retained JS root spans must index their own expression source.

Every iteration and slot-props introduction has a scope identity, including
sites whose opaque/pattern positions enumerate no names yet. Scope keys must
name actual introduction sites; tags form a unique dense set. Provenance and
scope keys must resolve in the exact numbered tree. Failed provenance rows
without a produced node remain in producer order. On rejection the error owns
all original parts, preserving partial nodes, scope facts and decisions for
repair or diagnostics.

This is creation-time provider validation, not a new pipeline stage or a
release verifier. The transitional `Lowered` and its compatibility tables do
not adopt this owner in this provider slice. The dependent conversion consumer
must preserve its existing fused producer/pass walks and budgets rather than
adding a second tree walk merely to wrap the result.
Conversion ownership adoption is deferred until a fused checked
producer/builder can seal these fields within those existing walks. An
intermediate child that only transfers ownership into unsealed mutable parts
is deferred alongside it.

## Validation and remaining work

Twelve native laws cover original JS/AST pointer identity; exact ids 0–10
through attached bindings, branches, iteration, slot fallback and components;
agreement with the mutable pass's ids and visit accounting; paired nested
enter/leave ids and parents; UTF-8/source/owner spans; canonical conditionals;
scope uniqueness and missing/dangling keys; retained-source JS coordinates;
exhaustion; and preservation of failed provenance after invalidation.

The laws passed in a small Rust harness using byte-exact copies of the actual
provider/op/expression/scope/provenance modules and compatible cached native
OXC/L0 libraries. This avoids a heavy local Cargo build. It is not a current
workspace build: the cached L0 predates relocated dump/stage APIs. Fresh-head
Actions must verify the whole crate, conversion reexport and strict Clippy.
Replay, formatting and whitespace checks pass. Independent source review
found no remaining concrete blocker.

The exact storage inventory reviews the artifact's one owned provenance
vector (one direct import and one bound use) as contract storage. The separate
test module's measured allocation/string/arena-vector rows cover its fixture
and traversal collectors; registration under `cfg(test)` does not make the
per-file scanner omit that file. The strict storage policy passes with these
exact rows; no storage ceiling or benchmark budget is increased.

Still unfinished: actual conversion ownership adoption; the once-parsed L1
embed handoff; resolution/declaration production; neutral analyses and pass
moves; dialect compatibility legalization; product fixture gates and routes.

## Publication gate repair

The first exact published provider head (`38b9967`) failed script formatting
and the generated Croquis ledger gate. Its strict 100-benchmark run measured
98 benchmarks within their unchanged ceilings; DOM compile medium and
stress-deep exceeded their ceilings by 21,516 and 235,101 instructions.
The canonical Stack remains outside the queue until a repaired exact head
passes all required gates.

The repair formats the actual replay script, regenerates the two affected
Croquis shards from the current source, and marks the small PageWalk counter
primitives/accounting helper inline across the newly introduced crate
boundary. A separate-crate optimized Rust 1.98.0 AArch64 caller produced
identical assembly before and after the hints, so that small caller does not
establish the cause or fix of the real ci-opt ThinLTO regression. Fresh
strict instruction Actions must establish whether the actual measured paths
are repaired. No benchmark identity or instruction ceiling changes.
