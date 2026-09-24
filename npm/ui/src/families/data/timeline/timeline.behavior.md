# Timeline Behavior Contract

Normative state x input -> outcome table for `timeline-root.vue`,
`timeline-item.vue`, `timeline-indicator.vue`, `timeline-connector.vue`,
`timeline-content.vue`, and `timeline-time.vue` (`@vizejs/ui/timeline`). Every
row is proven by the named test in `timeline.test.ts` or
`timeline-ssr.test.ts`; compile-only guarantees live in
`timeline.types.test-d.ts`.

A timeline is a native `<ol>` of `<li>` items, so assistive technology already
announces item count and position. The current item carries
`aria-current="step"`; indicators and connectors are `aria-hidden` decoration;
`TimelineTime` renders a native `<time datetime>`.

| ID  | State                     | Input                    | Outcome                                                                                  | Evidence                                                                 |
| --- | ------------------------- | ------------------------ | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| L1  | untracked                 | render                   | labelled `ol`/`li` structure, hidden decorative parts, last-item marker, native `<time>` | `renders a labelled ordered list with decorative parts and native time`  |
| L2  | root `value`              | render, value change     | items before the value are `complete`, the match is `current`, later ones `upcoming`     | `derives complete, current, and upcoming states from the root value`     |
| L3  | explicit item `status`    | render                   | the explicit status wins over derived progress                                           | `explicit item status overrides derived progress`                        |
| L4  | `reversed`, `horizontal`  | render                   | native `reversed` numbering and orientation data attributes                              | `reversed and horizontal timelines expose native and data semantics`     |
| L5  | conditional items, `Date` | insert item, render time | inserted items join document order; `Date` values serialize to deterministic ISO strings | `items added later join document order and Date values serialize to ISO` |
| L6  | missing provider          | setup                    | item parts fail closed with the shared context diagnostic                                | `compound parts require matching providers`                              |
| L7  | SSR and hydration         | isolated render/mount    | progress is derived during the single server pass and hydrates without warnings          | `timeline-ssr.test.ts`                                                   |

Items render before later siblings register, so during server rendering an
item seen before the current value is treated as `complete`; after mount a
value that matches no item clears every derived status.
