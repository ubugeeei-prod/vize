/** Compile-only assertions for the public Calendar contract. */

import {
  Calendar,
  CalendarGrid,
  CalendarNext,
  CalendarRoot,
  CalendarYearSelect,
  createPlainDate,
  parseIsoDate,
  type CalendarDayState,
  type CalendarDayStateToken,
  type CalendarNavigationUnit,
  type CalendarRootExpose,
  type CalendarSlotState,
  type CalendarState,
  type DateMatcher,
  type DateRange,
  type PlainDate,
  type Weekday,
} from "./calendar.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: CalendarRootExpose;
declare const slot: CalendarSlotState;
declare const day: CalendarDayState;

type _PlainDateIsStructural = Expect<
  Equal<PlainDate, { readonly year: number; readonly month: number; readonly day: number }>
>;
type _RangeHoldsDates = Expect<Equal<DateRange["start"], PlainDate>>;
type _WeekdayIsClosed = Expect<Equal<Weekday, 0 | 1 | 2 | 3 | 4 | 5 | 6>>;
type _MatcherIsPredicate = Expect<Equal<DateMatcher, (date: PlainDate) => boolean>>;
type _StateIsClosed = Expect<
  Equal<CalendarState, "disabled" | "empty" | "pending" | "readonly" | "selected">
>;
type _DayStateIsClosed = Expect<
  Equal<
    CalendarDayStateToken,
    "disabled" | "idle" | "outside" | "range-middle" | "selected" | "unavailable"
  >
>;
type _UnitIsClosed = Expect<Equal<CalendarNavigationUnit, "month" | "year">>;
type _ExposeValue = Expect<Equal<typeof root.value, PlainDate | null>>;
type _ExposeSetValue = Expect<Equal<typeof root.setValue, (value: PlainDate | null) => boolean>>;
type _FocusedDateMayBePending = Expect<Equal<typeof slot.focusedDate, PlainDate | null>>;
type _DayDate = Expect<Equal<typeof day.date, PlainDate>>;
type _ParseIsNullable = Expect<Equal<ReturnType<typeof parseIsoDate>, PlainDate | null>>;

const temporalLike: PlainDate = { year: 2026, month: 9, day: 25 };
const rootProps: InstanceType<typeof CalendarRoot>["$props"] = {
  modelValue: createPlainDate(2026, 9, 25),
  min: temporalLike,
  max: null,
  numberOfMonths: 2,
  weekStartsOn: 1,
  weekdayFormat: "narrow",
  isDateUnavailable: (date) => date.day === 13,
  now: () => Date.UTC(2026, 8, 25),
  timeZone: "Asia/Tokyo",
  "onUpdate:modelValue": (value: PlainDate | null) => value,
};
const gridProps: InstanceType<typeof CalendarGrid>["$props"] = { monthIndex: 1 };
const nextProps: InstanceType<typeof CalendarNext>["$props"] = { unit: "year" };
const yearProps: InstanceType<typeof CalendarYearSelect>["$props"] = { from: 1990, to: 2030 };

root.navigate("month", 1);
root.setVisibleMonth({ year: 2027, month: 1 });

// @ts-expect-error weekStartsOn is a closed weekday index.
const badWeekStart: InstanceType<typeof CalendarRoot>["$props"] = { weekStartsOn: 7 };

// @ts-expect-error model values are plain dates, not Date objects.
const badValue: InstanceType<typeof CalendarRoot>["$props"] = { modelValue: new Date() };

// @ts-expect-error navigation units are month or year only.
const badUnit: InstanceType<typeof CalendarNext>["$props"] = { unit: "week" };

// @ts-expect-error navigation direction is -1 or 1.
root.navigate("month", 2);

void Calendar;
void rootProps;
void gridProps;
void nextProps;
void yearProps;
void badWeekStart;
void badValue;
void badUnit;
