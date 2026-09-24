# PasswordField behavior contract

Normative state x input -> outcome table for `password-field.vue`,
`password-field-input.vue`, and `password-field-toggle.vue`
(`@vizejs/ui/password-field`). The root is generic over the strength hook's
return type. Every row is proven by the named test in `password-field.test.ts`
or `password-field-ssr.test.ts`; compile-only assertions live in
`password-field.types.test-d.ts`.

| #   | State              | Input                      | Outcome                                                                                                                                       | Proven by                                                                    |
| --- | ------------------ | -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| P1  | any password       | `estimatePasswordStrength` | 0-4 score and label from length and character variety; short passwords capped at "weak"; Unicode-aware checks                                 | `scores length and character variety with a dependency-free heuristic`       |
| P2  | named, seeded      | render                     | native `type="password"` with `current-password` autocomplete, no autocapitalize/spellcheck, and a `type="button"` toggle with `aria-pressed` | `renders a native password input with sign-in autocomplete and a toggle`     |
| P3  | hidden             | toggle press               | pointer press keeps focus in the input; toggling swaps `type`, `aria-pressed`, the toggle label, and `data-state`, emitting `update:visible`  | `the toggle reveals and hides the password while keeping focus in the input` |
| P4  | controlled         | toggle / typing            | controlled `visible` and `modelValue` win until the parent accepts them                                                                       | `controlled visibility and value win until the parent accepts them`          |
| P5  | typing             | key events / blur          | Caps Lock state follows `getModifierState("CapsLock")`, publishes `data-caps-lock`, emits `capsLockChange`, and clears on blur                | `detects Caps Lock from key events and clears it on blur`                    |
| P6  | strength hook      | typing                     | `evaluateStrength` runs on every change and types the slot's `strength`; without it `strength` is `undefined`                                 | `the strength hook evaluates every change and types the slot`                |
| P7  | in a form          | reset                      | form reset restores `defaultValue` and hides the password                                                                                     | `form reset restores the password and hides it again`                        |
| P8  | disabled/read-only | typing / toggle            | disabled locks input and toggle; read-only keeps the value but still allows revealing it                                                      | `disabled and read-only fields lock editing and the toggle appropriately`    |
| P9  | imperative         | expose                     | `focus`, `setValue`, `setVisible`, `toggleVisible`, `reset`                                                                                   | `exposes focus, setValue, setVisible, toggleVisible, and reset`              |
| P10 | inside `Field`     | `v-bind="fieldProps"`      | id and ARIA relations reach the input; parts outside a PasswordField throw `VIZE_UI_CONTEXT_MISSING`                                          | `binds Field fieldProps and requires a PasswordField provider for parts`     |
| P11 | SSR / hydration    | isolated requests          | byte-identical markup including slot-rendered strength, hydration without diagnostics, interactive toggle                                     | `renders byte-identical password markup and hydrates without mismatches`     |

## Public extension contract

| Surface         | Contract                                                                                                        |
| --------------- | --------------------------------------------------------------------------------------------------------------- |
| Parts           | `root`, `input`, `toggle`.                                                                                      |
| Data attributes | `data-vize-ui`, root `data-state` (`hidden`/`visible`/`readonly`/`disabled`), `data-visible`, `data-caps-lock`. |
| Slots           | Root default slot receives `PasswordFieldSlotState<Strength>`; toggle slot receives `{ visible }`.              |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
