# Tree Behavior Contract

Normative state x input -> outcome table for `tree-root.vue`, `tree-item.vue`,
`tree-item-toggle.vue`, and `tree-item-checkbox.vue` (`@vizejs/ui/tree`), plus
the `useTreeVirtualizer` and `useTreeReorder` adapters. Every row is proven by
the named test in `tree.test.ts`, `tree-adapters.test.ts`, `tree-model.test.ts`,
or `tree-ssr.test.ts`; compile-only guarantees live in `tree.types.test-d.ts`.

The tree follows the WAI-ARIA APG tree view pattern. It renders **flat**: every
visible node is one `role="treeitem"` row carrying `aria-level`,
`aria-setsize`, and `aria-posinset`, so DOM order equals visual order and rows
can be virtualized without nested `role="group"` containers. Nodes are
consumer-owned data of any type `T`; `getKey` and `getChildren` are typed
against `T`, and the key type `K` is inferred from `getKey` and flows into every
v-model, emit, slot, and exposed method.

| ID  | State                            | Input                                              | Outcome                                                                                                   | Evidence                                                                             |
| --- | -------------------------------- | -------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| T1  | default                          | render                                             | `tree` with flat `treeitem` rows, levels, set positions, `aria-expanded` on parents only, roving tabindex | `renders APG tree semantics with flat levels, set positions, and roving tabindex`    |
| T2  | focus on a row                   | ArrowDown/Up/Right/Left, Home, End                 | APG movement: Right expands then enters, Left collapses then moves to the parent, Home/End jump           | `arrow keys, Home, and End follow the APG tree keyboard model`                       |
| T3  | focus inside a branch            | collapse an ancestor                               | focus and the active key move to the collapsed ancestor instead of being lost                             | `collapsing an ancestor moves focus from a hidden descendant to the ancestor`        |
| T4  | `dir="rtl"`                      | ArrowLeft / ArrowRight                             | horizontal expand and collapse keys swap                                                                  | `RTL maps ArrowLeft to expand and ArrowRight to collapse`                            |
| T5  | single selection                 | click, Enter, focus movement                       | click and Enter select one key; Enter and double click emit `action`; `selectionFollowsFocus` opt-in      | `single selection follows click and Enter, and emits action for activation`          |
| T6  | multiple selection               | click, Shift+click, Space, Shift+Arrow, Ctrl/Cmd+A | toggles, anchor ranges, focus extension, and select-all/clear-all with `aria-multiselectable`             | `multiple selection toggles, extends ranges, and selects all`                        |
| T7  | `selectionMode="none"`           | click, Enter                                       | no `aria-selected`, no selection emits; `action` still fires                                              | `selection mode none omits aria-selected and ignores selection input`                |
| T8  | `checkable` cascade              | Space, checkbox click                              | checks cascade to descendants; ancestors derive `checked`/`mixed`/`unchecked` in `aria-checked`           | `checkboxes cascade to descendants and derive tri-state ancestors`                   |
| T9  | `checkPropagation="independent"` | Space                                              | only the requested node toggles                                                                           | `independent checkboxes toggle only the requested node`                              |
| T10 | lazy branch                      | expand                                             | `loadChildren` runs once, row is `aria-busy` while loading, children inherit a checked parent, cached     | `lazy children load on expand with busy state, load events, and inherited checks`    |
| T11 | lazy branch                      | load rejects, then `reload()`                      | `data-load-state="error"` and `loadError`; reload retries and renders children                            | `failed lazy loads expose an error state and can reload`                             |
| T12 | focus on a row                   | printable characters                               | typeahead focuses the next row whose text matches, using the shared typeahead buffer                      | `typeahead moves focus to the next row whose text matches`                           |
| T13 | any                              | `*`, `expandAll()`, `collapseAll()`                | `*` expands the focused row's siblings; expand-all covers every resolved branch                           | `asterisk expands siblings and expandAll/collapseAll cover every resolved branch`    |
| T14 | disabled row / disabled tree     | click, keys, Tab                                   | disabled rows stay focusable but never expand, select, or check; a disabled tree leaves the tab order     | `disabled rows stay focusable but cannot expand, select, or check`                   |
| T15 | controlled `expanded`/`selected` | user input                                         | update events fire while rendered state waits for the parent                                              | `controlled expansion and selection wait for the parent to accept requests`          |
| T16 | toggle part / `expandOnClick`    | click                                              | the toggle expands without selecting and keeps focus on the row                                           | `the toggle expands on click without selecting, and expandOnClick toggles rows`      |
| T17 | exposed instance                 | imperative calls                                   | typed `focus`, `focusKey`, `expand`, `toggle`, `setSelected`, `toggleChecked`, `getCheckedState`          | `exposes typed state and imperative focus, selection, and checkbox controls`         |
| T18 | missing provider                 | setup                                              | compound parts fail closed with the shared context diagnostic                                             | `compound parts require a matching root provider`                                    |
| T19 | `useTreeVirtualizer`             | render                                             | only the window renders, every row still reports its full-tree set metadata                               | `a virtualizer renders only the window and keeps set metadata for every row`         |
| T20 | `useTreeVirtualizer`             | End, typeahead                                     | focus scrolls unmounted rows into the window first; typeahead reaches them through `getTextValue`         | `keyboard focus scrolls rows outside the window into view before focusing them`      |
| T21 | active row scrolled out          | Tab into the tree                                  | the tree element becomes the tab stop and forwards focus to the active row                                | `the tree element takes sequential focus while the active row is outside the window` |
| T22 | `useTreeReorder`                 | Alt+ArrowUp/Down/Right/Left                        | `onMove` receives sibling, nesting, and outdent requests; data is never mutated by the tree               | `Alt+Arrow keys request sibling, nesting, and outdent moves`                         |
| T23 | `useTreeReorder` with `canMove`  | move request                                       | vetoed and self-targeted moves are dropped; rows register with the shared pointer sortable engine         | `canMove vetoes moves and rows register with the pointer sort engine`                |
| T24 | model helpers                    | index, flatten, derive, toggle                     | pre-order indexing, duplicate-key diagnostic, tri-state derivation, disabled-aware cascade                | `tree-model.test.ts`                                                                 |
| T25 | SSR and hydration                | isolated render/mount                              | byte-identical markup, virtual window from `initialRect`, hydration without warnings                      | `tree-ssr.test.ts`                                                                   |

## Notes

- Lazy branches that start expanded load after mount, so server and client
  render the same idle markup before any request runs.
- The toggle and checkbox parts are `aria-hidden` pointer affordances: keyboard
  users expand with arrows and check with Space on the row itself, which keeps
  interactive controls from nesting inside a `treeitem`.
- Keys are compared with `Object.is`; `1` and `"1"` are distinct keys but must
  not both appear in one tree when reordering, because the drag engine keys
  rows by `String(key)`.
- No styles ship with the primitive. Rows expose `data-state`, `data-level`,
  `data-selected`, `data-checked`, `data-load-state`, `data-dragging`, and
  `data-drop-position`; indentation and virtual offsets are consumer-owned.
