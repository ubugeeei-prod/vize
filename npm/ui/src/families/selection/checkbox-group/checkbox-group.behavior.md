# CheckboxGroup behavior contract

Normative state x input -> outcome table for `checkbox-group.vue`,
`checkbox-group-item.vue`, and `checkbox-group-select-all.vue`
(`@vizejs/ui/checkbox-group`). The group is generic: `options: readonly Value[]`
infers `Value`, so `v-model` is `readonly Value[]` for strings, unions, or
objects. The select-all parent follows the WAI-ARIA APG tri-state checkbox.
Every row is proven by the named test in `checkbox-group.test.ts` or
`checkbox-group-ssr.test.ts`; inference and misuse are pinned in
`checkbox-group.types.test-d.ts`.

| #   | State                 | Input                   | Outcome                                                                                                                                           | Proven by                                                                         |
| --- | --------------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| C1  | named, seeded         | render                  | labelled `role="group"` of native checkboxes sharing `name`; primitive options submit `String(value)`; summary `data-state` and slot state        | `renders a labelled group of named native checkboxes with typed values`           |
| C2  | uncontrolled objects  | toggle items            | values stay in option order, `update:modelValue` then `change(values, toggled, selected)`; object options submit their index by default           | `toggling items updates values in option order and emits changes`                 |
| C3  | partial selection     | select-all              | parent is `mixed`/indeterminate while some enabled options are selected; toggling selects or clears enabled options and keeps disabled selections | `select-all is tri-state and toggles every enabled option`                        |
| C4  | controlled with `by`  | toggle                  | fresh object copies match options by key; `getFormValue` controls submitted values; the controlled value wins until accepted                      | ``controlled values win and `by` matches fresh object copies``                    |
| C5  | in a form, `required` | submit / toggle / reset | submits every selected value; every box is `required` only while nothing is selected; form reset restores `defaultValue`                          | `submits selected values, requires one selection, and restores defaults on reset` |
| C6  | disabled group        | toggle / API            | every checkbox (and select-all) is natively disabled, `aria-disabled` on the group, and no changes are emitted                                    | `disabled groups disable every checkbox and ignore toggles`                       |
| C7  | imperative            | expose                  | `isSelected`, `setSelected` (unknown values ignored), `setAll`, `reset`, and the selection summary                                                | `exposes isSelected, setSelected, setAll, reset, and the selection summary`       |
| C8  | part without provider | setup                   | throws the stable `VIZE_UI_CONTEXT_MISSING: CheckboxGroup` diagnostic                                                                             | `items and select-all require a CheckboxGroup provider`                           |
| C9  | SSR / hydration       | isolated requests       | byte-identical markup (checked + `aria-checked="mixed"`), hydration applies `indeterminate` and stays interactive                                 | `renders byte-identical checkbox group markup and hydrates without mismatches`    |

## Public extension contract

| Surface         | Contract                                                                                                                |
| --------------- | ----------------------------------------------------------------------------------------------------------------------- |
| Parts           | `root` (group), `item`, `select-all` (native checkboxes; wrap them in `<label>` for names).                             |
| Data attributes | `data-vize-ui`, root `data-state` (`all`/`some`/`none`/`disabled`), `data-orientation`; item `data-index`/`data-state`. |
| Identity        | Items resolve `value` to an option by identity (through reactive proxies); `by` compares model values with options.     |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
