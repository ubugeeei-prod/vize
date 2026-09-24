# DateTimeField behavior contract

Normative state x input -> outcome table for `datetime-field.vue`
(`@vizejs/ui/datetime-field`). DateTimeField renders date and time segments in
one locale order on the shared segment engine, so DateField rows F2–F8,
F10–F12, and F15 and TimeField rows T2–T4 apply. Every row below is proven by
the named test.

| #   | State             | Input               | Outcome                                                                                                                                          | Proven by                                                                                                                                          |
| --- | ----------------- | ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| DT1 | locale            | render              | year/month/day and hour/minute(/second)/day-period segments follow `Intl` order for the locale and clock; `name` submits `YYYY-MM-DDTHH:MM[:SS]` | `renders date and time segments in one locale order with a named ISO input`                                                                        |
| DT2 | empty             | digits              | typing advances across the date/time boundary and commits one `PlainDateTime` only when every segment is filled                                  | `typing fills every segment and commits a PlainDateTime`                                                                                           |
| DT3 | min / max / dates | commit / empty step | out-of-window date-times or unavailable dates are invalid; empty segments step from `placeholderValue` (else the host clock at key time)         | `min, max, and unavailable dates invalidate; placeholderValue seeds stepping`                                                                      |
| DT4 | imperative        | expose              | `setValue`, `clear`, and 12-hour rendering of midnight                                                                                           | `exposes the date-time value and imperative editing`                                                                                               |
| DT5 | model             | helpers             | `PlainDateTime` compare, minute arithmetic across days, ISO parsing, and zoned-instant resolution                                                | `plain date-time helpers compare, shift, parse, and resolve zoned instants`                                                                        |
| DT6 | SSR/hydration     | isolated requests   | byte-identical segments and deterministic ids hydrate silently                                                                                   | `renders byte-identical locale-ordered date-time segments across SSR requests`, `hydrates date-time segments with generated ids and no mismatches` |

Root data: `data-vize-ui="datetime-field"`, `data-state`, `data-value`,
`data-hour-cycle`, `data-granularity`; parts `root`, `segment`, `literal`.
