# Segmented behavior contract

Normative state × input → outcome table for `src/components/segmented/segmented.vue`
and `src/components/segmented/segmented-item.vue`.
Arrow keys move through enabled radios on both axes and wrap.
Every row is proven by `src/components/segmented/tests/behavior.test.ts`.

| #   | State                | Input                     | Outcome                                                     | Proven by                                          |
| --- | -------------------- | ------------------------- | ----------------------------------------------------------- | -------------------------------------------------- |
| G1  | value day            | render                    | radiogroup; day is checked and tabbable; the others are not | `checks the selected radio and roves the tab stop` |
| G2  | value day            | ArrowRight                | selects the next item and moves focus                       | `moves and wraps with the arrow keys`              |
| G3  | value month          | ArrowRight                | wraps to the first item                                     | `moves and wraps with the arrow keys`              |
| G4  | middle item disabled | ArrowRight from the first | skips the disabled item                                     | `skips disabled items`                             |
| G5  | group disabled       | ArrowRight                | no selection change                                         | `a disabled group ignores selection`               |
| G6  | keyboard helper      | Home, End, arrows         | indexes wrap inside the enabled list                        | `resolves roving indexes for both axes`            |
