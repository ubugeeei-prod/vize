# Slot utilities behavior contract

Normative state x input -> outcome table for `@vizejs/ui/slot-utils`. Every row
is proven by `src/families/foundations/slot-utils/slot-utils.test.ts`;
compile-only assertions live in
`src/families/foundations/slot-utils/slot-utils.types.test-d.ts`.

| #   | State                   | Input                    | Outcome                                                                                            | Proven by                                                                                                  |
| --- | ----------------------- | ------------------------ | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| SU1 | several prop sources    | `mergeProps(...)`        | classes joined, styles merged (later wins), handlers chained in order, last defined value wins     | `merges class, style, handlers, and last defined values`                                                   |
| SU2 | undefined / one handler | `mergeProps(...)`        | an explicit undefined is kept only without an earlier value; a single handler passes through as-is | `keeps an explicit undefined when no earlier value exists and passes single handlers through`              |
| SU3 | prop key                | `isHandlerKey(key)`      | `onX` and `onX:y` keys are handlers; `once`/`on` are not                                           | `recognizes handler keys`                                                                                  |
| SU4 | slot output             | `hasSlotContent(slot)`   | comments, whitespace, and empty fragments count as empty; nested content counts                    | `detects slot content through comments, whitespace, and fragments`                                         |
| SU5 | mounted component       | empty named slot         | wrappers for empty slots are skipped and `presentSlotNames` lists only non-empty slots             | `skips wrappers for empty slots in the mounted DOM`                                                        |
| SU6 | server render           | render twice, hydrate    | byte-identical markup and no hydration diagnostics                                                 | `renders identical slot-aware markup on the server`, `hydrates server markup without mismatch diagnostics` |
| SU7 | public type API         | merged props, slot props | handler unions, `class: string`, `style: StyleValue`, typed slot names and slot props              | `src/families/foundations/slot-utils/slot-utils.types.test-d.ts`                                           |

Everything is pure; slot checks invoke the slot during render so the calling
component tracks its dependencies.
