# RadioGroup behavior contract

Normative state × input → outcome table for `radio-group.vue` and
`radio-group-item.vue` (`@vizejs/ui/radio-group`). Every row is proven by the
named mounted-DOM test in `src/families/selection/radio-group/radio-group.test.ts`; a row without a passing test
is a contract violation.

| #   | State                | Input             | Outcome                                                                                                                                              | Proven by                                                                     |
| --- | -------------------- | ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| R1  | named, selected      | render            | native radiogroup root with deterministic `id`, ARIA field state, orientation data/ARIA, and named native radio items with the selected item checked | `renders native radio group semantics with form and accessibility attributes` |
| R2  | uncontrolled         | pointer selection | selected value, checked item, native form value, root/item data state, and `update:modelValue` before `change` all follow the selected radio         | `uncontrolled radio group selects one item and submits its value`             |
| R3  | controlled           | pointer selection | emits the selection request and `change`; rendered checked state reverts to `modelValue` until the parent accepts the update                         | `controlled value wins until the parent accepts the request`                  |
| R4  | uncontrolled, seeded | form reset        | `defaultValue` seeds the initial selection and form reset restores it without request-global state                                                   | `defaultValue seeds state and native form reset restores it`                  |
| R5  | focusable            | Space / exposed   | native radio keyboard activation selects an item; `focus()` targets the checked item and item refs focus the native input                            | `keyboard activation and exposed focus follow native radio behavior`          |
| R6  | disabled             | render / Tab      | group-disabled items use native `disabled`, submit no value, emit no changes, and leave sequential focus                                             | `disabled groups and disabled items keep native availability semantics`       |
| R7  | missing provider     | item render       | `RadioGroupItem` outside a matching `RadioGroup` throws the stable context diagnostic                                                                | `items require a matching group provider`                                     |

The subpath remains tree-shakable and retains no packaged CSS; those package
contracts are pinned by `distribution.test.ts`, `check:size`, and
`check:tree-shaking`.
