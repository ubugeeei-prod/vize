# RangeCalendar behavior contract

Normative state x input -> outcome table for `range-calendar-root.vue`
(`@vizejs/ui/range-calendar`). The root publishes the same context as
CalendarRoot, so `calendar-grid.vue`, `calendar-heading.vue`,
`calendar-prev.vue`, `calendar-next.vue`, and the month/year selects are reused
as `RangeCalendar*` parts; keyboard, locale, bounds, and SSR rules from
`calendar.behavior.md` apply unchanged. Every row is proven by the named test.

| #   | State                  | Input                        | Outcome                                                                                                                                                                           | Proven by                                                                                                          |
| --- | ---------------------- | ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| R1  | empty                  | first activation             | the date becomes the anchor (`data-anchor`, `anchor-change`); grids are `aria-multiselectable`                                                                                    | `two activations anchor, preview, and commit an ordered range`                                                     |
| R2  | anchored               | pointer enter                | the preview range is marked with `data-in-range`, `data-range-start`, `data-range-end`, and `data-preview`; previewed middles are `aria-selected="false"`                         | `two activations anchor, preview, and commit an ordered range`                                                     |
| R3  | anchored               | second activation            | an ordered range commits: `anchor-change(null)`, `update:modelValue`, `change`; middles become `range-middle` and `aria-selected="true"`; `startName`/`endName` inputs submit ISO | `two activations anchor, preview, and commit an ordered range`                                                     |
| R4  | anchored               | keyboard focus / Escape      | focus movement previews the range; Escape cancels the anchor and is consumed only when an anchor exists                                                                           | `keyboard focus previews the range and Escape cancels the anchor`                                                  |
| R5  | unavailable dates      | completing activation        | a range spanning an unavailable date restarts the anchor unless `allowNonContiguousRanges`                                                                                        | `unavailable dates break ranges unless non-contiguous ranges are allowed`                                          |
| R6  | controlled / read-only | render / expose / activation | controlled ranges render across months (outside days included); `setValue`, `cancel`, `anchor` are exposed; read-only blocks anchoring                                            | `controlled ranges, read-only mode, and the exposed API`                                                           |
| R7  | SSR/hydration          | isolated requests            | byte-identical markup, no anchor or preview state, silent hydration                                                                                                               | `renders byte-identical range markup across isolated SSR requests`, `hydrates a range calendar without mismatches` |

## Props and emits beyond CalendarRoot

| Surface                       | Contract                                                                 |
| ----------------------------- | ------------------------------------------------------------------------ |
| `modelValue` / `defaultValue` | `DateRange \| null`; ranges are normalized so `start <= end`.            |
| `allowNonContiguousRanges`    | Permit committed ranges that include unavailable dates. Default `false`. |
| `startName` / `endName`       | Hidden inputs submitting the ISO endpoints.                              |
| `anchor-change(anchor)`       | Fires when the first endpoint is picked, cancelled, or completed.        |
| `select(range, event)`        | Fires for every completed selection, even when unchanged.                |
| root data                     | `data-mode="range"`, `data-start`, `data-end`, `data-anchor`.            |
