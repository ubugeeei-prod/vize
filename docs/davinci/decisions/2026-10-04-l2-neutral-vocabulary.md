# Framework-neutral L2 vocabulary (2026-10-04)

Decision for [#6856](https://github.com/ubugeeei-prod/vize/issues/6856), a
Stage 2 design-only slice of [#6829](https://github.com/ubugeeei-prod/vize/issues/6829).
The decision specifies a neutral destination and reviews every current op.
It changes no enum, parser, dump protocol or product route.

## Present family and its limits

`davinci/vize_l2/src/op.rs` contains eight child-position `Op` variants and
four common attached `BindingOp` variants, followed by ten explicit Vue
bindings. Regions are owned by their ops; attached bindings have one owner.
The concrete enums and arena payloads are Drop-free with pinned footprints.
Keep those properties: this is not a proposal for a uniform extensible
`Operation` record, runtime string dispatch, or per-op serialization.

The current `ui.slot-content` doc explicitly calls it an authored `v-slot`
syntactic surface. Its modifiers, absent-name spelling and carrier are not
a canonical cross-framework content contract. Common `BindOp`, `OnOp` and
`ModelOp` similarly retain dialect modifiers or attribute encodings. Their
`ui.*` names alone do not prove neutrality.

## Semantic membership rule

A core op specifies observable UI meaning independent of its spelling,
framework helper, or backend. A framework adapter legalizes its syntax to
that contract only when meaning agrees. Similar source syntax with different
runtime meaning remains different typed semantics; never approximate another
framework by mapping it to Vue.

Preserve source-order precedence, owned regions, lexical captures, expression
identity, provenance and diagnostics. Retain syntax in L1 and dialect records
when legalization has not completed. Every unconverted dialect construct
produces an explicit unsupported diagnostic rather than silently disappearing.

## Review and destination

The following names are design destinations, not newly available mnemonics.

| Current type / mnemonic                | Destination                                                                                                                             | Contract or reason                                                                                                           |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `ElementOp` / `ui.element`             | Keep core                                                                                                                               | Element identity, namespace, ordered attributes/bindings and owned children                                                  |
| `ComponentOp` / `ui.component`         | Keep core                                                                                                                               | Resolved component reference, ordered inputs and owned content; runtime invocation is target policy                          |
| `TextOp` / `ui.text`                   | Keep core                                                                                                                               | Literal text with authored provenance                                                                                        |
| `InterpolationOp` / `ui.interpolation` | Keep core                                                                                                                               | An expression rendered as text; conversion semantics must be specified                                                       |
| `CommentOp` / `ui.comment`             | Keep core                                                                                                                               | Preserved comment; target policy decides whether its output is applicable                                                    |
| `IfOp` / `ui.if`                       | Keep core                                                                                                                               | Ordered conditions and one owned region per branch                                                                           |
| `ForOp` / `ui.for`                     | Keep core                                                                                                                               | Source expression, iteration bindings, key semantics and owned repeated region                                               |
| `SlotOp` / `ui.slot`                   | Core `ContentInvokeOp` / `ui.content-invoke` after legalization                                                                         | Named content reference, ordered arguments and an owned fallback region                                                      |
| `SlotContentOp` / `ui.slot-content`    | Authored form becomes `framework::vue::SlotContentOp` / `vue.slot-content`; legalizes to core `ContentProvideOp` / `ui.content-provide` | Named content definition with parameter bindings, captures and its own region; `v-slot` spelling/modifiers stay Vue-specific |
| `BindOp` / `ui.bind`                   | Keep core after modifier legalization                                                                                                   | Ordered one-way value binding; explicit neutral property/attribute channel and name/spread form                              |
| `OnOp` / `ui.on`                       | Keep core after modifier legalization                                                                                                   | Event name and handler with specified attachment/lifetime; Vue event guards are dialect semantics                            |
| `ModelOp` / `ui.model`                 | Keep core read/write contract; move Vue realization annotations into Vue records                                                        | Matched value-type flow; IME, `.lazy`, `.number`, `.trim` and runtime helper selection are not generic attributes            |

All ten existing Vue bindings remain typed Vue dialect payloads:

| Current variants                 | Existing mnemonics                  | Destination                                              |
| -------------------------------- | ----------------------------------- | -------------------------------------------------------- |
| `VueDirective`, `VueCssBind`     | `vue.directive`, `vue.css-bind`     | `framework::vue`, with their own legality rules          |
| `VueSync`, `VueSlotScope`        | `vue.sync`, `vue.slot-scope`        | Vue 2 legalization, with version/quirk identity retained |
| `VueOnce`, `VueMemo`, `VueShow`  | `vue.once`, `vue.memo`, `vue.show`  | Vue render/update semantics; not generic core flags      |
| `VueHtml`, `VueText`, `VueCloak` | `vue.html`, `vue.text`, `vue.cloak` | Vue content replacement and cloak behavior               |

`ExprRef::Filter` remains the `vue.filter` dialect expression. Retained
JS/TS references, foreign references and opaque expressions retain their
current capabilities and conservative laws. Core consumers must not inspect
opaque text as JS or equate two expressions because their bytes match.

## Content contracts, not a slot rename

`ContentProvideOp` is a definition attached to exactly one component-content
owner. It owns one region, parameter bindings and lexical capture references.
`ContentInvokeOp` invokes the resolved content with ordered arguments and
owns its fallback. Static or computed names use the existing `DynamicName`
concept, while default-name synthesis is an explicit legalization fact.

The Vue adapter must specify content lookup, default grouping, conditional
providers, argument evaluation, fallback triggering and invocation lifetime.
Preserve the distinction between absent content, content returning nothing,
and a falsey rendered value: these are not automatically equivalent. If a
rule is Vue-specific, keep it in a typed `vue.*` contract and require Vue
legalization; no neutral consumer guesses it from the owner's framework.

This separates today's syntactic `SlotContentOp` from the destination's
owned-region semantics. Update the content grouping and provenance tables,
region ownership verifier and dump parser together when implemented. Do not
attach a second owner to a region or duplicate the body to manufacture a
neutral-looking op. SFC and Vue JSX can share a content contract only after
their independently specified lowering preserves these same laws.

No claim is made that Svelte snippets, Solid children or another framework's
render props have this contract. Their future adapter either proves an exact
legalization or keeps its distinct dialect op. They are outside Vue Fes
implementation scope.

## Binding and execution boundaries

- `ui.bind` distinguishes property versus attribute binding as a typed
  semantic channel, not by searching raw Vue `prop`/`attr` strings. Spread
  precedence and explicit names remain in authored order. Vue `camel`
  normalization records provenance rather than becoming a core modifier.
- `ui.on` contains no raw `.stop`/`.prevent` modifier list in its final core
  form. Vue legalizes guards/wrappers with defined evaluation order or keeps
  them in a typed Vue event binding until a target supports it. Other
  frameworks' event delegation or capture rules are not inferred from Vue.
- `ui.model` retains paired read/write references and their type-flow law.
  A target consumes explicit framework realization metadata; it must not
  interpret arbitrary attributes as control flags. Unsupported write targets
  and missing realization capabilities produce diagnostics.
- JS/TS scopes, symbols, imports and references are core facts. Vue macro,
  props/emits/model/slots and unwrapping contracts live in `framework::vue`.
  They must be produced from the original retained syntax, never Croquis
  output presented as native facts.
- JSX execution model and interpretation depth remain explicit L2 semantics
  under #6885/#6859. A re-render component is not legalized to run-once just
  because both can produce `ui.element`. L3/L4 consume semantic contracts
  without branching on the source JSX dialect.

## Implementation and acceptance

Keep core payload modules separate from `framework::vue`; closed typed
child/attached enums may contain explicit dialect cases without importing
Vue meaning into generic passes. Add a dialect case only alongside a real
lowering/consumer, not speculatively for future frameworks.

Implement #6838's canonical artifact and #6841's Vue feature composition with
small slices. Rename/move commits stay separate from semantic legalization.
When mnemonic changes land, update dump printers/parsers, exhaustive matches,
provenance, verifiers, fixtures and supported protocol versions together.
Internal level formats may break; published legacy APIs and byte output keep
their existing compatibility requirements.

Acceptance must demonstrate:

1. Every current op and expression variant is accounted for; no authored
   modifiers, unknown values or dialect constructs are dropped.
2. Owned-region and single-binding-owner invariants, Drop-free storage and
   footprint/instruction budgets retain their gates.
3. Vue template and Vue JSX/TSX fixtures compare the actual shared semantic
   contract where meaning agrees, with explicit negative cases for execution
   models, fallback emptiness, modifier evaluation and model realization.
4. Legacy differential oracles stay dev-only; original same-run JS/TS syntax
   and native facts supply production. Product switches still require their
   fix-history gates, including #6880 for compilers.
5. Exact-head Actions and the protected merge queue pass with unchanged
   budgets. Other-framework design examples earn no native coverage credit.

This design-only decision satisfies #6856 after review and merge. The op
migration and runtime completeness remain implementation work in #6838,
#6841, #6885 and #6859; this page does not close those issues.
