# Descriptions behavior contract

Normative state × input → outcome table for `src/components/descriptions/descriptions.vue`
and `src/components/descriptions/descriptions-item.vue`.
Every row is proven by `src/components/descriptions/tests/behavior.test.ts`.

| #   | State        | Input  | Outcome                                                             | Proven by                                            |
| --- | ------------ | ------ | ------------------------------------------------------------------- | ---------------------------------------------------- |
| D1  | labeled list | render | `dl` with `role="group"` named by `label`                           | `names the list and pairs each term with its detail` |
| D2  | item         | render | `role="term"` and `role="definition"` expose the label and the slot | `names the list and pairs each term with its detail` |
| D3  | item slot    | render | the slot receives the term label                                    | `passes the term to the detail slot`                 |
