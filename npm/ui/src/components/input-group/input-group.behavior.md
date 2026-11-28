# Input Group behavior contract

Normative state × input → outcome table for `src/components/input-group/input-group.vue`,
`src/components/input-group/input-group-addon.vue`,
`src/components/input-group/input-group-input.vue`,
`src/components/input-group/input-group-textarea.vue`, and
`src/components/input-group/input-group-button.vue`.
Every row is proven by `src/components/input-group/tests/behavior.test.ts`.

| #   | State          | Input  | Outcome                                                                       | Proven by                                           |
| --- | -------------- | ------ | ----------------------------------------------------------------------------- | --------------------------------------------------- |
| I1  | enabled group  | render | group, addon, text field, textarea, and button are distinct parts             | `renders addons, fields, and a button in one group` |
| I2  | text field     | edit   | `update:modelValue` carries the next string                                   | `emits the next field value`                        |
| I3  | button         | click  | one `press`                                                                   | `emits press from the group button`                 |
| I4  | group disabled | render | the field, textarea, and button are disabled and the group is `aria-disabled` | `disables every control from the group`             |
