# YearPicker behavior contract

Normative state x input -> outcome table for `year-picker.vue`
(`@vizejs/ui/year-picker`), a paged grid of years on the MonthPicker period
grid (rows MP2–MP4 apply with pages of `pageSize` years instead of years of
twelve months). Every row is proven by the named test.

| #   | State      | Input                     | Outcome                                                                                                                          | Proven by                                                                                                                  |
| --- | ---------- | ------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| Y1  | known      | render                    | pages align to multiples of `pageSize` (`2016 – 2027` for 12); selected and current years are marked; `name` submits the year    | `renders an aligned page of years with the current and selected year`                                                      |
| Y2  | focused    | Arrow / PageUp / PageDown | rows are `columns` wide; pages move by `pageSize`, Shift by ten pages; focus clamps to `min`/`max` and paging disables at bounds | `keyboard pages by row and page, selection emits, and bounds disable paging`                                               |
| Y3  | imperative | expose / predicate        | `navigate`, `setValue` (scrolls to the page), `firstYear`/`lastYear`; unavailable years are marked                               | `exposes value and paging`                                                                                                 |
| Y4  | SSR        | isolated requests         | byte-identical markup and silent hydration                                                                                       | `renders byte-identical year picker markup across isolated SSR requests`, `hydrates year picker markup without mismatches` |
