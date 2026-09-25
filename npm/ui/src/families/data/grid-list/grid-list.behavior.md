# GridList behavior contract

`@vizejs/ui/grid-list` applies the WAI-ARIA APG grid pattern to interactive lists. `grid-list.vue` is the
generic root (`Item`); `grid-list-item.vue` renders each `role="row"` with one `role="gridcell"`. Reordering
uses the shared `sortable` foundation through a drag handle. Every row is proven by the named test in
`grid-list.test.ts` or `grid-list-ssr.test.ts`.

| #   | State                      | Input                                                  | Outcome                                                                                           | Proven by                                                                                     |
| --- | -------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| L1  | any                        | render                                                 | `role="grid"` with `aria-rowcount`, rows with `aria-rowindex` and one gridcell, a single tab stop | `renders an APG grid of rows with one gridcell and a single tab stop`                         |
| L2  | `layout="list"`            | Up/Down, Home/End, PageUp/PageDown, `loop`, characters | roving focus moves, wraps with `loop`, and typeahead matches `getTextValue`                       | `list layout: Up/Down, Home/End, PageUp/PageDown, loop, and typeahead`                        |
| L3  | `layout="grid"`            | Left/Right, Up/Down, `dir="rtl"`                       | Left/Right step items, Up/Down step by `columns`, rtl flips                                       | `grid layout: Left/Right step items, Up/Down step rows, rtl flips`                            |
| L4  | `selectionMode="multiple"` | click, Ctrl, Shift, Space, Shift+Arrow, Ctrl+A         | replace, toggle, and range selection skipping disabled items; `aria-selected` / `aria-disabled`   | `multiple selection: click, Ctrl-click, Shift-click, Space, Shift+Arrow, Ctrl+A`              |
| L5  | single / none              | click, Enter, double click                             | single replaces; none publishes no `aria-selected`; Enter and double click emit `action`          | `single selection replaces; none mode publishes no aria-selected; Enter and double click act` |
| L6  | `reorderable`              | Enter on handle, arrows, Enter                         | a keyboard move commits; `reorder` and `update:items` fire; list keys ignore handle events        | `keyboard reorder through the drag handle commits and emits the new order`                    |
| L7  | no items                   | render                                                 | an empty row renders the `empty` slot                                                             | `empty lists render an empty row`                                                             |
| L8  | custom compositions        | import                                                 | `GridListItem` is exported                                                                        | `GridListItem is exported for custom compositions`                                            |
| L9  | SSR                        | two requests + hydrate                                 | byte-identical markup with one tab stop; warning-free hydration                                   | `renders identical grid-list markup across SSR requests and hydrates cleanly`                 |
