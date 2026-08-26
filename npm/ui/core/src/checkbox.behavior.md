# Checkbox behavior contract

Normative state × input → outcome table for `checkbox-control.vue` (`@vizejs/ui/checkbox`).
Every row is proven by the named mounted-DOM test in `src/checkbox.test.ts`; a row
without a passing test is a contract violation.

| #   | State                | Input              | Outcome                                                                       | Proven by                                                             |
| --- | -------------------- | ------------------ | ----------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| C1  | any                  | render             | native `input[type=checkbox]`, accessible name from `ariaLabel`               | `renders a native checkbox with an accessible name and focus control` |
| C2  | any                  | exposed `focus()`  | input receives focus                                                          | `renders a native checkbox with an accessible name and focus control` |
| C3  | unchecked            | render             | `aria-checked="false"`, `data-state="unchecked"`                              | `reports aria-checked across unchecked, checked, and mixed states`    |
| C4  | checked              | render             | `aria-checked="true"`, `data-state="checked"`                                 | `reports aria-checked across unchecked, checked, and mixed states`    |
| C5  | indeterminate        | render             | `aria-checked="mixed"`, `data-state="indeterminate"`, native `.indeterminate` | `reports aria-checked across unchecked, checked, and mixed states`    |
| C6  | uncontrolled         | pointer click      | toggles; emits `update:modelValue` then `change`, in that order               | `toggles with a pointer click and emits model before change`          |
| C7  | uncontrolled         | Space              | toggles exactly once like a native checkbox                                   | `toggles with Space like a native checkbox`                           |
| C8  | wrapped in `<label>` | click on the label | toggles the checkbox (label association)                                      | `clicking the associated label toggles the checkbox`                  |
| C9  | controlled           | pointer click      | emits the request; the rendered state reverts to the prop value               | `controlled: the parent-provided value always wins`                   |
| C10 | controlled           | prop update        | rendered state follows `modelValue`                                           | `controlled: the parent-provided value always wins`                   |
| C11 | uncontrolled, seeded | render             | `defaultChecked` seeds the initial state                                      | `uncontrolled: defaultChecked seeds state and reset restores it`      |
| C12 | uncontrolled, seeded | exposed `reset()`  | restores the default state                                                    | `uncontrolled: defaultChecked seeds state and reset restores it`      |
| C13 | indeterminate        | pointer click      | emits `update:indeterminate` `false` and `change` `true`                      | `indeterminate announces mixed and requests clearing on toggle`       |
| C14 | disabled             | click / Space      | no toggle, no `change`                                                        | `disabled checkbox ignores pointer and keyboard activation`           |
| C15 | disabled             | Tab                | skipped by the tab order                                                      | `disabled checkbox ignores pointer and keyboard activation`           |

Mixed-state precedence (`indeterminate` wins over `checked`) is additionally
pinned as pure logic by `gives the mixed visual state precedence`.
