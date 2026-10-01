# Checked shared expression emission

Paired issue: [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

`write_expression` consumes an actual L2 `ResolutionTable`, the authoritative
authored file and a separate framework `AccessProvider`. It copies decoded
source bytes and splices resolved identifier ranges directly into the shared
append-only `Writer`. It performs no parse, string search, alignment recovery,
JS AST generation or serialization between levels.

Preflight checks the authored source identity, every exact L2 coordinate
projection, framework access facts, runtime helper availability and generated
byte limits. Prepared access decisions are retained once. Failure preserves
the writer's text, links, helper set and indentation. Emission then appends
pieces directly; `push_named_parts` needs no temporary rewritten string.

Every rewritten accessor gets one named link to its original identifier.
Object shorthand expands its original key only when its value changes.
For the semantic shorthand name `__proto__`, including Unicode-escaped
spellings, a computed string key preserves an own data property instead of
creating the special colon-form prototype setter. This key participates in
the same atomic byte-length preflight and links to the complete authored
identifier. Ordinary shorthand spelling remains unchanged.
Unchanged references and copied text preserve their spans. The non-recording
writer emits identical bytes without retaining links.

`ResolvedExpressions` borrows source-ordered, uniquely owned L2 tables. A
target requests the actual retained `JsExpr`; lookup requires pointer-equal
AST and coordinate capability, matching source, and the authored span. Missing
tables and another parse of equal text are checked failures. L4 never repeats
name resolution. The four unfinished target input records use this real view.

The separate Vue accessor provider uses stable binding IDs and explicit script
facts. It keeps locals/globals, prefixes context/setup/props/data/options reads,
quotes prop aliases safely, preserves escaped identifier spelling and reads
inline refs through `.value` or the target vocabulary's `unref` helper. New
callee unwrapping is parenthesized to preserve constructor precedence. Ref and
maybe-ref writes retain the proper `.value` assignment/update target. Inline
`SetupLet` writes need parent/RHS lowering and are explicitly unsupported;
read-only binding writes are rejected. No global-name or ref heuristic is used.
The same compile-unit registry must supply L2 binding lookup and Vue access
facts; these caller-owned IDs are not a completed canonical file binder or
proof that an arbitrary foreign access table belongs to this artifact.
These spellings follow the
[Vue 3.5.35 compiler source](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/transforms/transformExpression.ts);
this is not complete Vue dialect/runtime parity.

Actual L1 preparation and once-parsed syntax feed the L2 coordinate bridge and
resolution provider in integration laws. The original AST pointers survive.
Unicode plus entity identifiers have exact generated/authored named spans;
equal-byte-length `&acE;` and multi-character `&fjlig;` expansions remain
indivisible, and a stale authored source is refused. This uses the real L1
decoder and its private checked maps, not hand-written L4 map fixtures.

Seven L2 resolver laws, thirteen new L4 laws and all 42 existing L4 unit tests pass
against actual whole-library source and pinned cached dependencies through
rustc, with no Cargo build. Direct narrow Clippy checks also pass for the
actual L2/L4 libraries and new integration targets under the workspace
panic/indexing policy. Native integration first used immutable L1 source
`90f6b1dd9562d6278582e72c19465cd8f092fe76`; the same three laws also pass with
the actual full carrier at `2c9d0ff1368f870acf1bf69106def33eebb13cf7`, rebuilt
against identical OXC/L0 metadata. Its real provider must be present in the
final integrated base before this consumer is published.

The actual emitted plain/escaped `__proto__` fixtures also execute in Node with
object, null and primitive values; own descriptors and prototypes match the
original shorthand. Named computed-key widths and a later-access refusal are
checked against actual L2 AST occurrences and recording/non-recording writers.
The ordinary/escaped fixtures compare complete emitted code; the entity law
compares the complete v3 map with exact UTF-16 columns and name indexes.
The six actual JavaScript executions use Node's `vm.compileFunction` in its
default current realm, preserving the original own-descriptor/prototype oracles
without implied-eval lint warnings or a suppression. The actual skeleton tool
ratchets the completed expression module's measured L4 count from five to four;
the four unfinished target modules and all instruction/allocation gates remain.
Source, generated inventory, exact-head Actions and
protected queue checks still gate delivery. Existing 100 instruction workloads
do not establish performance coverage for this new producer/consumer path.

Full file binding, source-language resolution, complete expression/handler/slot
grammar, TS erasure, every Vue dialect, DOM/SSR/Vapor/TS targets and product
integration remain unfinished. The current L1 parser admission is bounded;
the checked L2 handoff also refuses leading/trailing line comments because
raw expression emission can swallow generated delimiters. Correct native
handling must consume retained comment facts, not loosen the legacy guard.
No product dispatch changes, extra pipeline stages, fixture changes or numeric
budget changes are introduced. #6840 and all product fix-history gates remain
open; native product completion is not claimed.

## Actual inputs and outputs

The writer input is a complete real L2 resolution table, the authoritative
authored file and an access provider derived from the same compile-unit
binding registry. The native coordinate capability remains attached to the
table's retained AST. A target may instead use the checked borrowed
`ResolvedExpressions` view; it requires the exact requested AST, coordinates,
source and authored span and never repeats identifier resolution.

The output appends decoded expression bytes and selected identifier accessors
to the caller's shared writer. Recording output includes complete authored
named links, including the computed `__proto__` key; non-recording output has
identical bytes. Failure appends nothing and preserves links, helpers and
indentation. Neither successful output nor caller-owned numeric IDs prove
that an arbitrary foreign access table belongs to a canonical artifact.

TODO: adopt the real artifact-owned file binding registry when declaration
origins and lexical visibility exist, then derive L3 semantic/access facts
from that owner. The current explicit ContextOnly caller/test policy does not
replace this missing product provider. Source-faithful emission preserves the
complete prepared decoded source. The merged L1
[`prepare_vue_interpolation_in` provider](./2026-10-02-l1-vue-interpolation-source.md)
selects the authored ASCII-edge window and decodes text once before parsing;
L4 applies no trimming or new output window. The native hook and complete
module/map acceptance remain a separate unfinished integration slice.
