# Search Field behavior contract

Normative state × input → outcome table for `search-field.vue` (`@vizejs/ui/search-field`).
Every row is proven by the named mounted-DOM test in `src/search-field.test.ts`;
a row without a passing test is a contract violation.

| #   | State                | Input                 | Outcome                                                                                                                               | Proven by                                                                          |
| --- | -------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| S1  | editable, native     | render                | `role="search"` root, native searchbox, deterministic input/clear ids, search keyboard hints, form attributes, and invalid ARIA state | `renders a named native searchbox with root landmark and accessibility attributes` |
| S2  | uncontrolled         | input / change/search | value follows the native element; emits `update:modelValue` before `input`, `change`, and `search`                                    | `uncontrolled search field emits model before input, change, and search`           |
| S3  | controlled           | native input          | emits the request; the rendered value reverts to `modelValue` until the parent accepts the update                                     | `controlled search value wins until the parent accepts the request`                |
| S4  | uncontrolled, seeded | form reset            | `defaultValue` seeds the initial value and form reset restores it without request-global state                                        | `defaultValue seeds state and native form reset restores it`                       |
| S5  | clearable            | default clear button  | default clear emits `update:modelValue` before `clear`, empties the field, and returns focus to the searchbox                         | `clear button updates before clear event and returns focus to the searchbox`       |
| S6  | unavailable          | render / Tab          | clear availability and focus follow empty, disabled, and readonly state while preserving native input semantics                       | `clear visibility and availability follow empty, disabled, and readonly state`     |
| S7  | composing            | IME composition       | native text is not rewritten during composition; real `CompositionEvent` payloads are emitted and reconciliation happens afterward    | `tracks IME composition without rewriting controlled native text`                  |
| S8  | uncontrolled         | exposed methods       | `setValue()`, `clear()`, `select()`, `focus()`, and `reset()` update the native searchbox without bundled CSS                         | `exposes value mutation, clear, selection, focus, and reset controls`              |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
