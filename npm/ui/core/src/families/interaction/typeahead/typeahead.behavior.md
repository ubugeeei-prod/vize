# Typeahead behavior contract

Normative state × input → outcome table for `@vizejs/ui/typeahead`. Every row is
exercised by `src/families/interaction/typeahead/typeahead*.test.ts`; compile-only assertions live in
`src/families/interaction/typeahead/typeahead.types.test-d.ts`.

| #   | State                     | Input                                             | Outcome                                                               | Proven by                |
| --- | ------------------------- | ------------------------------------------------- | --------------------------------------------------------------------- | ------------------------ |
| T1  | empty buffer              | one printable grapheme                            | locale-aware prefix match becomes active and immutable snapshot emits | prefix test              |
| T2  | one-grapheme buffer       | same grapheme with case/diacritic variation       | query stays atomic and matching items cycle                           | repeated-input test      |
| T3  | mixed buffer              | another grapheme                                  | query extends and narrows the prefix match                            | mixed-input test         |
| T4  | pending buffer            | timeout elapses                                   | query and timer ownership clear                                       | timeout test             |
| T5  | pending buffer            | reactive timeout changes                          | existing timer is rescheduled from the change                         | timeout test             |
| T6  | keyboard host             | IME, Dead/Process, command shortcut, or named key | event is preserved for its native or owning-component behavior        | keyboard-filter test     |
| T7  | keyboard host             | AltGraph/international printable input            | grapheme is consumed without treating it as a command shortcut        | international-input test |
| T8  | empty buffer              | Space with default options                        | activation key is preserved                                           | keyboard-filter test     |
| T9  | nonempty buffer           | Space                                             | multi-word query extends                                              | multi-word test          |
| T10 | empty buffer, opted in    | Space                                             | leading-space query is accepted                                       | allow-space test         |
| T11 | Unicode input             | emoji ZWJ grapheme                                | complete grapheme remains one atomic query                            | Unicode test             |
| T12 | manual input              | zero or multiple graphemes                        | stable runtime diagnostic rejects ambiguous input                     | Unicode test             |
| T13 | pending buffer            | reactive disabled becomes true                    | query/timer clear synchronously and later input is ignored            | disabled test            |
| T14 | pending buffer            | reactive source becomes invalid                   | ownership clears before the option diagnostic surfaces                | invalid-reactive test    |
| T15 | match callback throws     | active state already moved                        | collection/query remain committed and original failure surfaces       | callback-failure test    |
| T16 | registry is disposed      | input attempts collection mutation                | buffer clears and collection diagnostic is preserved                  | registry-failure test    |
| T17 | active                    | reset, dispose, or Vue scope stop                 | timers release and imperative calls become terminal                   | lifecycle test           |
| T18 | concurrent SSR requests   | identical collections                             | byte-identical markup contains no handlers or scheduled timers        | SSR test                 |
| T19 | SSR followed by hydration | printable key                                     | host identity remains and active/query state renders without warning  | hydration test           |
| T20 | DOM, SSR, and Vapor lanes | authored consumer compiles                        | every renderer accepts the same keyboard props and reactive query     | renderer gate            |
| T21 | root and subpath consumer | only typeahead is retained                        | equal CSS-free bundles exclude unrelated component families           | tree-shaking gate        |
| T22 | public TypeScript API     | mutation or invalid sources                       | compile-only assertions reject misuse                                 | type declaration test    |

## Accessibility obligation

Typeahead supplements a complete composite keyboard interface; it never
replaces arrow navigation, semantic roles, selection, or a visible focus
indicator. The controller updates logical active state only. A roving-focus or
`aria-activedescendant` adapter must expose that state to the DOM, and consumers
must keep item `textValue` aligned with the name users perceive.
