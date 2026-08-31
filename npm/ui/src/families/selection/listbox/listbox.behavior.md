# Listbox Behavior

## Contract

Listbox is a headless compound primitive for option selection. `listbox.vue`
renders `role="listbox"` and owns DOM focus through `aria-activedescendant`.
`listbox-item.vue` renders `role="option"` and registers with the local
collection registry for ordering, typeahead, and disabled-state recovery.
Styling is owned by consumers through parts, slots, CSS, and data attributes.

## Public Surface

| Surface              | Contract                                                                                                                                                                                                                                              |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Listbox` props      | `id`, `modelValue`, `defaultValue`, `disabled`, `required`, `selectionMode`, `orientation`, `direction`, `loop`, `typeahead`, `typeaheadTimeout`, `ariaLabel`, `ariaLabelledby`, `ariaDescribedby`, `ariaErrormessage`, `ariaInvalid`                 |
| `Listbox` emits      | `update:modelValue(value)`, `change(value, previous, nativeEvent)`                                                                                                                                                                                    |
| `Listbox` slots      | `default(state)`, `empty(state)`                                                                                                                                                                                                                      |
| `Listbox` expose     | `element`, `id`, `value`, `selectedValues`, `activeValue`, `disabled`, `required`, `invalid`, `selectionMode`, `orientation`, `direction`, `state`, `focus`, `navigate`, `setActiveValue`, `setValue`, `selectValue`, `toggleValue`, `clear`, `reset` |
| `ListboxItem` props  | `id`, `value`, `disabled`, `textValue`, `order`, `ariaLabel`, `ariaLabelledby`, `ariaDescribedby`                                                                                                                                                     |
| `ListboxItem` slots  | `default(state)`, `indicator(state)`                                                                                                                                                                                                                  |
| `ListboxItem` expose | `element`, `value`, `active`, `selected`, `disabled`, `selectionMode`, `state`, `focus`, `select`                                                                                                                                                     |
| Parts                | `root`, `item`                                                                                                                                                                                                                                        |
| Root data attributes | `data-vize-ui="listbox"`, `data-state`, `data-disabled`, `data-required`, `data-invalid`, `data-orientation`, `data-selection-mode`, `data-selection-count`, `data-value`                                                                             |
| Item data attributes | `data-vize-ui="listbox-item"`, `data-state`, `data-value`, `data-selected`, `data-active`, `data-disabled`, `data-selection-mode`                                                                                                                     |

## Normative Behavior

| Input                    | Single Selection                                                                                            | Multiple Selection                   |
| ------------------------ | ----------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| Tab                      | Moves focus to the listbox when enabled. Disabled roots are skipped.                                        | Same as single.                      |
| Arrow Down / Arrow Right | Moves the active option to the next navigable item for matching orientation.                                | Same as single.                      |
| Arrow Up / Arrow Left    | Moves the active option to the previous navigable item for matching orientation.                            | Same as single.                      |
| Home / End               | Moves the active option to the first or last navigable item.                                                | Same as single.                      |
| Printable grapheme       | Runs locale-aware typeahead over option text and moves only active state.                                   | Same as single.                      |
| Enter / Space            | Selects the active option.                                                                                  | Toggles the active option.           |
| Pointer click            | Selects the clicked enabled option.                                                                         | Toggles the clicked enabled option.  |
| Disabled item            | Is exposed with `aria-disabled`, omitted from active navigation and typeahead, and cannot change selection. | Same as single.                      |
| Controlled value         | Emits the requested value and waits for the parent to accept it.                                            | Same as single with readonly arrays. |
| Uncontrolled value       | Mutates internal state, emits the requested value, and can reset to `defaultValue`.                         | Same as single with readonly arrays. |

## SSR

Generated root and option ids use the deterministic-id primitive. Isolated SSR
requests must produce byte-identical markup for the same tree, and hydration must
not replace the rendered root or option ids.
