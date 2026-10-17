# Retained template reference authority (#9251)

The original `b49525def17b778fb10b110a9e3b59ef2d1efbe5` Draft preserved the unsafe
synthetic expression/arrow parser for review. The replacement removes that
consumer parser entirely. Parent #9222 remains the separately delivered script
span/JSX fix. All 70 original contracts and source/provider packets are unchanged.
This child remains Draft until exact-source native/CLI CI and source review pass.

## Producer, packet and consumer

`Parser::parse_with_template_syntax_facts` enables a non-serialized sidecar on the
original Armature attribute/interpolation callbacks. Ordinary `parse()` retains
no sidecar records. Existing Relief expression and modern slot roots are borrowed
by exact pointer; no expression or slot is reparsed. Selected `v-for`, named
`v-on`, and explicitly selected Vue 2 legacy slot parameter goals retain genuine L1 `NativeForHead`,
`RetainedHandlerBody`, and `RetainedSlotParams` owners. For selection replaces the ordinary whole-head expression attempt. Named handlers
preserve every successful original Expr AST (including function/object values);
only a failed Expr selects the actual statement-body owner. That second goal is
explicitly included in the cost boundary below.

Each sidecar record admits its actual grammar, original source-buffer identity,
whole authored extent, cooked input, and canonical L1 entity/wrapper coordinates
together. A refused parser goal remains a refusal. Callers cannot attach a map or
construct a source/AST pair. A source copy, mismatched extent, changed cooked text,
or truncated packet cannot authorize a query.

Croquis consumes those records into a separate demand-only lexical packet. The
existing compiler relation and serialized Croquis output are unchanged. A
separate private-constructor semantic reuse packet avoids selected historical
reparses without supplying editor witnesses or seeding the global text cache. Full
identifier AST spans, including escape sequences, are projected through the
original owner; semantic-name length and source-name searching are never used.
References distinguish witnessed template locals, component-context lookup,
and unknown enclosing scopes. A refused slot/loop cannot convert its children
into component globals. Binding patterns include defaults, computed keys, rest,
callback parameters and block declarations. Type-only identifiers are excluded.

Collections and conditions are read before same-element loop aliases. Dynamic
slot names and ordinary attributes are read before slot props; slot declarations
scope only the children. Raw directives, retained `If` branch conditions/user
keys and modern slots are covered. Legacy `slot-scope` and `<template scope>`
are parameter scopes only when the original producer explicitly selects a Vue 2
dialect. Default Vue 3 treats these as plain static attributes: their strings do
not execute, and child identifiers still resolve in component context.
Patina runs the actual configured rule capability before script-only prefilters,
including scriptless SFCs, and consumes only witnessed component-context value
reads. Historical script receiver/reference coverage is unchanged.

## Bounds and refusal frontier

The existing native 31-unit admission limits are unchanged. Croquis separately
limits each retained syntax goal to depth 64 and 4096 units, including parameter and
local-declaration collection. Template traversal has depth 64 and 4096 nodes/facts.
Exhausting the template or producer packet budget discards all partial positives.
Invalid authored coordinates and malformed/unsupported syntax remain explicit
packet gaps. No gap counts as a successful clean reference result.

The bounded value walker currently refuses class/JSX expressions, direct `eval`
references, and statement forms outside its admitted block/variable/expression/
return/throw/if subset (including loops, switch, try, and function/class
statements). Transformed text-call/compound/hoisted template nodes are not admitted;
Patina consumes the original raw parser tree. These are visible capability
frontiers, not provider parity or complete JavaScript support. Unknown receivers
in script are unrelated to these template scopes and keep their existing warnings.

## Whole-packet controls

- `independent.json` and `native.json`: all original 70 cases, including all 17
  template cases; their bytes and hashes remain fixed.
- `cli70.json`: the same 70 source/config/status/whole JSON formatter expectations,
  with 126 diagnostics. The actual source-built CLI runs every case before an
  aggregate assertion; no output is filtered or normalized.
