# Shortcut behavior contract

Normative state × input → outcome table for `@vizejs/ui/shortcut`. Every row is
exercised by `src/shortcut*.test.ts`; compile-only assertions live in
`src/shortcut.types.test-d.ts`.

| #   | State                          | Input                                        | Outcome                                                               | Proven by             |
| --- | ------------------------------ | -------------------------------------------- | --------------------------------------------------------------------- | --------------------- |
| S1  | registered chord               | matching keydown with exact modifiers        | handler receives an immutable match and the native action is canceled | chord test            |
| S2  | registered chord               | keydown with extra or missing modifiers      | nothing dispatches and the event keeps its native action              | chord test            |
| S3  | `Mod` pattern                  | parsed on apple and standard platforms       | Meta and Control resolve respectively and match accordingly           | platform test         |
| S4  | registered sequence            | first chord of the sequence                  | pending state records the step and later chords complete the handler  | sequence test         |
| S5  | pending sequence               | timeout elapses or a non-continuing key      | pending clears; the key is retried as a fresh sequence start          | sequence-reset test   |
| S6  | shadowed shortcut              | scope activated above global                 | the deepest active scope wins; release restores the earlier routing   | scope test            |
| S7  | scoped binding, scope inactive | matching keydown                             | nothing dispatches                                                    | scope test            |
| S8  | conflicting registrations      | identical normalized sequence in one scope   | `getConflicts` groups them; the latest registration wins routing      | conflict test         |
| S9  | binding with `when` gate       | gate resolves false                          | binding is skipped without consuming the event                        | enablement test       |
| S10 | text-editing target            | printable shortcut without `allowInEditable` | binding is skipped so typing is preserved, opted-in bindings dispatch | editable-target test  |
| S11 | held key                       | auto-repeated keydown                        | only `allowRepeat` chords re-dispatch                                 | repeat test           |
| S12 | modifier-only or IME input     | lone modifier keydown or composing event     | pending state is preserved and nothing dispatches                     | filter test           |
| S13 | reactive target                | target ref resolves or changes               | native listeners move to the new target, including shadow roots       | target test           |
| S14 | reactive disabled              | disabled becomes true                        | pending clears synchronously and later input is ignored               | disabled test         |
| S15 | invalid pattern or options     | malformed pattern, chord, or option value    | stable runtime diagnostics reject the misuse                          | diagnostics test      |
| S16 | any pattern                    | formatted for display                        | keycaps follow platform order and style deterministically             | format test           |
| S17 | active registry                | dispose or Vue scope stop                    | listeners and timers release and imperative calls become terminal     | lifecycle test        |
| S18 | concurrent SSR requests        | identical consumers                          | byte-identical markup contains no listeners or scheduled timers       | SSR test              |
| S19 | SSR followed by hydration      | matching keydown                             | host identity remains and the shortcut dispatches without warnings    | hydration test        |
| S20 | DOM, SSR, and Vapor lanes      | authored consumer compiles                   | every renderer accepts the same registry props and formatted keycaps  | renderer gate         |
| S21 | root and subpath consumer      | only shortcut is retained                    | equal CSS-free bundles exclude unrelated component families           | tree-shaking gate     |
| S22 | public TypeScript API          | mutation or invalid options                  | compile-only assertions reject misuse                                 | type declaration test |

## Accessibility obligation

Shortcuts are accelerators, never the only path: every action reachable
through a shortcut must stay reachable through visible, focusable controls.
Bindings skip text-editing targets by default so shortcuts cannot steal
typing, single-character shortcuts should stay opt-in per WCAG 2.2 §2.1.4
(Character Key Shortcuts), and consumers should surface `formatShortcut`
output near the control (for example in `<kbd>` or `aria-keyshortcuts`) so
users can discover the accelerator. Platform detection is deterministic on
the server; pass an explicit platform when rendering keycaps into SSR markup.
