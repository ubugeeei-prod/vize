# Virtualizer behavior contract

Normative state × input → outcome table for `@vizejs/ui/virtualizer`. Every row
is exercised by `src/families/interaction/virtualizer/virtualizer*.test.ts`; compile-only assertions live in
`src/families/interaction/virtualizer/virtualizer.types.test-d.ts`.

| #   | State          | Input                                       | Outcome                                                      | Proven by                                                          |
| --- | -------------- | ------------------------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------ |
| V1  | fixed sizes    | initial layout                              | visible range plus overscan renders, with exact offsets      | `renders only the visible window plus overscan`                    |
| V2  | variable sizes | per-index size resolver                     | offsets accumulate per item, measurements are ignored        | `resolves variable sizes per index` (cache)                        |
| V3  | estimated      | dynamic measurement lands                   | estimate is replaced and later offsets shift                 | `overrides estimates with measurements and reports deltas`         |
| V4  | any            | scroll offset changes                       | window and range follow the offset                           | `windows follow scroll offset updates`                             |
| V5  | attached       | viewport scroll event                       | offset is read from the element and the window updates       | `reads scroll and size from an attached viewport`                  |
| V6  | horizontal     | orientation `horizontal`                    | inline-axis offsets and viewport extents drive the window    | `virtualizes the inline axis`                                      |
| V7  | lanes          | `lanes` greater than one                    | items window per lane with independent offsets               | `assigns lanes round-robin with independent offsets` (cache)       |
| V8  | grid           | row and column virtualizers over one host   | cells materialize with row-major geometry                    | `windows rows and columns over one shared viewport` (grid)         |
| V9  | sticky         | scrolled past a sticky index                | newest passed sticky item stays rendered and flagged         | `keeps the active sticky item mounted while scrolled past`         |
| V10 | anchoring      | item before the viewport changes size       | scroll offset shifts by the delta, view stays stable         | `anchors the viewport when items above it change size`             |
| V11 | restoration    | snapshot captured, layout re-estimated      | restore lands on the anchored item plus its gap              | `restores a snapshot through its anchored item`                    |
| V12 | measured       | `invalidateMeasurements(fromIndex)`         | measurements from the index are dropped and layout resets    | `invalidates measurements from an index`                           |
| V13 | measured       | tracked node leaves the document            | node is released, its measurement survives for reuse         | `measures rendered elements and recovers disconnected nodes`       |
| V14 | infinite       | range nears the trailing edge               | one forward load with an abort signal runs at a time         | `loads forward when the range nears the end` (infinite)            |
| V15 | infinite       | range nears the leading edge                | backward load fires; `notifyPrepended` keeps the view stable | `prepending shifts measurements and keeps the view anchored`       |
| V16 | infinite       | `cancel` while a load is in flight          | signal aborts and the stale result is discarded              | `ignores stale results after cancellation` (infinite)              |
| V17 | scrolling      | `scrollToIndex` with each alignment         | offset satisfies start, center, end, and auto placement      | `scrolls an index into each alignment`                             |
| V18 | any            | invalid options, indexes, or disposed calls | `VIZE_UI_VIRTUALIZER_*` diagnostics                          | `validates options and rejects misuse`                             |
| V19 | concurrent SSR | identical trees with `initialRect`          | byte-identical windows without any DOM access                | `renders byte-identical SSR windows` (ssr)                         |
| V20 | hydration      | server window mounts                        | no replacement and no hydration diagnostics                  | `hydrates the server-rendered window without diagnostics`          |
| V21 | public types   | invalid alignment or mutating readonly refs | compilation rejects misuse                                   | `src/families/interaction/virtualizer/virtualizer.types.test-d.ts` |