- `template-boundaries.json`: 42 additional original-source/provider/native
  packets, historically labelled 27 admitted, five product-accuracy cases and
  ten refusals. Complete expected native diagnostics
  include severity, message, help, labels, fix, and physical UTF-8 byte ranges.
- `template-cli.json`: all 42 historical complete CLI packets, including the refusal cases.
  The six legacy-attribute cases preserve their historical ESLint-derived labels;
  `template-authority-review-corrections.json` supplies a separate current Vue 3
  overlay, complete compiler/runtime proof and corrected whole native/CLI packets.
  These six cases are provider blind spots, not qualified lexical-scope parity.
  The derived current inventory is 23 admitted, 11 product-accuracy cases and eight
  refusals; the historical 27/5/10 labels remain byte-for-byte in the original.
  The compressed proof preserves complete official compiler/runtime observations;
  Rust embeds only the smaller authored-source/whole-expectation overlay.
- Four additional setup-guard sources prove actual compiler `$setup` bindings past
  statement, declarator and pattern budgets. Refusal remains Unknown and adds no
  new template warning. They run through the whole native Linter in both presets;
  they do not replace any of the 147 existing real-CLI observations.
  Clean native output for a refusal is not acceptance/parity credit; its packet
  must also retain an explicit Croquis coverage gap.

The boundaries cover authored attribute/interpolation entities, escaped names,
Unicode preceding a token, callbacks/block locals, type positions, loop collection
ordering/destructuring/defaults, modern and legacy slot defaults, dynamic slot
arguments, If conditions, statement handlers, malformed/failed slots, 31-unit
refusals, and whole-template width/depth rollback. Original source hashes and
complete pinned provider packets are replayed without range normalization.

Five cases intentionally differ from the pinned ESLint provider: same-element
loop collection/condition lookup, attributes/dynamic names outside slot props,
and TypeScript type-only positions. `syntaxProof` preserves the complete pinned
Vue compiler output and TypeScript erasure for each case, with implementation
hashes and empty compiler diagnostics. The executable replay proves the real
`_ctx.$listeners` value lookup that the rule must warn about; type-only names do
not become value warnings. Provider silence is not authority to lose a warning.

Original Expr/Slot parse counters and actual AST pointer identity are tested in
Armature; a separate Croquis integration test checks that repeated packet
consumption does not add expression parses. L1 does not expose a native parse
counter here; its owner identity and authenticated parameter/collection access
are asserted instead. Heavy Rust/native/actual CLI execution belongs to Actions.

## Whole-lint cost boundary

The optional semantic bridge authenticates original source identity/bytes and
exact whole-goal span/cooked text again at Drawer entry. Successful handler Expr
roots retain their ordinary coordinates and AST identity. A positively witnessed
statement-only Body can reuse `Inline` instead of rerunning the old TS/TSX
callback classifier; an ambiguous expression, function/class, block, or trailing
comment never supplies that proof. For collections reuse only the exact existing
legacy AST name walk where comment stripping leaves the input unchanged. Alias
parsing, the global identifier cache, occurrences and editor ownership witnesses
keep their previous behavior.

The following counts are **source call-path analysis**, not a global observed
parser census. They exclude test-only legacy differential replays and optional
unused-binding probes. The isolated `template_semantic_reuse` test records full
ordinary/demand outputs and the existing observed counters/spans separately.
Native L1 and classifier total parse counts remain uninstrumented/unknown; no
vendor feature or new metric dependency is introduced.

