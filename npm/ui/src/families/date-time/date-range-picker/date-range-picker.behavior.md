# DateRangePicker behavior contract

Normative state x input -> outcome table for the DateRangePicker compound
(`@vizejs/ui/date-range-picker`): `date-range-picker-root.vue`,
`date-range-picker-field.vue` (one per `boundary`),
`date-range-picker-content.vue`, and `date-range-picker-calendar.vue`, with
`DateRangePickerTrigger` re-exporting PopoverTrigger. Popover, field, and
range-calendar rules follow their own contracts. Every row is proven by the
named test.

| #   | State          | Input                          | Outcome                                                                                                                                      | Proven by                                                                                                                            |
| --- | -------------- | ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Q1  | closed         | trigger / two day activations  | the popover opens focused on today, the range commits in order, the popover closes, and both fields plus `startName`/`endName` inputs update | `renders start and end fields and commits a two-click calendar range`                                                                |
| Q2  | complete range | clear one field / type it back | clearing an endpoint commits `null` but keeps the other draft; typing it back commits an ordered range even when entered reversed            | `editing one field keeps a draft endpoint until both are known and orders the range`                                                 |
| Q3  | imperative     | expose / disabled              | `setValue` updates both fields and the open calendar; `setValue(null)` empties them; disabled pickers disable the trigger                    | `exposes imperative range and open control and respects disabled`                                                                    |
| Q4  | SSR/hydration  | closed and open requests       | byte-identical markup in both states; an open range picker hydrates silently                                                                 | `renders byte-identical closed and open range picker markup across SSR requests`, `hydrates an open range picker without mismatches` |

Root data: `data-vize-ui="date-range-picker"`, `data-state`, `data-start`,
`data-end`, `data-disabled`, `data-readonly`. Each field root carries
`data-boundary`.
