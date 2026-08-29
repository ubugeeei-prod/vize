# Toggle behavior contract

Normative state × input → outcome table for `toggle-button.vue` (`@vizejs/ui/toggle`).
Every row is proven by the named mounted-DOM test in `src/toggle.test.ts`; a row
without a passing test is a contract violation.

| #   | State                | Input                  | Outcome                                                                                                                                   | Proven by                                                       |
| --- | -------------------- | ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| T1  | unpressed, native    | render                 | native `<button type="button">`, accessible name from slot, `aria-pressed="false"`, `data-vize-ui="toggle"`                               | `renders a native toggle button with pressed semantics`         |
| T2  | uncontrolled         | pointer click          | toggles `aria-pressed` and `data-state`; emits `update:modelValue` then `change`, in that order                                           | `toggles with pointer activation and emits the requested value` |
| T3  | controlled           | pointer click          | emits the request; the rendered state keeps the parent-provided `modelValue` until props update                                           | `controlled value wins until the parent accepts the request`    |
| T4  | uncontrolled, seeded | render / reset         | `defaultPressed` seeds the initial state; exposed `reset()` restores it                                                                   | `uncontrolled defaultPressed seeds state and reset restores it` |
| T5  | idle, non-native     | Enter / Space          | non-native rendering exposes `role="button"` and emulates native button keyboard timing with one requested toggle per activation key      | `non-native toggle emulates Enter and Space activation timing`  |
| T6  | disabled, native     | click / Enter / Space  | native `disabled` attribute, no `aria-disabled` mirror, no toggle, no `change`, skipped by Tab                                            | `disabled native and non-native toggles suppress activation`    |
| T7  | disabled, non-native | click / Space / Tab    | `tabindex="-1"`, `aria-disabled="true"`, no toggle, no `change`, skipped by Tab                                                           | `disabled native and non-native toggles suppress activation`    |
| T8  | any                  | slot / exposed methods | slot receives live `pressed` and `disabled` booleans; `focus()` focuses the control and `setPressed()` updates uncontrolled pressed state | `exposes focus, setPressed, and slot state`                     |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
