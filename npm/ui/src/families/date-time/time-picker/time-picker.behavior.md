# TimePicker behavior contract

Normative state x input -> outcome table for `time-picker.vue`
(`@vizejs/ui/time-picker`): a `role="listbox"` of time slots composed from the
Listbox family (`listbox.behavior.md` covers keyboard, typeahead, and
active-descendant focus). Every row is proven by the named test.

| #   | State      | Input               | Outcome                                                                                                                                                            | Proven by                                                                                                                  |
| --- | ---------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| TP1 | any        | render              | options run from `min` to `max` every `step` minutes with localized labels (also the typeahead text); the selected slot is `aria-selected`; `name` submits `HH:MM` | `renders a listbox of localized slots between min and max every step minutes`                                              |
| TP2 | enabled    | option activation   | emits `update:modelValue` and `change` with `PlainTime`; unavailable slots are disabled options; read-only ignores selection                                       | `selecting slots emits PlainTime values and honors unavailable slots and read-only`                                        |
| TP3 | imperative | expose / generation | `value`, `slots`, `hourCycle`, `focus`, `setValue`; `createTimeSlots` clamps `step` to 1–1440 and rounds `min` up to whole minutes                                 | `exposes value, slots, focus, and setValue; slot generation clamps inputs`                                                 |
| TP4 | SSR        | isolated requests   | byte-identical markup and silent hydration                                                                                                                         | `renders byte-identical time picker markup across isolated SSR requests`, `hydrates time picker markup without mismatches` |
