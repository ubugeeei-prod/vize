# NativeSelect Behavior

## Contract

`native-select.vue` is a headless primitive over the platform `<select>` element. It
does not portal, render overlays, own typeahead, or replace browser selection
UI. Consumers style the native element and prop-rendered options through parts,
slots, CSS, and data attributes.

`readOnly` is intentionally unsupported because native `<select>` has no
readonly state. Consumers that need a non-editable submitted value should keep
the component controlled and ignore changes, or render disabled UI with a
separate hidden form value when that submission behavior is required.

## Public Surface

| Surface              | Contract                                                                                                                                                                                            |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `NativeSelect` props | `id`, `name`, `modelValue`, `defaultValue`, `options`, `multiple`, `size`, `disabled`, `required`, `direction`, `ariaLabel`, `ariaLabelledby`, `ariaDescribedby`, `ariaErrormessage`, `ariaInvalid` |
| Emits                | `update:modelValue(value)`, `change(value, previous, nativeEvent)`                                                                                                                                  |
| Slots                | `default(state)` for consumer-owned native `<option>` and `<optgroup>` children                                                                                                                     |
| Expose               | `element`, `id`, `value`, `selectedValues`, `disabled`, `required`, `invalid`, `selectionMode`, `multiple`, `direction`, `state`, `focus`, `setValue`, `clear`, `reset`                             |
| Parts                | `root`, `option` for prop-rendered options                                                                                                                                                          |
| Root data attributes | `data-vize-ui="native-select"`, `data-state`, `data-disabled`, `data-required`, `data-invalid`, `data-selection-mode`, `data-selection-count`, `data-direction`, `data-value`                       |
| Option data attrs    | `data-vize-ui="native-select-option"`, `data-state`, `data-value`, `data-selected`, `data-disabled` for prop-rendered options                                                                       |

## Normative Behavior

| Input                  | Single Selection                                                                | Multiple Selection                                                      |
| ---------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Tab                    | Moves focus to the native select when enabled. Disabled selects are skipped.    | Same as single.                                                         |
| Native open/navigation | Delegated entirely to the browser and operating system.                         | Same as single, including platform modifier keys.                       |
| Controlled value       | Emits the requested string and waits for the parent to accept it.               | Emits a readonly string array in DOM option order and waits for parent. |
| Uncontrolled value     | Mutates internal state, emits the requested string, and resets to default.      | Mutates internal state, emits selected strings, and resets to default.  |
| `options` prop         | Renders flat native options before slotted children. Disabled options remain.   | Same as single.                                                         |
| Default slot           | Receives value, selected values, state, disabled, invalid, mode, and direction. | Same as single; consumers bind selected state on custom options.        |
| `disabled`             | Removes the select from focus order and native form submission.                 | Same as single.                                                         |
| `required`/`name`      | Uses native constraint validation and form submission semantics.                | Same as single with browser-defined multiple submission behavior.       |
| `ariaInvalid`          | Reflects `aria-invalid` and gates `aria-errormessage`.                          | Same as single.                                                         |
| `readOnly`             | Unsupported by native select and not exposed as a prop.                         | Same as single.                                                         |

## SSR

Generated ids use the deterministic-id primitive. Isolated SSR requests must
produce byte-identical markup for the same tree, including selected
prop-rendered options, and hydration must not replace the rendered native select.
