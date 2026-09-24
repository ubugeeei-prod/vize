# Dashboard grid behavior contract

Normative state x input -> outcome table for `dashboard-grid.vue` and
`dashboard-grid-item.vue` (`@vizejs/ui/dashboard-grid`). Rows are proven by
`dashboard-grid-layout.test.ts`, `dashboard-grid.test.ts`, and
`dashboard-grid-ssr.test.ts`; compile-only assertions live in
`dashboard-grid.types.test-d.ts`.

| #    | State                 | Input                      | Outcome                                                                                            | Proven by                                                                                                  |
| ---- | --------------------- | -------------------------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| DG1  | two items             | collision check            | overlapping cells collide; an item never collides with itself                                      | `detects overlapping cells but never self-collision`                                                       |
| DG2  | raw layout            | normalize                  | integer cells, sizes within min/max and columns, x inside the grid                                 | `normalizes sizes, constraints, and positions into the grid`                                               |
| DG3  | gaps / overlaps       | compact                    | items rise until blocked (static items never move); `"none"` only pushes overlaps down             | `compacts upward until blocked by static widgets and resolves overlaps`                                    |
| DG4  | layout                | move                       | the mover takes its cell, colliding items are pushed down, others compact; no-ops return the input | `moves push colliding widgets down and compact the rest`                                                   |
| DG5  | static widgets        | move onto / move static    | the layout is returned unchanged                                                                   | `moves onto static widgets are rejected and static widgets never move`                                     |
| DG6  | layout                | resize                     | size respects min/max and remaining columns; neighbours are pushed; `"none"` skips compaction      | `resizes within constraints and pushes neighbours`                                                         |
| DG7  | layout                | rows                       | the occupied row count is reported                                                                 | `reports the number of occupied rows`                                                                      |
| DG8  | default               | render                     | a labelled region with a native CSS grid; widgets are labelled groups on grid lines                | `renders a labelled CSS grid with widgets on native grid lines`                                            |
| DG9  | focused handle        | arrows, Shift+arrows       | move and resize by one cell with collision handling; static widgets and other keys are ignored     | `keyboard moves and resizes widgets and pushes collisions`                                                 |
| DG10 | measured grid         | pointer drag / resize grip | pointer deltas snap to whole cells; `data-dragging` is set while active                            | `pointer drags snap to cells using the measured cell size`                                                 |
| DG11 | disabled / controlled | input                      | disabled grids ignore input; controlled layouts only change via the prop                           | `disabled grids ignore input and controlled layouts only change via props`                                 |
| DG12 | no provider           | mount a widget             | stable `VIZE_UI_CONTEXT_MISSING: DashboardGrid` diagnostic                                         | `widgets outside a grid throw the context diagnostic`                                                      |
| DG13 | SSR                   | isolated requests, hydrate | byte-identical grid-line placement; hydration without warnings                                     | `renders byte-identical grid placement across SSR requests`, `hydrates the grid without mismatch warnings` |
| DG14 | DOM/SSR/Vapor         | compile                    | both SFCs compile in every renderer lane                                                           | `scripts/check-renderers.ts`                                                                               |

Placement is pure CSS grid (`grid-column`/`grid-row` spans), so server output
is final layout; pointer input only needs the measured grid width.
