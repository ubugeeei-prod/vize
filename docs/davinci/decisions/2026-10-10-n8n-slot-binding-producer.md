# Retain slot binding grammar at its producer

Status: independently reviewed producer boundary for #8142; implementation and
whole-source qualification are in progress. Public installed acceptance is still
required. Audited preceding source: `d3e5b5fd1c565bf46780cc5a8d583663b049e73a`.
The preceding dynamic-name and configured-modifier slices remain separately
qualified source history; their shorthand-default refusal remains explicit.

## The missing grammar

`v-slot:[slot]="{ slot = fallback }"` declares `slot`; `fallback` is a read,
not a declaration. OXC's expression parser rejects this cover grammar after
storing the initializer in `state.cover_initialized_name`. Its object-expression
property contains an identifier, while the initializer is held in private parser
state. Suppressing the unfinished error would discard the initializer and make
`JsExpression { ast, raw }` lie about its raw bytes.

Armature's sole retained parse producer is `parser/expression.rs`, invoked for
slot values in `parser/attribute.rs`. It currently demands one complete TypeScript
expression and rejects trailing non-trivia. OXC's actual binding grammar already
represents object/array/rest/defaults as `BindingPattern`, including declaration
spans and default RHS expressions. Its `FormalParameters` retains those patterns
plus top-level initializers, rest, optional flags, and TypeScript annotations.
The real Vue language tools fixture `tsc/#2554/main.vue` has
`v-slot="{ foo }: any"`; Atelier currently accepts a complete parameter list.
A lone BindingPattern would regress that actual boundary. Use the existing
formal-parameter grammar directly on the original
raw slice, without `let ... = x`, arrow-function wrappers, a program parse, a
retry, string scanning, or a full Croquis analysis for a syntax-only rule.

## One role-aware retained carrier

The types belong to Relief's existing expression module:

```rust
pub struct RetainedJsAst<'a> {
    pub ast: &'a RetainedJsAstKind<'a>,
    pub raw: &'a str,
}
pub enum RetainedJsAstKind<'a> {
    Expression(oxc_ast::ast::Expression<'a>),
    SlotBindings(Result<oxc_ast::ast::FormalParameters<'a>, &'a oxc_diagnostics::Diagnostics>),
}
```

`SimpleExpressionNode::js_ast` becomes `Option<RetainedJsAst>`. The arena owns the
kind and payload in one allocation replacing the current allocated Expression.
Successful parameter trees and ordinary expressions are Drop-free arena values.
Diagnostics own heap messages and require destruction, so a failed parameter
goal parks its complete owned Diagnostics through the existing
`Allocator::alloc_owned`; the kind holds only the resulting compile-lifetime
borrow. Do not place the diagnostic owner inside OXC's non-dropping arena.
The copied carrier remains one non-null reference plus the exact raw slice;
verify its 24-byte Option and the existing 88-byte node gate, and measure enum
arena cost rather than claiming zero growth. Do not add a second per-node
carrier, store semantics in `identifiers`, tag pointers, or create self-references.

Both references borrow the same compile Allocator: its OXC pool owns kind/tree
bytes and its owned parking owns each error value. `reset(&mut self)` first clears
owned parking, running every diagnostic destructor, then resets the OXC arena;
Rust excludes all retained borrows before that reset. Dropping the Allocator also
drops the parking. No diagnostics, tree references or carrier cross a compile or
enter an owned cache. Success adds no owned parking allocation. Errors add one
parked Box plus their complete diagnostic heap data; the arena's allocated-byte
counter excludes that heap and must not be reported as the complete error cost.
The existing L0 reset/drop witness tests establish the parking destructor law.
Assert that the kind remains non-Drop, and measure error parking separately.

Keep public `JsExpression { ast: &Expression, raw }` unchanged as an expression
view. The carrier supplies `as_expression() -> Option<JsExpression>` and a typed
slot accessor that exposes either the complete FormalParameters or its retained
parser diagnostics. Expression consumers cannot mistake a slot pattern for an
expression. Slot failure diagnostics are retained for the parameter consumer;
no second parse is needed to explain a failed slot pattern. Unsafe-depth refusal
retains a slot error role with no invented diagnostic, so a later consumer cannot
fall back to its raw-text parser; it never attempts OXC. Other guard refusal keeps
the existing delimiter diagnostic. Ordinary expression failures keep the
existing absent-AST contract and existing diagnostics unchanged.

Select the role at the attribute producer using `dir_node.name == "slot"` for
its value only. Dynamic arguments remain Expressions. One guard and one profiler
increment precede exactly one role-selected OXC attempt. Both grammars must pass
the same complete-raw trailing-trivia check; slot failures retain diagnostics
without granting an AST. Decoded attributes keep the original decoded arena raw
slice, not reconstructed node content. Byte-identical clones copy the carrier;
any transformed bytes invalidate the complete carrier.

