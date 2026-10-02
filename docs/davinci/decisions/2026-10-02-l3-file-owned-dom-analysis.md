# File-owned native DOM analysis (2026-10-02)

Issues: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839) and
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

## Decision

`build_dom_file_decisions` accepts the sole immutable `FileArtifact`. It derives
the canonical artifact from that owner and attaches its DOM scratch to the
existing `NodeEvent` traversal. There is no additional AST/tree walk, numbering
stage, serialization or expression parse. Ordinary bare-artifact DOM and
conditional analysis keep their existing behavior.

For interpolation and static-name bindings, the actual canonical event's node
selects the private factory `FileResolution` row. Admission checks the original
AST pointer, complete decoded-source pointer, authored span and coordinate
capability pointer against that row. It checks the recorded lexical scope and
the already-resolved Value declarations' identities/names. It does not repeat
scope/name resolution or create declarations from free identifier names.

Each admitted row retains the actual `FileResolution` beside DOM facts. Numeric
query keys are local indices; equal foreign NodeIds do not authenticate an
owner. The entry obtains every queried ID from its sole file's canonical events
and exposes no API associating arbitrary tables or artifacts with that file.

Retained literal ASTs receive `LiteralConstant` only after their factory row is
checked. Complete nonliteral expressions require at least one actual reference
and receive neutral `FileDependent` value semantics. Those semantics can mark
text/property/class/style changes. They establish no purity, Vue exposure,
runtime access spelling, ref unwrapping or constant-initializer propagation.
Complete zero-reference nonliterals are explicitly unsupported.

## Retained owner and target boundary

`NativeFileAnalysis` retains its sole file borrow and a private native result.
Its read-only artifact, tables and DOM views are derived from that result and
file. It has no owner-discarding conversion to `NativeAnalysis`. Compile-fail
laws prohibit extracting that private result, discarding the live file, and
passing it to a bare-artifact consumer.

This distinct entry/result prevents an emitter using caller-declared
`ContextOnly` identities from silently interpreting script declaration identity
as `_ctx`. Actual Vue runtime exposure/access capability is a separate required
provider. The file entry never calls a caller-supplied expression-facts trait or
uses a missing-name context/global fallback.

An incomplete file fails before traversal. Missing or mismatched expression,
scope and declaration rows are typed DOM refusals at the original authored
location. No partial scratch tables are returned after traversal failures.

## Existing conditional facts and held control integration

The owned ordinary DOM producer and corrected conditional source are composed
without changing their recorded helper-demand order or branch ownership.
Semantic dependency roles contain no runtime helper names or numeric Vue bits.
Conditional grouping still follows genuine header order/direct root counts;
equal branch spans are not treated as branch identities. Fallback demand occurs
when the first actual branch root leaves, before later branch demands.

File conditional conditions do not become `ContextDependent` by association.
File If/For admission remains unsupported until genuine control factories record
their complete owned condition/collection/body scope and declaration facts.
For additionally requires original Dense no-hole/context authority, active
attribute membership, actual Template-origin BindingIds and a checked retained
formal-writing/access contract. Raw JsBinding values or copied coordinate
bytes cannot supply those capabilities.

## Validation and delivery scope

Six actual admitted Program → diagnostic Component/File factory → L3 laws
cover dynamic interpolation/bindings, original entity AST/coordinates, nested
script-unit shadowing, incomplete child-local refusal, zero-reference
nonliteral refusal, and equal numeric IDs in distinct owners. They use the
real lower `FileProducer`, `ProgramScope` and `TemplateScope` APIs. Diagnostic
construction does not certify Vue script roles, original template custody or
native SFC admission. Full L3 rustdoc checks include the three new
ownership/type-boundary compile-fail cases.

Local evidence compiles the complete owned L3 source against the genuine first
File provider's unchanged L2 source and coherent retained Shared L1/OXC/L0 and
selected Pending native modules. Production and test fixtures use the
workspace's strict Clippy restrictions. This is bounded source/API evidence;
exact-head hosted Cargo, portable targets and unchanged instruction gates
remain mandatory before delivery.

The publication parent is genuine File
`6e5b4561e8acedef0c05592129fe81691815f5ce`, following Pending
`ff28296247bcd7a73d7421d462b99d84b2f259fa` and actual merged Shared
`315a31732b2d70e5cf5dae33ce5e5f9c13f8b7b3`. Only reviewed owned L3/edge
changes are replayed; no research provider/control ancestry is copied. Neutral
conditional-frame unit laws construct checked canonical parts through the
existing `Artifact::try_new`, rather than calling an unpublished factory.
Native Stack membership, exact-head Actions and actual queue merge are still
required. No legacy normal/build dependency or product route is introduced.

## TODO

- Deliver the genuine File/Program/component providers on accepted main and
  revalidate the same source through exact-head Actions and the protected queue.
- Consume an actual same-file Vue runtime exposure/access capability in L4;
  verify complete generated modules, execution and authored maps.
- Integrate actual owned If/For file control factories in the same traversal;
  preserve the existing For counterexamples and refuse unsupported formals.
- Complete other Vue dialects, JSX/TSX, full JS/TS semantics, hoist/cache and
  product replacement only after the existing compiler fix-history gate.

Neither #6839 nor #6840 is complete, and no production route changes here.
