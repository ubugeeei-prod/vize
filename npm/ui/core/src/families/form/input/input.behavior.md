# Input behavior contract

Normative state × input → outcome table for `text-input.vue` (`@vizejs/ui/input`).
Every row is proven by the named mounted-DOM test in
`src/families/form/input/input.test.ts`; a row
without a passing test is a contract violation.

| #   | State                | Input                 | Outcome                                                                                                                          | Proven by                                                               |
| --- | -------------------- | --------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| I1  | editable, native     | render                | native text-like `<input>`, deterministic `id`, accessible name, form attributes, `data-vize-ui="input"`, and invalid ARIA state | `renders a named native input with form and accessibility attributes`   |
| I2  | uncontrolled         | native input / change | value follows the native element; emits `update:modelValue` before `input`, and emits `change` with the committed string         | `uncontrolled input emits model before input and reports native change` |
| I3  | controlled           | native input          | emits the request; the rendered value reverts to `modelValue` until the parent accepts the update                                | `controlled value wins until the parent accepts the request`            |
| I4  | uncontrolled, seeded | form reset            | `defaultValue` seeds the initial value and form reset restores it without request-global state                                   | `defaultValue seeds state and native form reset restores it`            |
| I5  | disabled             | render / Tab          | native `disabled`, `data-state="disabled"`, no sequential focus                                                                  | `disabled and read-only inputs keep native availability semantics`      |
| I6  | read-only            | render / Tab          | native `readonly`, `data-state="readonly"`, remains focusable                                                                    | `disabled and read-only inputs keep native availability semantics`      |
| I7  | composing            | IME composition       | `data-composing` and exposed `composing` track composition start/end while preserving the composed string                        | `tracks IME composition without losing the composed value`              |
| I8  | uncontrolled         | exposed methods       | `setValue()` updates uncontrolled state, `select()` selects text, `focus()` focuses the input, and `reset()` restores default    | `exposes value mutation, selection, focus, and reset controls`          |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
