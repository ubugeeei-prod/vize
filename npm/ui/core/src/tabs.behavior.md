# Tabs Behavior Contract

Normative state x input -> outcome table for `tabs-root.vue`, `tabs-list.vue`,
`tabs-trigger.vue`, and `tabs-content.vue` (`@vizejs/ui/tabs`). Every row in
this isolated slice is proven by the named focused tests; registry, package
exports, renderer fixtures, and size budgets are intentionally left for the
parent integration step.

| ID  | State                   | Input                  | Outcome                                                                                 | Evidence                                                                |
| --- | ----------------------- | ---------------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| T1  | default / selected      | render                 | root, tablist, tab, and tabpanel expose deterministic ids, ARIA wiring, slots, and data | `renders accessible tab semantics with deterministic ids and slots`     |
| T2  | uncontrolled automatic  | Arrow key              | roving focus skips disabled triggers and immediately selects the focused tab            | `automatic activation follows roving focus and skips disabled triggers` |
| T3  | uncontrolled manual     | Arrow key, then Space  | focus moves without selection until native activation requests the focused tab          | `manual activation waits for keyboard or pointer activation`            |
| T4  | controlled              | click                  | emits requested value while rendered selection stays controlled until parent accepts it | `controlled value wins until the parent accepts the request`            |
| T5  | disabled root / trigger | click or Tab           | disabled triggers leave activation and sequential focus while preserving content state  | `disabled roots and triggers suppress activation and focus`             |
| T6  | exposed instances       | focus, setValue, reset | public refs expose element state and imperative focus/value methods                     | `exposes typed state and imperative focus/value controls`               |
| T7  | missing provider        | setup                  | compound parts fail closed with the shared context diagnostic                           | `compound parts require a matching root provider`                       |
| T8  | SSR and hydration       | isolated render/mount  | generated ids are byte-identical per request and hydrate without replacement warnings   | `tabs-ssr.test.ts`                                                      |

Content panels use `hidden` while inactive, retain stable `tabpanel` markup for
hydration, and default `aria-labelledby` to the paired trigger id. Trigger
`indicator` is a named slot for consumer-owned active marker rendering; no
visual styling ships with the primitive.
