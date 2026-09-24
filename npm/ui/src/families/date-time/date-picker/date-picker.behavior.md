# DatePicker behavior contract

Normative state x input -> outcome table for the DatePicker compound
(`@vizejs/ui/date-picker`): `date-picker-root.vue`, `date-picker-field.vue`,
`date-picker-content.vue`, and `date-picker-calendar.vue`, with
`DatePickerTrigger` re-exporting PopoverTrigger. The root wraps PopoverRoot, so
dismissal, focus containment, and positioning follow `popover.behavior.md`;
field and calendar rules follow `date-field.behavior.md` and
`calendar.behavior.md`. Every row is proven by the named test.

| #   | State                             | Input                    | Outcome                                                                                                                                                 | Proven by                                                                                                                |
| --- | --------------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| P1  | closed                            | trigger activation       | the trigger (`aria-haspopup="dialog"`) opens the calendar dialog, emits `update:open` and `open-change`, and moves focus to the selected (or today) day | `renders a date field with a trigger that opens a calendar dialog focused on the value`                                  |
| P2  | open                              | day activation           | the value commits (`update:modelValue`, `change`), the popover closes, focus returns to the trigger, and the field segments update                      | `selecting a day commits the value, closes the popover, and restores focus to the trigger`                               |
| P3  | open                              | field typing / Escape    | a value typed in the field scrolls the open calendar to it; Escape dismisses                                                                            | `typing in the field moves the open calendar and keyboard Escape dismisses`                                              |
| P4  | `closeOnSelect=false` / read-only | day activation           | the popover stays open; read-only pickers never change the value                                                                                        | `closeOnSelect=false keeps the popover open and read-only blocks value changes`                                          |
| P5  | disabled / imperative             | render / expose          | disabled pickers disable the trigger and field; `setOpen` and `setValue` are exposed; `min` reaches the calendar                                        | `disabled pickers cannot open and expose imperative value and open control`                                              |
| P6  | SSR/hydration                     | closed and open requests | byte-identical markup in both states; an open picker hydrates silently                                                                                  | `renders byte-identical closed and open picker markup across SSR requests`, `hydrates an open picker without mismatches` |

`DatePickerContent` forwards `placement`, `initialFocus`, `ariaLabel`, and
`ariaLabelledby`; every other PopoverContent prop (for example `portalDisabled`
or `offset`) falls through as an attribute. Root data: `data-vize-ui="date-picker"`,
`data-state` (`open`/`closed`), `data-value`, `data-disabled`, `data-readonly`.
Pass `today` or `now` to the root for SSR-stable calendars.
