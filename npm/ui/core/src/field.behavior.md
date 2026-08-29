# Field behavior contract

Normative state x input -> outcome table for `field.vue`, `field-label.vue`,
`field-description.vue`, and `field-error-message.vue` (`@vizejs/ui/field`).
Every row is proven by the named test in `src/field.test.ts` or
`src/field-ssr.test.ts`; compile-only assertions live in
`src/field.types.test-d.ts`.

| ID  | State        | Input                                    | Outcome                                                                 | Test                                                       |
| --- | ------------ | ---------------------------------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------- |
| FC1 | valid        | render with label and control            | FieldLabel id and `for` match Field slot `fieldProps`                   | `wires label and described-by props through public SFCs`   |
| FC2 | valid        | `hasDescription` with FieldDescription   | control receives the description id in `aria-describedby`               | `wires label and described-by props through public SFCs`   |
| FC3 | field error  | matching normalized error by `name`      | Field becomes invalid, error message renders, and ARIA error props bind | `renders normalized form errors and emits invalid changes` |
| FC4 | forced error | `invalid` prop without matching errors   | Field becomes invalid while the message slot remains consumer-owned     | `allows direct invalid overrides for native validation`    |
| FC5 | invalid      | `hasDescription=false`                   | description element can render, but the control omits its id            | `suppresses optional ARIA relations when declared absent`  |
| FC6 | invalid      | `hasErrorMessage=false`                  | error element can render, but control omits `aria-errormessage`         | `suppresses optional ARIA relations when declared absent`  |
| FC7 | no provider  | Field part rendered outside Field        | setup throws the stable missing-context diagnostic                      | `rejects field parts outside a Field root`                 |
| FC8 | SSR          | isolated request renders with IdProvider | byte-identical control, label, description, and error ids               | SSR test                                                   |
| FC9 | public types | malformed props and slot/expose misuse   | compilation rejects misuse                                              | `src/field.types.test-d.ts`                                |

## Public contract

- `Field` renders `data-vize-ui="field"`, `part="root"`, `data-state`,
  `data-invalid`, and `data-name`.
- `FieldLabel` renders `data-vize-ui="field-label"`, `part="label"`,
  `data-state`, `data-invalid`, and `data-name`.
- `FieldDescription` renders `data-vize-ui="field-description"`,
  `part="description"`, `data-state`, `data-invalid`, and `data-name`.
- `FieldErrorMessage` renders `data-vize-ui="field-error-message"`,
  `part="error-message"`, `data-state`, `data-invalid`, and `data-name` while
  the field is invalid or `forceMount=true`.
- `Field` exposes no CSS custom properties and ships no opinionated styles.
- `hasDescription` is the SSR-safe declaration that a description element is
  present. `FieldDescription` does not mutate the parent relation implicitly.
- `hasErrorMessage` is the SSR-safe declaration that an error message element
  is present while invalid. Set it to `false` for native-only invalid state.
