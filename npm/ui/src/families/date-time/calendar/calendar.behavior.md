# Calendar behavior contract

Normative state x input -> outcome table for the Calendar compound
(`@vizejs/ui/calendar`): `calendar-root.vue`, `calendar-grid.vue`,
`calendar-heading.vue`, `calendar-prev.vue`, `calendar-next.vue`,
`calendar-month-select.vue`, and `calendar-year-select.vue`, plus the
timezone-free date model in `plain-date.ts`. Every row is proven by the named
mounted-DOM, SSR, or unit test; a row without a passing test is a contract
violation.

| #   | State                 | Input                                  | Outcome                                                                                                                                                                                                                   | Proven by                                                                                                                                                                                                                                         |
| --- | --------------------- | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C1  | known date            | render                                 | root `role="group"` labelled by the heading; each month is a `<table role="grid">` named by its month; weekday headers carry `abbr`; one roving `tabindex=0` day; today has `aria-current="date"`; outside days are inert | `renders an APG date grid with locale weekdays, roving focus, and today`                                                                                                                                                                          |
| C2  | uncontrolled          | day activation                         | emits `update:modelValue` then `change(value, previous, event)`; selection, `data-*`, and the hidden ISO input update                                                                                                     | `uncontrolled selection emits update and change with the previous date`                                                                                                                                                                           |
| C3  | controlled            | day activation                         | emits the request; rendered selection follows `modelValue` until the parent accepts it                                                                                                                                    | `controlled value wins until the parent accepts the request`                                                                                                                                                                                      |
| C4  | focused day           | Arrow / Home / End / PageUp / PageDown | ±1 day, ±1 week, week start/end, ±1 month, Shift ±1 year; focus follows into new months; `update:focusedDate` fires; Tab is not intercepted                                                                               | `keyboard moves focus by day, week, week edge, month, and year across months`                                                                                                                                                                     |
| C5  | RTL / locale          | render / ArrowLeft / ArrowRight        | `dir` flips horizontal arrows; week start comes from CLDR region data (or `-u-fw-`, or `weekStartsOn`); labels come from `Intl`                                                                                           | `RTL flips horizontal arrows and locale week start follows the region`                                                                                                                                                                            |
| C6  | min / max / predicate | render / keyboard / activation         | out-of-range days are disabled and focus clamps to the window; unavailable days stay focusable with `aria-disabled` and never select; prev/next disable at the bounds                                                     | `min, max, and unavailable dates constrain focus, navigation, and selection`                                                                                                                                                                      |
| C7  | multiple months       | prev / next / keyboard                 | `numberOfMonths` grids render side by side; paged navigation moves by the page; moving focus past the window scrolls it by one month; moving back inside does not scroll                                                  | `previous and next controls page by month or year and keep focus date in view`                                                                                                                                                                    |
| C8  | month / year selects  | native `change`                        | the chosen month or year becomes the first visible month; focus date keeps its day clamped to the month; out-of-range months are disabled options                                                                         | `month and year selects jump the view and respect bounds`                                                                                                                                                                                         |
| C9  | disabled / read-only  | render / activation                    | disabled removes every day and control from interaction; read-only keeps navigation and focus but blocks selection with `aria-readonly`/`aria-disabled`                                                                   | `disabled and read-only calendars keep availability semantics`                                                                                                                                                                                    |
| C10 | no value, no clock    | mount                                  | the grid is `pending` until mount, then reads the host clock in `timeZone`, the nearest `LocaleProvider` zone, or the host zone                                                                                           | `without today or now the calendar is pending until mount reads the injected time zone clock`                                                                                                                                                     |
| C11 | LocaleProvider        | render                                 | locale, direction, display calendar (`japanese`, `buddhist`, …), and numbering system are inherited                                                                                                                       | `LocaleProvider supplies locale, direction, calendar display, and numbering`                                                                                                                                                                      |
| C12 | imperative            | expose                                 | `focus()`, `setValue()`, `setFocusedDate()`, `setVisibleMonth()`, `navigate()` plus normalized slot state                                                                                                                 | `exposes focus, setValue, setFocusedDate, setVisibleMonth, and navigate`                                                                                                                                                                          |
| C13 | custom composition    | default slot                           | typed `CalendarSlotState` drives consumer-rendered grids; `fixedWeeks` renders six rows; `day`/`weekday` slots replace cell content                                                                                       | `custom composition receives typed slot state and renders fixed weeks`                                                                                                                                                                            |
| C14 | SSR, explicit `today` | isolated requests / hydration          | markup is byte-identical, selected `<option>`s carry `selected`, and hydration reuses the root with no warnings                                                                                                           | `renders byte-identical calendar markup across isolated SSR requests`, `hydrates a calendar with an explicit today without mismatches`                                                                                                            |
| C15 | SSR, no clock         | isolated requests / hydration          | the server renders a deterministic `data-state="pending"` shell without days; hydration is silent and the client fills the grid after mount                                                                               | `without today or now SSR renders a pending shell and the client fills it after mount`                                                                                                                                                            |
| C16 | SSR, injected `now`   | server + client                        | the injected instant resolves in `timeZone` identically on both sides                                                                                                                                                     | `an injected now renders the same today on server and client`                                                                                                                                                                                     |
| D1  | date model            | arithmetic                             | epoch-day math matches UTC across eras and leap rules; month/year arithmetic clamps days                                                                                                                                  | `epoch-day arithmetic matches UTC Date math across eras and leap rules`, `month, year, and week arithmetic clamps to real calendar days`                                                                                                          |
| D2  | date model            | adapters                               | Temporal-like records are copied; ISO strings round-trip; `Date` adapters never shift days; instants resolve per IANA zone                                                                                                | `validation rejects impossible dates and copies Temporal-like records`, `ISO strings round-trip including extended years`, `Date adapters read local or UTC fields and never shift days`, `instants resolve to calendar dates per IANA time zone` |
| D3  | date model            | ranges / locale data                   | ranges are ordered; week start and formatters are deterministic                                                                                                                                                           | `ranges are ordered, normalized, and compared by day`, `week start follows CLDR regions, the fw extension, and explicit overrides`, `formatters render UTC-anchored labels with calendar and numbering overrides`                                 |

