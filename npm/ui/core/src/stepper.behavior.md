# Stepper Behavior Contract

Normative state x input -> outcome table for `stepper-root.vue`,
`stepper-list.vue`, `stepper-item.vue`, `stepper-trigger.vue`, and
`stepper-content.vue` (`@vizejs/ui/stepper`). Every row in this isolated slice
is proven by the named focused tests; registry, package exports, renderer
fixtures, and size budgets are intentionally left for the parent integration
step.

| ID  | State                   | Input                 | Outcome                                                                                                        | Evidence                                                                                 |
| --- | ----------------------- | --------------------- | -------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| S1  | default / current       | render                | root, list, item, trigger, and content expose deterministic ids, ARIA wiring, slots, and data                  | `renders accessible stepper semantics with deterministic ids, slots, and data`           |
| S2  | linear / future pending | click                 | focusable future triggers advertise `aria-disabled` and do not emit or change value until prior steps complete | `linear navigation prevents future activation until prior enabled steps are complete`    |
| S3  | linear / reset default  | reset                 | imperative reset restores the configured default even when normal linear activation is currently locked        | `reset restores the configured default value even when linear activation is locked`      |
| S4  | dynamic item value      | prop update           | item registration, trigger ids, content ids, and fallback current value update when a step value changes       | `items reregister when their step value changes`                                         |
| S5  | free / disabled item    | Arrow key, then Enter | roving focus skips natively disabled steps and native keyboard activation selects any enabled step             | `free navigation and roving focus can activate any enabled step while skipping disabled` |
| S6  | controlled              | click                 | emits the requested value while rendered current state stays controlled until parent accepts it                | `controlled current value wins until the parent accepts the request`                     |
| S7  | disabled root / item    | click or Tab          | disabled triggers leave activation and sequential focus while preserving content state                         | `disabled roots and items suppress user activation and sequential focus`                 |
| S8  | exposed instances       | focus, next, previous | public refs expose element state and imperative focus/value methods                                            | `exposes typed state and imperative focus/value controls`                                |
| S9  | optional content role   | render                | content can render without a landmark role or default trigger label                                            | `content can opt out of the region role and default trigger label`                       |
| S10 | missing provider        | setup                 | compound parts fail closed with the shared context diagnostic                                                  | `compound parts require matching Stepper providers`                                      |
| S11 | SSR and hydration       | isolated render/mount | generated ids are byte-identical per request and hydrate without replacement warnings                          | `stepper-ssr.test.ts`                                                                    |

`navigationMode="linear"` allows the current step, previous enabled steps, and
future enabled steps only when every prior enabled step is marked completed.
Linear-locked future triggers remain roving-focusable with `aria-disabled` so
the flow remains discoverable; true root or item `disabled` states use native
disabled buttons and are skipped by keyboard navigation. Content panels retain
stable markup for hydration, use `hidden` while inactive, and default
`aria-labelledby` to the paired trigger id when a landmark role is present.
