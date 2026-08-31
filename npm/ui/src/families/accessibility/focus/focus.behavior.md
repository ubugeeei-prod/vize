# Focus behavior contract

Normative state × input → outcome table for `@vizejs/ui/focus`. Every row is
exercised by `src/families/accessibility/focus/focus*.test.ts`; compile-only
assertions live in `src/families/accessibility/focus/focus.types.test-d.ts`.

| #   | State                      | Input                                              | Outcome                                                              | Proven by                                                |
| --- | -------------------------- | -------------------------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------- |
| F1  | idle target                | host receives focus                                | immutable focus snapshot and owned state are published               | direct-focus test                                        |
| F2  | idle target                | descendant receives focus                          | descendant focus is ignored                                          | ownership-mode test                                      |
| F3  | idle within                | composed descendant receives focus                 | host owns focus and records the deepest active element               | within/shadow tests                                      |
| F4  | focused within             | focus moves between composed descendants           | ownership remains active without duplicate transitions               | within-boundary test                                     |
| F5  | focused                    | focus leaves the owned boundary                    | immutable blur snapshot includes the destination                     | direct/within tests                                      |
| F6  | focused                    | modality changes from pointer to keyboard          | focus-visible state updates without a duplicate focus phase          | modality test                                            |
| F7  | focused after mount        | `autoFocus` is enabled                             | visible-ring state remains true regardless of input modality         | auto-focus test                                          |
| F8  | focused                    | reactive disabled becomes true                     | lifecycle settles without moving DOM focus                           | reactive test                                            |
| F9  | focused                    | reactive disabled resolves to an invalid value     | lifecycle settles before the stable runtime diagnostic surfaces      | invalid-reactive test                                    |
| F10 | DOM focused, observer idle | explicit refresh                                   | ownership and ring state reconcile from the composed active element  | refresh test                                             |
| F11 | focused                    | manual cancel                                      | listeners/state release while DOM focus remains unchanged            | cancellation test                                        |
| F12 | focused                    | host is removed without blur                       | mutation observer settles leaked ownership                           | removal test                                             |
| F13 | focused                    | host blur delivery is unavailable                  | document focus safety net settles ownership                          | safety-net test                                          |
| F14 | transition callback active | callback cancels reentrantly                       | true→false is ordered and no stale focus phase is published          | reentrancy test                                          |
| F15 | callbacks throw            | multiple independent callbacks fail                | state remains settled and failures aggregate after notification      | callback-failure test                                    |
| F16 | listener setup fails       | mutation observation cannot start                  | installed document/modality resources roll back                      | setup-failure test                                       |
| F17 | cleanup throws             | one or more owned resources fail to release        | every cleanup runs and all failures remain inspectable               | cleanup tests                                            |
| F18 | effect scope ends          | any focus hook is scope-owned                      | controller is terminally disposed without user callbacks             | scope test                                               |
| F19 | concurrent SSR requests    | identical component trees                          | byte-identical handler-free markup renders without a server document | SSR test                                                 |
| F20 | SSR followed by hydration  | host receives and loses focus                      | DOM identity remains and reactive state renders without diagnostics  | hydration test                                           |
| F21 | DOM, SSR, and Vapor lanes  | authored consumer compiles                         | all renderer lanes accept the same public props and reactive state   | renderer-conformance gate                                |
| F22 | root and subpath consumers | only focus is retained                             | equivalent CSS-free bundles exclude unrelated component signatures   | tree-shaking gate                                        |
| F23 | public TypeScript API      | state mutation, invalid mode, or callback mismatch | compile-only assertions reject misuse                                | `src/families/accessibility/focus/focus.types.test-d.ts` |

## Accessibility obligation

The controller normalizes ownership and focus-indicator intent; it never moves
focus, invents a role, or emits CSS. Consumers must bind the returned props to
the same semantic host, keep keyboard focus order meaningful, and render a
visible indicator whenever `isFocusVisible` is true. `autoFocus` is reserved
for programmatic mount focus where a ring is required even without keyboard
modality. Focus styling remains entirely user-controlled.