Add a narrow vendored `Parser::parse_slot_parameters` public entry alongside
`parse_expression`: dispatch the existing parser configurations, initialize once,
and reuse the existing formal-parameter grammar in `js/function.rs` with an
explicit EOF closing token instead of a synthetic parenthesis. Preserve the
ordinary parenthesized entry byte-for-byte, including its trailing-comma/rest
and TypeScript rules. The new goal uses ArrowFormalParameters semantics, rejects
arrow-invalid `this` and decorators, finalizes fatal/lexer/unfinished errors, and
returns the typed parameters. It must reject leftover tokens at the
producer, invalid methods/accessors, invalid rest placement, and invalid targets.
The ordinary expression entry and its errors remain unchanged. No upstream
repository write is involved.

## Public API and actual consumers

These types are publicly re-exported by Relief, Armature, and Atelier Core;
this is a Rust source compatibility change, not an internal implementation
claim. All five repository `JsExpression` literals keep their public fields.
The actual node field type changes, so external `.js_ast` consumers must choose
a role. The public Atelier `retained_whole_expression` helper currently returns
`Option<&JsExpression>` and would return the copied `Option<JsExpression>` view;
that signature change must be explicitly reviewed and released in a new minor.
Relief currently has no direct `oxc_diagnostics` dependency; retaining that existing
locked diagnostic type requires a narrow direct dependency declaration, not an
upgrade. The frozen 0.440 publisher completed with its verified public tuple. Explicit
publication clearance now permits only this already-locked direct dependency
edge; no version or broad dependency update is included. This source belongs to
a later minor and supplies no 0.440 installed acceptance.

The audited checkout has 31 `.js_ast` occurrences in 22 Rust files, five
`JsExpression` literals in four files, and 15 `SimpleExpressionNode` literals in
seven files. The implementation must cover these actual boundaries:

| Consumer                                                                                        | Required migration                                                                                                                                                                                                                               |
| ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Relief construction and `from_node`                                                             | Preserve empty initialization and byte-identical clone custody; keep the 88-byte gate.                                                                                                                                                           |
| Armature retention/profiler/dynamic-argument tests                                              | Select grammar once; preserve all old ordinary-expression laws and complete raw spans.                                                                                                                                                           |
| Atelier Core `retained.rs`, expression/shape/codegen helpers                                    | Take the copied Expression view; preserve module-dialect gates and all current byte-exact outputs.                                                                                                                                               |
| Atelier Core `steps/expression.rs` parameter handling                                           | Consume retained SlotBindings parameter items/rest/annotations and their original default RHS/spans; do not route the new carrier through `parse_as_params` or `rewrite_reparsed`. Retain invalid-pattern diagnostics without parsing again.     |
| Atelier `codegen/slots/params.rs`, `generate.rs` and `create_slots.rs`                          | Replace the `(source) => null` default-prefix wrapper parse and string declaration extraction with the retained FormalParameters. Preserve the existing lexical `_ctx` visitor and whole default-prefix packets using original zero-based spans. |
| Atelier `steps/v_slot.rs` declaration scopes and Patina `visitor_scope.rs`                      | Consume retained item/rest declarations only; retain raw public destructuring APIs solely for their existing v-for alias route, with no v-slot reparse fallback.                                                                                 |
| Atelier node literals in inline handlers, hoisting, lane/context/structural/element and codegen | Copy a carrier only for unchanged bytes; invalidate generated content as before.                                                                                                                                                                 |
| Vapor `l3/retained.rs`, private native `Expr`, assignment and model consumers                   | Carry the role-aware allocation through existing structural AST copies so writing `node.js_ast` does not require a second wrapper allocation; existing local JsExpression views stay views.                                                      |
| Croquis identifiers/unused facts/template ids/second pass                                       | Accept only Expression views where read expressions are demanded.                                                                                                                                                                                |
| Croquis slot first pass and `components.rs::push_slot_usage`                                    | Walk the already-retained parameter item patterns/rest for declarations and original offsets; remove both v-slot wrapper reparses on this route.                                                                                                 |
| Patina `valid_v_slot/dynamic_binding.rs`                                                        | Walk parameter item/rest BindingPattern declarations and retained argument lexical reads; exclude property keys/default RHS/enclosing bindings.                                                                                                  |
| Patina `no_v_html`, Canon dynamic-component resolution and Vapor key decisions                  | Demand an Expression view; preserve complete-raw refusal and original outputs.                                                                                                                                                                   |

Croquis's publicly reachable raw `extract_slot_props` and
`extract_slot_prop_bindings` functions and the Vue 2 `slot-scope`/`scope` attribute
path are existing separate APIs. Their existing contracts remain recorded; they
cannot be a fallback for new v-slot consumers. This proposal does not replace a
product's Davinci route or waive its original fix-history gate. Existing compiler
parameter reparsing is a real obligation to retire for retained v-slot input,
not an excuse to call the syntax-rule work complete early.

