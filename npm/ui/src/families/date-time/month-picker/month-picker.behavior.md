# MonthPicker behavior contract

Normative state x input -> outcome table for `month-picker.vue`
(`@vizejs/ui/month-picker`), a grid of the twelve months of one year built on
the shared linear period grid (`period-grid-runtime.ts`). Every row is proven
by the named test.

| #   | State              | Input                                  | Outcome                                                                                                                                                                                                       | Proven by                                                                                                                    |
| --- | ------------------ | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| MP1 | known year         | render                                 | a `role="group"` labelled by the live year heading holds a `<table role="grid">` of `columns`-wide rows; one roving `tabindex=0` month; the current month has `aria-current="date"`; `name` submits `YYYY-MM` | `renders a labelled month grid with roving focus and the current month`                                                      |
| MP2 | focused            | Arrow / Home / End / PageUp / PageDown | ±1 month, ±1 row, row edges, ±1 year, Shift ±10 years; Enter/Space selects and emits `update:modelValue` and `change`                                                                                         | `keyboard moves by month, row, row edge, year, and decade; activation selects`                                               |
| MP3 | RTL / bounds       | arrows / activation / previous–next    | RTL flips horizontal arrows; focus clamps to `min`/`max`; out-of-range months are disabled; unavailable months stay focusable and unselectable; year controls disable at bounds                               | `RTL, min/max bounds, unavailability, and year paging controls`                                                              |
| MP4 | no value, no clock | mount / expose                         | pending until mount, then the host clock picks the year; `setValue`, `navigate`, and `focus` are exposed                                                                                                      | `pending until mount without a clock, exposed API, and unit helpers`                                                         |
| MP5 | SSR/hydration      | isolated requests                      | byte-identical markup and silent hydration with an explicit `today`                                                                                                                                           | `renders byte-identical month picker markup across isolated SSR requests`, `hydrates month picker markup without mismatches` |

SSR determinism follows `calendar.behavior.md`: pass `today` or `now` +
`timeZone`, or a value, to render a complete grid on the server.
Parts: `root`, `header`, `previous`, `heading`, `next`, `grid`, `row`, `cell`, `month`.
