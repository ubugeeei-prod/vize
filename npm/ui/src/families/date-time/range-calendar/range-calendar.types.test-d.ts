/** Compile-only assertions for the public RangeCalendar contract. */

import {
  RangeCalendar,
  RangeCalendarGrid,
  RangeCalendarRoot,
  type DateRange,
  type PlainDate,
  type RangeCalendarRootExpose,
} from "./range-calendar.ts";
import { CalendarGrid, createDateRange } from "../calendar/calendar.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

declare const root: RangeCalendarRootExpose;

type _ValueIsRange = Expect<Equal<typeof root.value, DateRange | null>>;
type _AnchorIsDate = Expect<Equal<typeof root.anchor, PlainDate | null>>;
type _GridIsShared = Expect<Equal<typeof RangeCalendarGrid, typeof CalendarGrid>>;

const props: InstanceType<typeof RangeCalendarRoot>["$props"] = {
  modelValue: createDateRange({ year: 2026, month: 9, day: 1 }, { year: 2026, month: 9, day: 5 }),
  allowNonContiguousRanges: true,
  startName: "from",
  endName: "to",
  "onUpdate:modelValue": (value: DateRange | null) => value,
  "onAnchor-change": (anchor: PlainDate | null) => anchor,
};

root.cancel();

const badValue: InstanceType<typeof RangeCalendarRoot>["$props"] = {
  // @ts-expect-error range calendars take a range, not a single date.
  modelValue: { year: 2026, month: 1, day: 1 },
};

void RangeCalendar;
void props;
void badValue;