Vapor's compound-expression route needs the owned admitted Expression before
allocating the new kind. L2 therefore exposes additive `JsExpr::parse_ast_in`,
sharing the exact previous guard, TypeScript parse and complete-tail admission
with `parse_in`. `parse_in` delegates and retains its original payload/layout;
Vapor allocates the returned tree directly in one kind allocation. No extra
temporary expression wrapper, duplicated admission logic or dependency edge from
Davinci to a legacy crate is introduced.

## Required evidence and remaining boundary review

The public field/helper signature changes, single allocation/layout, retained
error ownership and full parameter consumer boundary received independent
review before implementation. Parser diagnostics must preserve current whole compiler
packets; if the direct goal produces different diagnostic text, declare and
independently qualify the exact intended addition instead of silently weakening
an old packet. No rule consumer may request or emulate a missing producer fact.

Append independent whole-source observations for shorthand defaults, nested
alias/default/rest, computed property keys, nested default-function lexical
scopes, invalid methods/accessors/rest, Unicode/escaped bindings, entity-decoded
values, typed/multiple/optional/top-level-rest parameters, CRLF, trailing comments,
incomplete raw, and guard refusals. Preserve the actual #2554 typed slot and all
existing `codegen/slots/tests.rs` whole default-prefix outcomes. Preserve the
original 47/16/current false/true controls and raw packet history; additions and
removal laws must make the changed refusal visible. Test both allowModifiers
values and disabled/unselected rules through complete CLI and JSON-RPC packets.

Require exact-head full Rust/compiler/Canonical/Croquis/Vapor/n8n Actions,
unchanged protected instruction budgets, actual native Stack merge, a genuinely
including public minor source tuple, and all installed CLI/LSP observations.
Measure the same parse counters, declarations and compiler outputs before/after:
one producer attempt per supported non-static node and no consumer v-slot
reparse. Full51/monorepo/type-aware/compiler/performance requirements and #8142
remain open until their own complete original evidence succeeds.

## Actual bounded implementation observations

The isolated Armature law passes with the actual 64-bit kind size 48,
Option 24 and node 88; the kind, parameters and carrier are non-Drop. Existing
error-only owned parking measured two heap calls, peak 88 bytes and zero arena
bytes for the parking itself. Reset released 264 bytes including the full
diagnostic owner and its contents. Empty depth refusal also parks its retained
error role in two heap calls/peak 88 plus the single kind allocation. These are
measured error costs; successful retention has no owned diagnostic parking.
The retained compiler slot helper collects binding names once and supplies both
the sorted callback-scope names and the existing default-RHS visitor from that
collection. Its raw guards, grammar, visitor policy and counter windows remain
unchanged; fresh instruction and whole-packet gates qualify the correction.
The four Croquis scope/usage refusal controls and the new 96 whole Patina
packets pass locally, preserving all 13 original input hashes. Local evidence
never substitutes for the full source/protected and installed campaigns.

Two newly authored argument-lambda cases exposed the native L1 header lexer
cutting `=>` at its HTML attribute assignment boundary. It now recognizes only
an adjacent arrow outside a quoted literal as a complete token. All other
HTML name boundaries, the existing iterative delimiter stack and unfinished
argument recovery remain. No later-bracket search, JS parse or retry is added;
the retained Expression producer still admits only complete grammar. Existing
missing-`]` vectors and new arrow-bearing unfinished headers remain mandatory.

The new independent campaign retains 96 exact pinned official observations and
245 separately authored whole source/installed consumer vectors (147 CLI,98
JSON-RPC). Two actual reference differences are explicit: the pinned Vue rule
misses a lambda's captured slot binding, and recovers facts from a malformed
rest header that this complete producer refuses. Raw provider responses and
authored contracts remain separate; equal aggregate counts establish no
semantic equivalence. Entity-decoded declaration coordinates retain their
existing decoded-relative offset contract; a physical entity map remains an
unfinished producer obligation. No full 51/monorepo/#8142 completion follows.

The bounded arrow-token correction changes two original 47 packets: the shadow
control becomes clean and the capturing argument gains the existing complete
own-binding diagnostic. `legacy-header-changes.json` retains all four complete
historical parser errors and the independently authored new directive finding;
applying only these declared edits supplies current packets, and reversing
them reconstructs the entire original before/after reports. All original source,
provider and packet bytes remain unchanged. The original 47 API tests pass
locally twice for both options with these exact inverse laws; the original 16
and 12 option cohorts also pass unchanged. The nine existing compiler slot
default-prefix tests pass without changing their expected code.

