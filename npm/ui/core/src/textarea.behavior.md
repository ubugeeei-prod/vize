# Textarea behavior contract

Normative state × input → outcome table for `textarea-control.vue` (`@vizejs/ui/textarea`).
Every row is proven by the named mounted-DOM test in `src/textarea.test.ts`; a
row without a passing test is a contract violation.

| #   | State                | Input                 | Outcome                                                                                                                          | Proven by                                                                  |
| --- | -------------------- | --------------------- | -------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| T1  | editable, native     | render                | native `<textarea>`, deterministic `id`, accessible name, form attributes, line constraints, and invalid ARIA state              | `renders a named native textarea with form and accessibility attributes`   |
| T2  | uncontrolled         | native input / change | value follows the native element; emits `update:modelValue` before `input`, and emits `change` with the committed string         | `uncontrolled textarea emits model before input and reports native change` |
| T3  | controlled           | native input          | emits the request; the rendered value reverts to `modelValue` until the parent accepts the update                                | `controlled value wins until the parent accepts the request`               |
| T4  | uncontrolled, seeded | form reset            | `defaultValue` seeds the initial value and form reset restores it without request-global state                                   | `defaultValue seeds state and native form reset restores it`               |
| T5  | disabled             | render / Tab          | native `disabled`, `data-state="disabled"`, no sequential focus                                                                  | `disabled and read-only textareas keep native availability semantics`      |
| T6  | read-only            | render / Tab          | native `readonly`, `data-state="readonly"`, remains focusable                                                                    | `disabled and read-only textareas keep native availability semantics`      |
| T7  | composing            | IME composition       | `data-composing` and exposed `composing` track composition start/end while preserving the composed string                        | `tracks IME composition without losing the composed multiline value`       |
| T8  | uncontrolled         | exposed methods       | `setValue()` updates uncontrolled state, `select()` and `setSelectionRange()` update text selection, `focus()` focuses the field | `exposes value mutation, selection range, focus, and reset controls`       |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
