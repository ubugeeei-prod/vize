# Result behavior contract

Normative state × input → outcome table for `src/components/result/result.vue`.
Every row is proven by `src/components/result/tests/behavior.test.ts`.

| #   | State               | Input        | Outcome                                                    | Proven by                                           |
| --- | ------------------- | ------------ | ---------------------------------------------------------- | --------------------------------------------------- |
| R1  | error               | render       | `role="alert"`, `data-status="error"`, name from the title | `announces an error as an alert`                    |
| R2  | success             | render       | `role="status"`, not an alert                              | `announces success as status`                       |
| R3  | not-found, empty    | render       | both use `role="status"`                                   | `renders not-found and empty as status, not alerts` |
| R4  | description omitted | render       | no description node                                        | `omits an empty description`                        |
| R5  | any                 | default slot | slot receives the live status                              | `passes the status to the actions slot`             |