| Input family | Previous call path | Demanded call path | Boundary |
| --- | --- | --- | --- |
| Complete simple/function/object handler Expr | Original Expr; historical classifier; retained reads | Same original Expr and same legacy paths | Existing fast path preserved |
| Admitted multiple-statement handler | Failed Expr; TS and TSX classifier attempts; Expr/program reads | Failed Expr; native Body; proven Inline; same Expr/program reads | One extra native goal replaces two classifier attempts |
| Admitted ordinary For | Whole-head Expr; original alias parsing; collection-name parse | Native alias/collection owners; same alias parsing; exact legacy collection-name walk | Common total attempts unchanged; no global cache substitution |
| For whose comment stripper rewrites collection | Whole-head Expr; alias parsing; rewritten collection parse | Native owners; unchanged legacy rewritten-view fallback | Additional native work remains visible |
| Refused/ambiguous Handler, including single-expression semicolon or trailing line comment | Original Expr trial and historical classifier/read fallbacks | Same paths plus attempted native Body when admitted by its guards | Additional attempt may remain; no zero-duplicate claim |

The full-output control prints each complete source, ordinary/demand parser
diagnostics, unfiltered Croquis Debug, complete semantic snapshot, and observed
counters before asserting equality. Refusal and comment inputs are included
without raising admission limits or deleting expected diagnostics. Ordinary
compiler/drawer callers do not construct either demand packet or its storage.

SliceA setup-binding evidence is deliberately not repurposed as negative proof:
absence of a setup declaration does not establish an unbound component global,
and no invented BindingId bypasses ownership. Issue #9217 remains open for the
separate foreign/unknown receiver acceptance frontier. The manually closed
#8142 is not evidence that all n8n source/public/performance acceptance is done.

## Positive setup values and reference kinds

A bare template name can denote a user's setup value rather than the removed
instance API. The existing SliceA runtime-import accessor admits `.vue` imports;
its false result is not evidence about arbitrary setup constants. No invented
binding identity or negative inference is used here.

`TemplateComponentBindings` has private positive records. Its crate-private
constructor consumes only the descriptor's own already-parsed clean Program,
checked by source-buffer identity and inline JS/TS block role. Actual top-level
script-setup value declarations use the shared bounded binding-pattern collector;
value imports, functions and classes retain their real declaration token spans.
Type-only declarations, nested/block locals and ordinary module locals do not
supply that authority. For normal scripts, only a direct default-export object
can supply the existing shared OptionsDescriptor's `Setup` return-key records.
Factory spelling and broad `bindings.contains(name)` never qualify a component.
A normal setup return is not admitted when a script-setup block replaces it.

Inspection authority also retains Unknown for each target surface. A missing,
external, custom-language, erroneous or source-mismatched script parse, too many
statements/declarators/import specifiers, or a refused binding-pattern walk does
not establish absence. Such a surface cannot authorize a new template warning;
script warnings remain independent. Clean fully inspected blocks and scriptless
components can still establish the next component lookup. No iteration cap
silently truncates the binding inventory.

This evidence is constructed only when an actual retained component-context
identifier needs it, after the same existing script parses, with no rule-local
parser or full-file receiver exemption. Scriptless and prefilter exits retain
one template callback. All historical script warnings are unchanged, including
bare script references in setup return objects and explicit `this.$listeners`.
The new template packet's reference kind is **bare value identifier** only.
`this.$listeners` and static member properties are not that kind and are never
reclassified as local values. Template-this diagnostics remain an explicit
unqualified frontier; this child does not add a lexical-this walker.

`template-component-bindings.json` appends 30 complete source/provider/compiler
observations: 15 listeners product controls and 15 scopedSlots research controls.
The separate native and CLI manifests cover all 15 listeners sources with every
historical script warning retained. `template-component-declaration-bindings.json`
adds realistic declaration forms, erased types and script/template-this contrasts
without changing any earlier corpus. Its runtime packets execute the pinned
compiler output through TypeScript erasure and the actual Vue custom renderer:
a bare setup value reaches the child, while the public proxy/template-this value
is undefined. Complete warnings, compiler binding metadata/code/maps, original
source/hash, child attrs, setup state and proxy observations are preserved.

The scopedSlots rule is not extended by this product change. Its compiler/runtime
packets stay research evidence for a separate implementation. Original 70/42
packets and earlier CLI manifests remain frozen; no provider omission or clean
unsupported observation is credited as complete native acceptance.
