# Input mask behavior contract

Normative state x input -> outcome table for `masked-input.vue`, the
`useInputMask` composable, and the pure `createInputMask` engine
(`@vizejs/ui/input-mask`). Every row is proven by the named test in
`input-mask.test.ts` or `input-mask-ssr.test.ts`; compile-only assertions live
in `input-mask.types.test-d.ts`.

| #   | State                   | Input                            | Outcome                                                                                                                                        | Proven by                                                                            |
| --- | ----------------------- | -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| M1  | lazy mask               | conform typed/pasted/masked text | accepted characters fill token slots in order, literals are inserted only before filled slots, rejects are skipped, `complete` when all filled | `conforms typed, pasted, and pre-masked text to token slots and literals`            |
| M2  | eager / placeholder     | conform                          | `eager` appends following literals; `lazy=false` shows placeholders; `\` escapes tokens; custom tokens transform and keep literal keys         | `supports eager literals, placeholders, escapes, and custom typed tokens`            |
| M3  | composable, no instance | `setValue` / controlled value    | runs in any effect scope, reports changes and a single completion, controlled values win                                                       | `useInputMask works outside components and reports completion`                       |
| M4  | seeded                  | render                           | native text input with deterministic id, masked value, numeric `inputmode` for digit masks, form and ARIA attributes, `data-state`             | `renders a native text input with mask, form, and accessibility hooks`               |
| M5  | focused                 | type / paste                     | value conforms, caret lands after the last affected slot, `update:modelValue` emits raw text, `complete` fires once                            | `typing conforms the value, keeps the caret after the slot, and emits once complete` |
| M6  | caret next to a literal | Backspace / Delete               | deleting only a literal removes the neighboring slot character instead of being undone                                                         | `Backspace and Delete across a literal remove the neighboring slot character`        |
| M7  | `valueFormat="masked"`  | type / controlled                | model value includes literals and placeholders; controlled values win until accepted                                                           | `masked value format, placeholders, and controlled values`                           |
| M8  | in a form               | reset / disabled / read-only     | form reset restores the default display; disabled and read-only publish their `data-state`                                                     | `form reset restores the default and disabled or read-only states publish`           |
| M9  | imperative              | expose                           | `focus`, `setValue`, `reset`, `masked`, `raw`, `complete`, `element`                                                                           | `exposes focus, setValue, reset, and mask state`                                     |
| M10 | inside `Field`          | `v-bind="fieldProps"`            | id and ARIA relations reach the native input                                                                                                   | `binds Field fieldProps for label, description, and error wiring`                    |
| M11 | SSR / hydration         | isolated requests                | byte-identical markup (including placeholders) and hydration without diagnostics; hydrated input stays interactive                             | `renders byte-identical masked markup and hydrates without mismatches`               |

## Public extension contract

| Surface         | Contract                                                                                     |
| --------------- | -------------------------------------------------------------------------------------------- |
| Parts           | `input` on the native `<input type="text">`.                                                 |
| Data attributes | `data-vize-ui="masked-input"`, `data-state`, `data-complete` (`"true"`/`"false"`).           |
| Tokens          | `9` digit, `a` ASCII letter, `*` ASCII letter or digit; extend with `defineInputMaskTokens`. |
| Composable      | `useInputMask` binds any input via `:value="masked"` and `@input="handleInput"`.             |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
