# Hover behavior contract

Normative state × input → outcome table for `@vizejs/ui/hover`. Every row is
exercised by `src/families/interaction/hover/hover*.test.ts`;
compile-only assertions live in
`src/families/interaction/hover/hover.types.test-d.ts`.

| #   | State                     | Input                                                         | Outcome                                                      | Proven by                                              |
| --- | ------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------ |
| H1  | idle                      | mouse or pen pointer enters                                   | immutable start snapshot and hovered state are published     | boundary test                                          |
| H2  | idle                      | touch or extensible unknown pointer enters                    | input is ignored                                             | filter test                                            |
| H3  | idle                      | disabled or nonmatching configured family enters              | input is ignored                                             | filter test                                            |
| H4  | hovered                   | pointer moves between host descendants                        | hover remains owned by the host                              | boundary test                                          |
| H5  | hovered                   | owning pointer leaves host                                    | normal end snapshot is emitted                               | boundary test                                          |
| H6  | hovered                   | reactive disabled/filter changes then pointer moves           | canceled end is emitted                                      | reactive test                                          |
| H7  | hovered                   | touch device switch, pointer cancel, blur, or hidden document | canceled end is emitted                                      | cancellation tests                                     |
| H8  | touch just ended          | compatibility mouse enter within 800 ms                       | emulated hover is ignored                                    | legacy fallback test                                   |
| H9  | legacy browser            | genuine mouse enter after suppression window                  | fallback hover starts and ends normally                      | legacy fallback test                                   |
| H10 | Pointer Events browser    | compatibility mouse handlers also run                         | duplicate mouse lifecycle is ignored                         | legacy fallback test                                   |
| H11 | active callback re-enters | start callback cancels                                        | stale `hovered=true` change is not published                 | reentrancy test                                        |
| H12 | callbacks throw           | multiple transition notifications                             | state settles and failures aggregate afterward               | callback-failure test                                  |
| H13 | active                    | manual cancel, dispose, or scope stop                         | state/listeners release with explicit cancellation semantics | ownership test                                         |
| H14 | concurrent SSR requests   | identical component trees                                     | byte-identical markup contains no handlers or DOM reads      | SSR test                                               |
| H15 | SSR followed by hydration | mouse enter/leave                                             | host identity remains and state renders without diagnostics  | hydration test                                         |
| H16 | public TypeScript API     | mutation or invalid closed union                              | compilation rejects misuse                                   | `src/families/interaction/hover/hover.types.test-d.ts` |

## Accessibility obligation

Hover must never be the only way to discover or invoke an action. Consumers
must provide equivalent focus, press, long-press, or visible-control behavior.
This primitive deliberately ignores touch and unknown pointers, emits no CSS,
does not change focus, and leaves semantic roles and styling entirely to the
owning component.
