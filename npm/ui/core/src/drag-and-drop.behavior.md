# Drag and drop behavior contract

Normative state × input → outcome table for `@vizejs/ui/drag-and-drop`. Every
row is exercised by `src/drag-and-drop*.test.ts`; compile-only assertions live
in `src/drag-and-drop.types.test-d.ts`.

| #   | State                        | Input                                                | Outcome                                                              | Proven by                |
| --- | ---------------------------- | ---------------------------------------------------- | -------------------------------------------------------------------- | ------------------------ |
| D1  | idle                         | primary pointer, legacy mouse, or single touch press | one contact is armed; selection is guarded; no session yet           | pointer tests            |
| D2  | armed                        | movement below the start distance                    | lifecycle remains silent and a later release settles silently        | start-distance test      |
| D3  | armed                        | movement at or past the start distance               | one session starts; the payload snapshot and grab announcement emit  | pointer lifecycle test   |
| D4  | dragging                     | pointer over an accepting target rectangle           | innermost target enters; edge, indicator, and move announcement emit | hit-test tests           |
| D5  | dragging over target         | pointer crosses into another allowed edge            | edge changes emit target move callbacks and re-announce              | indicator test           |
| D6  | dragging over target         | pointer leaves every measurable target               | leave callback fires; over state, edge, and indicator clear          | leave test               |
| D7  | dragging over nested rects   | overlapping targets contain the point                | DOM containment picks the innermost target, then the smallest area   | nested ownership test    |
| D8  | dragging                     | disabled or rejecting targets under the point        | those targets never enter hit testing or keyboard order              | filtering test           |
| D9  | dragging over target         | owning contact releases                              | drop callback, drop announcement, and `dragend` settle exactly once  | drop test                |
| D10 | dragging outside targets     | owning contact releases                              | `dragend` reports no target without cancellation                     | outside-drop test        |
| D11 | armed or dragging            | Escape, native drag, blur, hidden document, cancel() | canceled settlement runs every teardown exactly once                 | cancellation tests       |
| D12 | dragging                     | reactive disablement becomes true                    | the session cancels before new movement is observed                  | reactive teardown test   |
| D13 | dragging near container edge | pointer enters the auto-scroll threshold band        | the container scrolls immediately and while the pointer holds        | auto-scroll test         |
| D14 | idle focus on handle         | Enter or Space without modifiers                     | keyboard session grabs and announces the first valid target          | keyboard grab test       |
| D15 | keyboard session             | arrows, Home, and End                                | valid targets cycle in document order with indicator and speech      | keyboard navigation test |
| D16 | keyboard session             | Enter or Space                                       | drop lands on the current target and settles the session             | keyboard drop test       |
| D17 | keyboard session             | Escape, Tab, or focus leaving the handle             | the session cancels and announces the cancellation                   | keyboard cancel test     |
| D18 | any session                  | registration disposal of the owning source           | the session cancels first and the registry entry is removed          | disposal tests           |
| D19 | any session                  | controller disposal                                  | listeners, live region, and reactive state release without callbacks | disposal tests           |
| D20 | typed payloads               | data-transfer and clipboard adapters round-trip      | structured payloads serialize losslessly; malformed input reads null | transfer tests           |
| D21 | concurrent SSR requests      | identical component trees                            | byte-identical markup contains no handlers or DOM reads              | SSR test                 |
| D22 | SSR followed by hydration    | pointer drag                                         | host identity remains and reactive output updates without warning    | hydration test           |
| D23 | public TypeScript API        | mutation or invalid closed union                     | compilation rejects misuse                                           | type assertions          |

## Accessibility and touch obligations

Every drag handle must be keyboard focusable so pointer drags have an
Enter-grab, arrow-move, Enter-drop equivalent, and every session phase speaks
through an owned assertive `role="status"` live region created lazily on the
first announcement. Announcement builders are injectable so consumers localize
grab, move, drop, and cancel messages. The primitive assigns no role, label,
or visual style; consumers must render the indicator geometry themselves and
must apply `touch-action: none` when continuous touch dragging must take
precedence over viewport panning. Hit testing compares measured rectangles
directly, so drag previews and overlays can never mask a drop target.
