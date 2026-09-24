# TransferList Behavior

## Contract

TransferList is a headless, strongly typed dual listbox. `transfer-list-root.vue`
is generic over the item type `T`: `items` holds every item, `v-model` holds the
target (right-hand) values as `readonly T[]`, and the source panel shows the
rest. `transfer-list-panel.vue` renders one multiselectable `role="listbox"`
per side with active-descendant navigation and typeahead from the shared
collection registry and composite navigation. `transfer-list-item.vue`
renders `role="option"` where `aria-selected` means "checked for moving".
`transfer-list-search.vue` filters one panel, `transfer-list-action.vue` moves
checked or all visible items, and `transfer-list-empty.vue` reports an empty
panel. Named lists submit one hidden input per target value. No CSS ships.

## Normative Behavior

| #   | State          | Input                                                | Outcome                                                                                                                                                   | Proven by                                                                       |
| --- | -------------- | ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| T1  | any            | render                                               | two multiselectable listboxes split `items` by the target model; hidden inputs mirror the target                                                          | `renders two multiselect listboxes that split items by the target model`        |
| T2  | source         | click items / "Add selected"                         | toggles checked state (never for disabled items), moves checked items, emits `change(value, previous, moved, direction, event)`                           | `clicking checks items and the selected action moves them, emitting the move`   |
| T3  | `source-order` | move / "Add all" / "Remove all"                      | keeps canonical order, skips disabled items, disables the action when nothing can move                                                                    | `source-order mode keeps canonical order and move-all skips disabled items`     |
| T4  | `max`          | "Add all"                                            | moves only up to `max`, publishes `data-full`, disables further adds                                                                                      | `max caps the target and disables moving further items`                         |
| T5  | search         | type / Escape / ArrowDown                            | filters one panel accent-insensitively, move-all moves only visible items, empty state distinguishes filtering, Escape clears, ArrowDown enters the panel | `search filters one panel accent-insensitively and drives the empty state`      |
| T6  | panel focused  | arrows / Home / Space / Shift+Arrow / Ctrl+A / Enter | navigates skipping disabled items, toggles, extends checks, checks all visible, moves checked (or the active item); the highlight recovers to a neighbour | `keyboard: navigation, Space toggles, Shift+Arrow extends, Ctrl+A, Enter moves` |
| T7  | any            | double-click / printable key                         | moves one item; typeahead highlights by text                                                                                                              | `double-click moves an item and typeahead finds items by text`                  |
| T8  | controlled     | move                                                 | emits and waits for the parent                                                                                                                            | `controlled target waits for the parent`                                        |
| T9  | disabled       | any                                                  | panels leave the tab order; items, actions, and search are inert                                                                                          | `disabled transfer lists block every interaction`                               |
| T10 | any            | exposed API                                          | `moveToTarget`, `moveToSource`, `reset` share state                                                                                                       | `exposed methods move checked items`                                            |
| T11 | no provider    | mount a part                                         | throws `VIZE_UI_CONTEXT_MISSING`                                                                                                                          | `parts require a TransferList provider`                                         |
| T12 | pure helper    | `transferItems`                                      | respects `max`, order modes, and direction                                                                                                                | `transferItems respects max, order modes, and direction`                        |

## SSR

Panel ids derive from the root id; markup is byte-identical across isolated
requests and hydrates without warnings (`renders byte-identical TransferList
markup across isolated SSR requests`, `hydrates TransferList without mismatches
or node replacement`).
