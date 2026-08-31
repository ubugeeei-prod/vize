# Switch behavior contract

Normative state × input → outcome table for `switch-control.vue` (`@vizejs/ui/switch`).
Every row is proven by the named mounted-DOM test in `src/families/selection/switch/switch.test.ts`; a row
without a passing test is a contract violation.

| #   | State                | Input         | Outcome                                                                                                                                     | Proven by                                                            |
| --- | -------------------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| S1  | named, checked       | render        | native `<button type="button" role="switch">`, deterministic `id`, explicit `aria-checked`, ARIA field state, and checked hidden form value | `renders a named native switch with ARIA and form attributes`        |
| S2  | uncontrolled         | pointer click | toggles `aria-checked`, `data-state`, `data-checked`, and checked form value; emits `update:modelValue` before `change`                     | `uncontrolled switch toggles with pointer activation and form data`  |
| S3  | controlled           | pointer click | emits the request and `change`; rendered checked state reverts to `modelValue` until the parent accepts the update                          | `controlled checked state wins until the parent accepts the request` |
| S4  | uncontrolled, seeded | form reset    | `defaultChecked` seeds the initial state and form reset restores it without request-global state                                            | `defaultChecked seeds state and native form reset restores it`       |
| S5  | focusable            | Enter / Space | native button keyboard activation toggles the switch from both Enter and Space                                                              | `keyboard activation toggles with Enter and Space`                   |
| S6  | disabled             | click / Tab   | native `disabled`, `aria-disabled`, `data-state="disabled"`, no checked form value, no toggle, and no sequential focus                      | `disabled and read-only switches keep availability semantics`        |
| S7  | read-only            | click / Tab   | `aria-readonly`, `data-state="readonly"`, remains focusable, preserves form value, and suppresses user toggles                              | `disabled and read-only switches keep availability semantics`        |
| S8  | uncontrolled         | exposed API   | `toggle()` and `setChecked()` update state, `focus()` focuses the button, and `reset()` restores the default checked state                  | `exposes focus, toggle, setChecked, reset, and slot state`           |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