Source `0b20325a` runs the complete new 147 CLI and 98 JSON-RPC campaign,
original 125 configured-policy campaign, original compiler before/after and
navigation/type laws successfully in Actions 38036841187. Its complete source
check separately requires generated consumer inventories, normal formatting,
the explicit cost range comparisons and splitting the existing Croquis
component-reference helper from the oversized second-pass file. That helper's
ordinary-expression body and differential contract stay unchanged.

The unchanged SSR/Vapor large ladder also measures actual legacy consumer
re-parses falling 17→1 and 8→0 after the retained slot consumers remove their
eight pattern parses and SSR's eight parameter validations. Preserve the
original floors and fixture bytes, subtract exactly those named removals,
and record a separate complete current table in the original baseline document.
Both isolated six-fixture floor binaries then pass; Vapor's existing computed
name and dynamic directive controls remain in the same recorder. These are
consumer parse counters, separately from the one role-selected producer attempt
and the measured retained arena/error parking. Fresh full/protected source
qualification and a genuinely including public minor remain mandatory.

Independent review of the actual `a80191df` n8n artifact found that the old
188 whole parameter packets were written under Cargo's package working
directory, outside the existing workspace upload root. Anchor that writer
to `CARGO_MANIFEST_DIR/../../target/differential`, matching the new 96 writer.
All sources, expected packets, inverse-history laws and counts stay unchanged.
The complete original 188 upload is mandatory on the fresh successor; the
previous successful assertions alone do not qualify artifact retention.

Actual source 31d21 retains all 188 original and 96 new whole packets in
Actions 38039867839, including the unchanged 125 and new 245 CLI/editor vectors.
Its full Check38039867874 separately refuses a real canonical result mismatch:
three original TS-annotated slot patterns bind names through FormalParameters
in legacy codegen, while L2's older text-only cache guard misses those names
and caches their captured handlers. Keep all original files and errors; do not
rerun or relax the canonical oracle. Read the already-populated transform scope
alongside the two existing codegen param stacks, with no extra parsing, state,
allocation or carrier reinterpretation. Temporary event scope is popped before
all cache decisions. Six independent complete inline SFC controls cover typed
aliases, multiple names, tuples, defaults, a binding named `$event`, and outside
or unscoped handlers. Capture whole results and actual producer bytes before
qualification; original canonical 3, all old controls and strict 104 stay required.

The exact 943702→a801 paired entity probe also establishes a deliberate bounded
legacy improvement. In `<Panel v-slot="&#32;{ item }">{{ item }}</Panel>`, both
source states declare `item` at 18 versus independently authored physical 22:
the four-byte coordinate defect is preexisting. Selected Function/Module output
is unchanged and still raises an actual SyntaxError. Legacy Function/Module now
emit the retained decoded space and pass actual initial/update renderer laws;
all four literal-space controls remain whole-byte identical and successful.
The additive slot-parameter-entities fixture preserves the entire parent and
current code/preamble/map/error packets separately, plus independently authored
runtime expectations. Its new exact-source compiler judge captures all eight
whole packets before asserting their explicitly bounded goldens. Scratch actual
runtime evidence has comparison SHA 61a9071e7fcbf834e5f9338d904dd088f78c4ce83eb489a5d91503cb27f80dd6.
The compiler test now invokes the existing checksum-pinned full Vue loader and
executes all eight preserved historical modules and all eight actual modules.
Whole evaluation bytes, errors, renderer calls, initial/update trees and actual
Node/native hashes and process status are retained before the sixteen authored
laws. Historical modules are not described as a fresh baseline Rust rebuild.
The original default-slot-loop runtime judge remains unchanged and passes beside
the connected entity judge; fresh exact-head Actions remain mandatory.
No physical-coordinate, selected entity, universal compiler-byte, installed
producer, full 51/#8142 or release completion credit is granted by this slice.

An independently built original 943 source classifies the typed closure history:
both original lanes retain stale callbacks for alias, multi-binding and tuple
inputs. The producer repairs the three legacy observations; the narrow transform
scope guard repairs the three selected observations. Whole authored runtime laws
progress 4/12 at 943, 7/12 at 31d21, and 10/12 at be208. All outside/unscoped caches
remain correct. The two lexical default-RHS initial-value laws fail on every
source: `_ctx.fallback` yields empty text and undefined instead of the authored
script literal. Complete compiler Result packets and semantic runtime laws for
those defaults are identical across all three genuine source states. Preserve
the desired fallback expectations and fix the existing script binding handoff in
a focused follow-up; do not claim default semantics complete. The full native,
source and pinned TS 6.0.3/Vue 3.5.26/Node 24 observations are retained in
`/private/tmp/vize-typed-slot-default-original943-probe-20261010/classification-audit.json`,
SHA 256 `2f65e51977487598d93e7de0c647f54c32e04795957e41ed7176b747c5c3c34e`.
