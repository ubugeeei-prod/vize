# Fieldset behavior contract

Normative state x input -> outcome table for `fieldset.vue`,
`fieldset-legend.vue`, `fieldset-description.vue`, and
`fieldset-error-message.vue` (`@vizejs/ui/fieldset`). The group is a native
`<fieldset>` named by its `<legend>`; ids come from the shared field-wiring
foundation. Every row is proven by the named test in `fieldset.test.ts` or
`fieldset-ssr.test.ts`; compile-only assertions live in `fieldset.types.test-d.ts`.

| #   | State                 | Input                               | Outcome                                                                                                                  | Proven by                                                                        |
| --- | --------------------- | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| FS1 | valid, described      | render                              | native `<fieldset>` with id and `name`, first-child `<legend>`, description id joined into `aria-describedby`, no error  | `renders a native fieldset named by its legend and described by its description` |
| FS2 | matching form error   | `errors` with the group `name`      | group becomes invalid, error message renders with the derived id and joins `aria-describedby`; emits `invalid-change`    | `matching form errors mark the group invalid and reference the error message`    |
| FS3 | override / disabled   | `invalid`, `forceMount`, `disabled` | `invalid` overrides errors; `forceMount` keeps the message; `disabled` is the native attribute that disables descendants | `invalid override, forceMount, and native disabled propagation`                  |
| FS4 | part without provider | setup                               | throws the stable `VIZE_UI_CONTEXT_MISSING: Fieldset` diagnostic                                                         | `parts require a Fieldset provider`                                              |
| FS5 | SSR / hydration       | isolated requests                   | byte-identical markup with deterministic ids and relations; hydration without diagnostics                                | `renders byte-identical fieldset markup and hydrates without mismatches`         |

## Public extension contract

| Surface         | Contract                                                                                             |
| --------------- | ---------------------------------------------------------------------------------------------------- |
| Parts           | `root` (fieldset), `legend`, `description`, `error-message`.                                         |
| Data attributes | `data-vize-ui`, root `data-state` (`valid`/`invalid`/`disabled`), `data-invalid` on root and legend. |
| Ids             | `<id>-description` and `<id>-error`, matching `useFieldWiring` and `useFormFieldProps` conventions.  |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
