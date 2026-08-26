# Field wiring behavior contract

Normative state × input → outcome table for `useFieldWiring`
(`@vizejs/ui/field-wiring`). Every row is proven by the named test in
`src/field-wiring.test.ts` or `src/field-wiring-ssr.test.ts`; compile-only
assertions live in `src/field-wiring.types.test-d.ts`.

| #   | State        | Input                       | Outcome                                                              | Proven by                                          |
| --- | ------------ | --------------------------- | -------------------------------------------------------------------- | -------------------------------------------------- |
| W1  | valid        | render                      | label `for`/`id` and control `aria-labelledby` agree                 | `wires the label to the control`                   |
| W2  | valid        | no description              | control has no `aria-describedby`                                    | `wires the label to the control`                   |
| W3  | valid        | `hasDescription`            | `aria-describedby` names the description element                     | `describes the control while a description exists` |
| W4  | valid        | `invalid` becomes true      | `aria-invalid`, `aria-errormessage`, and described-by error id apply | `wires the error message while invalid`            |
| W5  | invalid      | `invalid` becomes false     | error wiring is removed, description wiring remains                  | `wires the error message while invalid`            |
| W6  | invalid      | `hasErrorMessage` false     | invalid state applies without dangling error wiring                  | `omits error wiring without an error element`      |
| W7  | any          | explicit `id`               | every derived id follows the consumer-owned id                       | `derives every id from a consumer-owned id`        |
| W8  | no setup     | `useFieldWiring`            | throws a stable setup diagnostic                                     | `rejects use outside component setup`              |
| W9  | any          | non-boolean option          | throws a stable option diagnostic                                    | `rejects invalid options`                          |
| W10 | SSR          | render under an IdProvider  | byte-identical deterministic wiring                                  | SSR test                                           |
| W11 | public types | mutation or invalid options | compilation rejects misuse                                           | `src/field-wiring.types.test-d.ts`                 |
