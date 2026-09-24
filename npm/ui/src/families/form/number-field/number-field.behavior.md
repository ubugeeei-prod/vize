# NumberField behavior contract

Normative state x input -> outcome table for `number-field.vue`,
`number-field-input.vue`, `number-field-increment.vue`, and
`number-field-decrement.vue` (`@vizejs/ui/number-field`). The input follows the
WAI-ARIA APG spinbutton pattern. Every row is proven by the named test in
`number-field.test.ts`, `number-field-parser.test.ts`, or
`number-field-ssr.test.ts`; compile-only assertions live in
`number-field.types.test-d.ts`.

| #   | State                    | Input                               | Outcome                                                                                                                                                                   | Proven by                                                                  |
| --- | ------------------------ | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| N1  | named, bounded           | render                              | `role="spinbutton"` text input with deterministic id, locale text, `aria-valuenow/min/max/valuetext`, `inputmode`, required state, hidden raw-number form value, parts    | `renders an APG spinbutton with locale text, bounds, and form hooks`       |
| N2  | editable                 | type, then blur                     | partial text (`-`, `1,234.`) is kept without emitting; blur parses, commits, re-formats, emits `update:modelValue` then `change(value, previous, "blur")`; empty -> null  | `typing keeps partial text and commits the parsed value on blur`           |
| N3  | editable                 | type an impossible character        | the previous text is restored and `reject(text, event)` fires; a minus sign is rejected while `min >= 0`                                                                  | `rejects characters that can never form a number`                          |
| N4  | bounded                  | ArrowUp/Down, PageUp/Down, Home/End | arrows move to the adjacent step-grid value, pages move by `largeStep` (default `step * 10`), Home/End jump to finite bounds, all clamped; keys are prevented             | `arrow, page, home, and end keys step along the grid within bounds`        |
| N5  | unbounded                | Home / End                          | native caret movement is kept (not prevented); no `aria-valuemin/max`; signed fields use a text keyboard                                                                  | `unbounded fields keep native Home and End caret movement`                 |
| N6  | uncommitted typing       | arrow key                           | stepping starts from the typed text; decimal steps never accumulate floating-point error                                                                                  | `typed text is the stepping origin and precision stays decimal-safe`       |
| N7  | commit                   | blur / Enter                        | typed values clamp into bounds unless `clampOnCommit=false`; `snapOnCommit` snaps to the step grid; Enter commits with source `"enter"`                                   | `commits clamp by default and snap when requested`                         |
| N8  | locale + `formatOptions` | render / type                       | currency, percent (1% default step), and unit styles format and parse per locale; zero-fraction currencies reject decimals and use a numeric keyboard                     | `formats and parses currency, percent, and unit styles per locale`         |
| N9  | triggers                 | click / press-and-hold              | AT click steps once; pointer press keeps focus in the input, steps, then repeats after `holdDelay` every `holdInterval` until release/leave/cancel or a bound disables it | `triggers step on click, disable at bounds, and repeat while held`         |
| N10 | wheel                    | wheel                               | ignored unless `allowWheel`; when opted in, only a focused input steps and cancels page scrolling (non-passive listener)                                                  | `wheel stepping is opt-in and requires focus`                              |
| N11 | controlled               | step request                        | emits the request; the input keeps showing `modelValue` until the parent accepts it                                                                                       | `controlled value wins until the parent accepts the request`               |
| N12 | in a form                | submit / reset                      | the hidden input submits the raw number (not formatted text); form reset restores `defaultValue` and its formatted text                                                   | `submits the raw number and restores the default on form reset`            |
| N13 | disabled / read-only     | keys, typing, triggers              | disabled: native disabled input, triggers, and hidden value; read-only: focusable with `aria-readonly`, triggers disabled, no edits or emits                              | `disabled and read-only fields keep availability semantics`                |
| N14 | imperative               | expose                              | `focus`, `setValue` (clamped, `NaN` -> null), `increment(count)`, `decrement(count)`, `commit`, `reset`, and normalized state                                             | `exposes focus, setValue, increment, decrement, commit, and reset`         |
| N15 | inside `Field`           | `v-bind="fieldProps"` on root       | the spinbutton receives the Field id, `aria-labelledby`, `aria-describedby`, `aria-errormessage`, and `aria-invalid`                                                      | `binds Field fieldProps to wire the label, description, and error`         |
| N16 | part without provider    | setup                               | throws the stable `VIZE_UI_CONTEXT_MISSING: NumberField` diagnostic                                                                                                       | `parts require a NumberField provider`                                     |
| N17 | parser                   | format -> parse                     | round-trips across locales, native numbering systems, accounting currency, percent, and unit styles; partial prefixes are accepted while typing                           | `round-trips formatted text across locales, numbering systems, and styles` |
| N18 | SSR / hydration          | isolated requests                   | byte-identical markup, no `NaN`/`Infinity`, and hydration without diagnostics; hydrated triggers remain interactive                                                       | `renders byte-identical spinbutton markup and hydrates without mismatches` |
| N19 | inside `LocaleProvider`  | render                              | without a `locale` prop the field formats and parses with the provider locale                                                                                             | `inherits the locale from the nearest LocaleProvider`                      |

## Public extension contract

| Surface         | Contract                                                                                                                                                     |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Parts           | `root` (`<div>`), `input` (spinbutton), `increment`, `decrement` (`<button tabindex="-1">`).                                                                 |
| Data attributes | `data-vize-ui` (`number-field`, `number-field-input`, `number-field-increment`, `number-field-decrement`, `number-field-value`), `data-state` on root/input. |
| Boolean hooks   | Root: `data-disabled`, `data-readonly`, `data-required`, `data-invalid`, `data-empty`. Triggers: `data-disabled`, `data-holding`.                            |
| Slots           | Root default slot receives `NumberFieldSlotState`; trigger slots receive `{ direction, disabled, holding }`.                                                 |
| Field wiring    | Bind a Field's `fieldProps` to `NumberField`; the root forwards the id and ARIA relations to the spinbutton and triggers reference it via `aria-controls`.   |
| Locale          | `locale` prop, else the nearest `LocaleProvider`, else the document language, else `en-US`.                                                                  |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