## SSR determinism

"Today" is never read from the wall clock during server rendering or
hydration. Resolution order:

1. `today` — an explicit `PlainDate`; deterministic everywhere.
2. `now` — an injected clock (for example the request timestamp) resolved in
   `timeZone`, else the nearest `LocaleProvider` time zone, else `UTC`. Inject
   the same instant on the server and the client.
3. Otherwise the calendar renders a `pending` shell (`data-state="pending"`,
   empty grid body) until mount, then reads `Date.now()` once in `timeZone`,
   the provided locale zone, or the host zone.

A `modelValue`, `defaultValue`, or `focusedDate` also decides which month is
visible, so a calendar with a value never waits for the clock. Week start uses
an embedded CLDR region table instead of `Intl.Locale#getWeekInfo()` so the
server and every browser agree. All labels format UTC midnight with
`timeZone: "UTC"`.

## Date model

`PlainDate` is `{ year, month, day }` in the ISO calendar. `Temporal.PlainDate`
(ISO calendar) satisfies it structurally; call `Temporal.PlainDate.from(value)`
to convert back. Adapters: `parseIsoDate` / `formatIsoDate`, `fromLocalDate` /
`toLocalDate`, `fromUtcDate` / `toUtcDate`, and `fromEpochMilliseconds(ms,
timeZone)`. Non-Gregorian calendars are supported for display only
(`calendar` prop or `LocaleProvider`): grid months stay ISO months, which is
exact for `gregory`, `iso8601`, `japanese`, `roc`, and `buddhist`.

## Public props (CalendarRoot)

| Prop                           | Type                            | Default     | Contract                                                  |
| ------------------------------ | ------------------------------- | ----------- | --------------------------------------------------------- |
| `modelValue` / `defaultValue`  | `PlainDate \| null`             | `undefined` | Controlled / uncontrolled selection.                      |
| `focusedDate`                  | `PlainDate \| null`             | `undefined` | Controlled roving focus date; decides the visible months. |
| `min` / `max`                  | `PlainDate \| null`             | `undefined` | Inclusive selectable window; swapped when reversed.       |
| `isDateUnavailable`            | `(date) => boolean`             | `undefined` | Focusable but unselectable dates.                         |
| `locale` / `dir`               | `string` / `"ltr" \| "rtl"`     | provider    | Defaults to the nearest `LocaleProvider`.                 |
| `calendar` / `numberingSystem` | `string`                        | provider    | Display-only Intl overrides.                              |
| `weekStartsOn`                 | `0`–`6`                         | locale      | First column; `0` is Sunday.                              |
| `weekdayFormat`                | `"narrow" \| "short" \| "long"` | `"short"`   | Column label width.                                       |
| `numberOfMonths`               | `number`                        | `1`         | Consecutive months, clamped to 1–12.                      |
| `pagedNavigation`              | `boolean`                       | `false`     | Month controls move by the page.                          |
| `fixedWeeks`                   | `boolean`                       | `false`     | Always six rows.                                          |
| `today` / `now` / `timeZone`   | see SSR determinism             | `undefined` | Clock injection.                                          |
| `disabled` / `readOnly`        | `boolean`                       | `false`     | Availability.                                             |
| `name`                         | `string`                        | `undefined` | Hidden input submitting `YYYY-MM-DD`.                     |

## Extension hooks

| Hook            | Values                                                                                                                                                                                                                     |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| parts           | `root`, `header`, `heading`, `prev`, `next`, `month-select`, `month-option`, `year-select`, `year-option`, `grid`, `grid-head`, `weekdays`, `weekday`, `grid-body`, `week`, `cell`, `day`                                  |
| root data       | `data-vize-ui="calendar"`, `data-mode`, `data-state`, `data-dir`, `data-months`, `data-value`, `data-disabled`, `data-readonly`, `data-pending`                                                                            |
| cell / day data | `data-date`, `data-state`, `data-weekday`, `data-today`, `data-focused`, `data-outside-month`, `data-selected`, `data-in-range`, `data-range-start`, `data-range-end`, `data-preview`, `data-disabled`, `data-unavailable` |
