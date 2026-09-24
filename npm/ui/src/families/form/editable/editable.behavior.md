# Editable behavior contract

Normative state x input -> outcome table for `editable.vue`,
`editable-preview.vue`, `editable-input.vue`, and `editable-trigger.vue`
(`@vizejs/ui/editable`). Every row is proven by the named test in
`editable.test.ts` or `editable-ssr.test.ts`; compile-only assertions live in
`editable.types.test-d.ts`.

| #   | State                        | Input                 | Outcome                                                                                                                                          | Proven by                                                                            |
| --- | ---------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------ |
| E1  | preview, named               | render                | focusable `role="button"` preview with the value; input and submit/cancel triggers are `hidden`; hidden form value                               | `renders a focusable preview with the input and editing triggers hidden`             |
| E2  | preview, `focus` mode        | focus preview         | enters edit mode, emits `update:editing` then `edit`, focuses the input and selects its text                                                     | `focusing the preview enters edit mode and focuses the selected input`               |
| E3  | editing                      | Enter / Escape        | Enter commits (`submit(value, previous)`), Escape discards (`cancel(value, discarded)`); focus returns to the preview without reopening          | `Enter submits, Escape cancels, and focus returns to the preview without reopening`  |
| E4  | editing, `submitMode`        | blur / Enter          | `both` (default) submits on blur and Enter; `enter` ignores blur; `none` requires an explicit submit                                             | `blur submits by default and submitMode controls Enter and blur`                     |
| E5  | `activationMode="none"`      | triggers              | focus and clicks no longer activate (Enter/F2 still do); edit/submit/cancel triggers act, and pointer presses keep focus in the input            | `triggers edit, submit, and cancel while keeping focus in the input`                 |
| E6  | click / dblclick modes       | click / dblclick / F2 | the preview activates only on its mode; Enter and F2 always activate a focused preview                                                           | `click and double-click activation modes`                                            |
| E7  | controlled                   | focus / submit        | controlled `editing` and `modelValue` win until the parent accepts them                                                                          | `controlled value and editing win until the parent accepts them`                     |
| E8  | empty / disabled / read-only | render / activation   | placeholder shows when empty; disabled and read-only previews stay focusable with `aria-disabled`, triggers are disabled, and nothing is emitted | `placeholder, disabled, and read-only states`                                        |
| E9  | in a form / imperative       | submit / expose       | the hidden input submits the committed value; `edit`, `submit`, `cancel`, `setValue`, and state are exposed                                      | `submits the committed value with a form and exposes edit, submit, cancel, setValue` |
| E10 | part without provider        | setup                 | throws the stable `VIZE_UI_CONTEXT_MISSING: Editable` diagnostic                                                                                 | `parts require an Editable provider`                                                 |
| E11 | SSR / hydration              | isolated requests     | byte-identical markup with `hidden` parts, hydration without diagnostics, and interactive preview                                                | `renders byte-identical inline-edit markup and hydrates without mismatches`          |

## Public extension contract

| Surface         | Contract                                                                                                       |
| --------------- | -------------------------------------------------------------------------------------------------------------- |
| Parts           | `root`, `preview`, `input`, and `edit-trigger` / `submit-trigger` / `cancel-trigger`.                          |
| Data attributes | `data-vize-ui`, `data-state` (`preview`/`editing`/`readonly`/`disabled`), `data-empty`, trigger `data-action`. |
| Visibility      | Parts toggle the native `hidden` attribute, so SSR markup and hydration never swap nodes.                      |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
