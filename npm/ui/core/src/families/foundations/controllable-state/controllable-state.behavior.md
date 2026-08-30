# Controlled state behavior contract

Normative state x input -> outcome table for `@vizejs/ui/controllable-state`.
Every row is proven by
`src/families/foundations/controllable-state/controllable-state.test.ts`;
compile-only assertions live in
`src/families/foundations/controllable-state/controllable-state.types.test-d.ts`.

| #   | State            | Input                    | Outcome                                                                        | Proven by                                                                        |
| --- | ---------------- | ------------------------ | ------------------------------------------------------------------------------ | -------------------------------------------------------------------------------- |
| CS1 | uncontrolled     | set updater              | internal value updates and `onChange` receives next and previous values        | `updates and resets uncontrolled state`                                          |
| CS2 | uncontrolled     | reset                    | current default value is requested and change ordering remains observable      | `updates and resets uncontrolled state`                                          |
| CS3 | controlled       | set                      | source value is not mutated; the change request is reported to the consumer    | `requests controlled updates without mutating...`                                |
| CS4 | controlled       | source update            | rendered value follows the external source                                     | `requests controlled updates without mutating...`                                |
| CS5 | control released | source becomes undefined | the last controlled value is retained as internal state                        | `retains the last value when control is released`                                |
| CS6 | custom equality  | equivalent update        | redundant writes are rejected without replacing the stored value               | `supports domain-specific equality...`                                           |
| CS7 | public type API  | invalid updater/defaults | compile-only assertions preserve the value type through reads, sets, and reset | `src/families/foundations/controllable-state/controllable-state.types.test-d.ts` |

The primitive performs no DOM work and emits no CSS. Concrete controls remain
responsible for native form integration, accessibility state, and reset events.
