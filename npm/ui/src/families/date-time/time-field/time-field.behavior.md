# TimeField behavior contract

Normative state x input -> outcome table for `time-field.vue`
(`@vizejs/ui/time-field`). TimeField shares the DateField segment engine, so
DateField rows F2–F8 and F10–F12 apply to hour, minute, second, and day-period
segments. Every row below is proven by the named test.

| #   | State              | Input                 | Outcome                                                                                                                                             | Proven by                                                                                                                                |
| --- | ------------------ | --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| T1  | locale clock       | render                | segment order, hour padding, and day-period labels follow `Intl`; `hourCycle` forces 12 or 24; `granularity` adds or removes minute/second segments | `renders locale clocks: 12-hour with a day period or 24-hour without`                                                                    |
| T2  | 12-hour, empty     | digits and `a`/`p`    | hours, minutes, and the day period fill in order and commit a 24-hour `PlainTime`; letters and arrows toggle AM/PM                                  | `typing hours, minutes, and a day period commits a 24-hour value`                                                                        |
| T3  | 12-hour            | step                  | 12 AM is midnight and 12 PM is noon; hours wrap 12→1 without flipping the period; minutes wrap and Page steps by 15                                 | `12 AM and 12 PM map to midnight and noon; hours wrap within the clock`                                                                  |
| T4  | 24-hour, seconds   | digits                | two-digit hours wait for the second digit when it can fit; `second` granularity commits only after seconds; no hidden input without `name`          | `24-hour typing accepts two-digit hours and seconds granularity requires seconds`                                                        |
| T5  | min / max          | commit / empty step   | out-of-window times are invalid; `placeholderValue` seeds stepping from empty                                                                       | `min and max mark times invalid; placeholderValue seeds empty stepping`                                                                  |
| T6  | disabled/read-only | keys                  | no edits are committed                                                                                                                              | `disabled and read-only time fields block edits`                                                                                         |
| T7  | imperative         | expose / clock change | `hourCycle`, `granularity`, `focus`, `setValue`, `clear`; switching the clock re-renders the same time                                              | `exposes hour cycle, granularity, and imperative editing`                                                                                |
| T8  | time model         | helpers               | `PlainTime` validation, comparison, truncation, and `HH:MM[:SS]` round-trips                                                                        | `plain time helpers validate, compare, truncate, and round-trip ISO text`                                                                |
| T9  | SSR/hydration      | isolated requests     | byte-identical segments and deterministic ids hydrate silently                                                                                      | `renders byte-identical locale-ordered time segments across SSR requests`, `hydrates time segments with generated ids and no mismatches` |

Root data adds `data-hour-cycle` (`12`/`24`) and `data-granularity` to the
DateField hooks, with `data-vize-ui="time-field"`, `time-field-segment`, and
`time-field-literal`. The hidden input submits `HH:MM`, or `HH:MM:SS` for
`second` granularity.
