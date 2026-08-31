# Move behavior contract

Normative state × input → outcome table for `@vizejs/ui/move`. Every row is
exercised by `src/families/interaction/move/move*.test.ts`;
compile-only assertions live in
`src/families/interaction/move/move.types.test-d.ts`.

| #   | State                     | Input                                                          | Outcome                                                            | Proven by              |
| --- | ------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------ | ---------------------- |
| M1  | idle                      | primary mouse, pen, touch, or extensible pointer presses       | one contact is owned; selection is guarded; no callback yet        | pointer tests          |
| M2  | armed                     | zero-distance native movement                                  | lifecycle remains silent                                           | pointer lifecycle test |
| M3  | armed                     | first non-zero movement                                        | immutable start precedes the exact delta                           | pointer lifecycle test |
| M4  | moving                    | later movement                                                 | delta is relative to the immediately preceding owned event         | pointer lifecycle test |
| M5  | moving                    | owning pointer releases                                        | normal end emits; listeners and exact selection styles restore     | pointer lifecycle test |
| M6  | armed or moving           | unrelated contact moves or releases                            | event is ignored                                                   | ownership tests        |
| M7  | moving                    | reactive disablement changes or becomes invalid                | canceled end settles before the value or diagnostic is published   | reactive teardown test |
| M8  | idle                      | physical or legacy arrow key, including repeat                 | atomic start → configured delta → end; native scrolling is stopped | keyboard test          |
| M9  | idle                      | modified, composing, descendant, or unrelated key              | native behavior is preserved                                       | keyboard filter test   |
| M10 | legacy touch environment  | owned contact moves; compatibility mouse follows               | touch deltas emit once and emulated mouse is suppressed            | legacy touch test      |
| M11 | legacy mouse environment  | primary mouse moves outside the host                           | document listeners retain ownership and emit exact deltas          | legacy mouse test      |
| M12 | armed                     | release without movement                                       | state settles without a synthetic move lifecycle                   | stationary test        |
| M13 | armed or moving           | cancel, drag, blur, hidden document, dispose, or cleanup error | every teardown runs once and disposal is terminal                  | cancellation tests     |
| M14 | start callback re-enters  | controller is canceled                                         | canceled end emits and stale delta is suppressed                   | reentrancy test        |
| M15 | callbacks throw           | multiple required notifications                                | state remains coherent and failures aggregate afterward            | callback-failure test  |
| M16 | concurrent SSR requests   | identical component trees                                      | byte-identical markup contains no handlers or DOM reads            | SSR test               |
| M17 | SSR followed by hydration | pointer movement                                               | host identity remains and reactive output updates without warning  | hydration test         |
| M18 | public TypeScript API     | mutation or invalid closed union                               | compilation rejects misuse                                         | type assertions        |

## Accessibility and touch obligations

The host must be keyboard focusable so every pointer move has an arrow-key
equivalent. This primitive deliberately assigns no role, label, bounds, or
visual style. Consumers should announce value changes when movement represents
a semantic control such as a slider, and must apply `touch-action: none` when
continuous touch dragging must take precedence over viewport panning. The
primitive emits no CSS and never assumes layout, direction, or value bounds.
