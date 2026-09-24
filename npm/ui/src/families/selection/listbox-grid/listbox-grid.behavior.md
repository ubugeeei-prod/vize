# ListboxGrid Behavior

## Contract

ListboxGrid is a headless, strongly typed 2-D option grid for icon, color,
swatch, and "rich select" pickers. `listbox-grid.vue` is generic over the value
type `T` and a `Multiple` literal (`T | null` or `readonly T[]`), renders
`role="listbox"` with `aria-activedescendant`, and keeps DOM focus on itself.
`listbox-grid-item.vue` renders `role="option"` and registers with the shared
collection registry, which also powers buffered typeahead. Arrow keys follow
the visual grid through the pure `moveInGrid` helper; `columns` must match the
consumer's CSS grid. Named grids submit one hidden input per selected value.
No CSS ships.

## Normative Behavior

| #   | State       | Input                                                       | Outcome                                                                                                  | Proven by                                                                        |
| --- | ----------- | ----------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| G1  | any         | render                                                      | `role="listbox"`, `tabindex=0`, `data-columns`, options expose row/column slot state, hidden form inputs | `renders listbox semantics with row and column slot state`                       |
| G2  | focused     | Arrow keys / Home / End / Ctrl+Home/End / PageUp / PageDown | focus highlights the selection, then moves in two dimensions, skipping disabled options; edges hold      | `focus highlights the selected option, then arrows move in two dimensions`       |
| G3  | `dir="rtl"` | ArrowLeft / ArrowRight                                      | mirrored                                                                                                 | `rtl mirrors horizontal arrows`                                                  |
| G4  | single      | Enter / Space / `selectionFollowsFocus`                     | selects the highlighted option; `change` carries the key event; follow-focus selects on arrows           | `Enter and Space select in single mode; selectionFollowsFocus selects on arrows` |
| G5  | multiple    | Space / Shift+Arrow / Ctrl+A                                | toggles, extends, selects every enabled option                                                           | `multiple mode toggles, extends with Shift+arrows, and selects all with Ctrl+A`  |
| G6  | any         | pointer                                                     | pointer-down keeps focus on the grid; click selects; disabled options ignore input                       | `click selects; disabled options ignore pointer input`                           |
| G7  | focused     | printable character                                         | typeahead moves the highlight by `textValue`                                                             | `typeahead moves the highlight by option text`                                   |
| G8  | disabled    | Tab / keys                                                  | removed from the tab order, keys ignored                                                                 | `disabled grids leave the tab order and ignore keys`                             |
| G9  | controlled  | click                                                       | emits and waits for the parent; `by` matches structurally equal values                                   | `controlled values wait for the parent`                                          |
| G10 | no provider | mount an item                                               | throws `VIZE_UI_CONTEXT_MISSING`                                                                         | `items require a ListboxGrid provider`                                           |
| G11 | pure helper | `moveInGrid`                                                | empty grids, ragged last rows, page moves, and row-wrap opt-out are deterministic                        | `pure grid navigation handles empty grids, ragged rows, and page moves`          |

## SSR

Ids come from the deterministic-id primitive; server markup is byte-identical
across isolated requests and hydrates without warnings (`renders byte-identical
ListboxGrid markup across isolated SSR requests`, `hydrates ListboxGrid without
mismatches or node replacement`).
