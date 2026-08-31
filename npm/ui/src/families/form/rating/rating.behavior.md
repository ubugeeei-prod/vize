# Rating behavior contract

Normative state x input -> outcome table for `rating.vue`
(`@vizejs/ui/rating`). Every row is proven by the named mounted-DOM or SSR test;
a row without a passing test is a contract violation.

| #   | State                | Input                     | Outcome                                                                                                                                                                            | Proven by                                                             |
| --- | -------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| R1  | named, selected      | render                    | native radio-group semantics, deterministic ids, named radio inputs, ARIA field state, data attributes, parts, and CSS custom properties reflect the normalized rating             | `renders native radio rating semantics with form and extension hooks` |
| R2  | uncontrolled         | pointer selection / clear | selecting a radio updates checked state, native form data, item slot state, and emits `update:modelValue` before `change`; selecting the current item clears only when `clearable` | `uncontrolled rating selects and clears through pointer activation`   |
| R3  | controlled           | pointer selection         | emits the requested value and `change`; rendered checked state reverts to `modelValue` until the parent accepts it                                                                 | `controlled value wins until the parent accepts the request`          |
| R4  | uncontrolled, seeded | form reset                | `defaultValue` seeds the initial radio and form reset restores it without request-global state                                                                                     | `defaultValue seeds state and native form reset restores it`          |
| R5  | focusable            | keyboard / exposed        | Space/Enter select, Home/End jump, arrows wrap, horizontal arrows honor `dir`, clear keys clear when allowed, and `focus()` targets the checked or first enabled radio             | `keyboard support honors native radio expectations and RTL direction` |
| R6  | disabled/read-only   | render / activation       | disabled ratings use native disabled radios and submit no value; read-only ratings remain focusable/submittable and suppress user changes                                          | `disabled and read-only ratings keep availability and form semantics` |
| R7  | normalized           | invalid bounds/value      | finite integer min/max/count/value repair prevents unsafe `NaN` or `Infinity` attributes in DOM and SSR output                                                                     | `normalizes finite integer bounds, value, direction, and state`       |
| R8  | imperative           | expose                    | instance methods focus, set, clear, reset, and expose normalized root/item state                                                                                                   | `exposes focus, setValue, clear, reset, and normalized state`         |
| R9  | SSR/hydration        | isolated requests         | server markup is byte-identical across requests and hydration preserves generated ids without diagnostics                                                                          | `renders byte-identical rating markup across isolated SSR requests`   |

## Public Props

| Prop               | Type                                 | Default     | Contract                                                                                        |
| ------------------ | ------------------------------------ | ----------- | ----------------------------------------------------------------------------------------------- |
| `id`               | `string \| null`                     | `undefined` | Consumer id for the radiogroup; nullish values use a deterministic fallback.                    |
| `name`             | `string`                             | `undefined` | Native radio name submitted with the selected value.                                            |
| `modelValue`       | `number \| null`                     | `undefined` | Controlled value; `undefined` selects uncontrolled mode and `null` clears.                      |
| `defaultValue`     | `number \| null`                     | `null`      | Initial uncontrolled value and reset target.                                                    |
| `min`              | `number`                             | `1`         | Lowest generated integer value.                                                                 |
| `max`              | `number`                             | `undefined` | Highest generated integer value; when omitted, `count` derives it.                              |
| `count`            | `number`                             | `5`         | Number of generated choices when `max` is omitted.                                              |
| `clearable`        | `boolean`                            | `false`     | Enables clearing through current-item activation, Escape, Delete, or Backspace.                 |
| `disabled`         | `boolean`                            | `false`     | Disables every radio, focus, activation, and native submission.                                 |
| `readOnly`         | `boolean`                            | `false`     | Keeps focus and current submission while blocking user changes.                                 |
| `required`         | `boolean`                            | `false`     | Applies native required validation to the generated radio set and `aria-required` to the group. |
| `dir`              | `"ltr" \| "rtl"`                     | `"ltr"`     | Exposes direction and flips horizontal arrow meaning in RTL.                                    |
| `itemLabel`        | `string`                             | `"Rating"`  | Prefix for generated item accessible names such as `Rating 3 of 5`.                             |
| `ariaLabel`        | `string`                             | `undefined` | Accessible group name when no external label exists.                                            |
| `ariaLabelledby`   | `string`                             | `undefined` | Space-separated ids that label the group.                                                       |
| `ariaDescribedby`  | `string`                             | `undefined` | Space-separated ids that describe the group.                                                    |
| `ariaErrormessage` | `string`                             | `undefined` | Error-message id used only while invalid.                                                       |
| `ariaInvalid`      | `boolean \| "grammar" \| "spelling"` | `false`     | Invalid state forwarded to ARIA and data hooks.                                                 |

## Emits, Slots, Expose

| Surface                                | Contract                                                                                 |
| -------------------------------------- | ---------------------------------------------------------------------------------------- |
| `update:modelValue(value)`             | Fired for distinct controlled/uncontrolled value requests.                               |
| `change(value, previous, nativeEvent)` | Fired after user activation requests a distinct rating.                                  |
| `clear(previous, nativeEvent)`         | Fired after user activation clears a non-empty value.                                    |
| default slot                           | Receives `RatingSlotState` for optional output beside the generated radios.              |
| `item` slot                            | Receives `RatingItemSlotState` for each generated indicator.                             |
| expose                                 | `root`, `elements`, normalized state, `focus()`, `setValue()`, `clear()`, and `reset()`. |

## Extension Hooks

| Hook                        | Values                                                                                                                                                                                                     |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| parts                       | `root`, `item`, `control`, `indicator`                                                                                                                                                                     |
| root data                   | `data-vize-ui="rating"`, `data-state`, `data-value`, `data-min`, `data-max`, `data-count`, `data-percent`, `data-dir`, `data-disabled`, `data-readonly`, `data-required`, `data-invalid`, `data-clearable` |
| item/control/indicator data | `data-vize-ui`, `data-state`, `data-value`, `data-index`, `data-active`, `data-checked`, `data-disabled`, `data-readonly`, `data-required`, `data-invalid`, `data-dir`                                     |
| CSS custom properties       | `--vize-rating-value`, `--vize-rating-min`, `--vize-rating-max`, `--vize-rating-count`, `--vize-rating-percent`                                                                                            |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
