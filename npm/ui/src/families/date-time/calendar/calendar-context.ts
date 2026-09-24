import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CalendarFormatters } from "./calendar-locale.ts";
import type { CalendarNavigationUnit, CalendarSlotState } from "./calendar-types.ts";
import type { PlainDate, PlainYearMonth } from "./plain-date.ts";

/** Shared state and handlers published by CalendarRoot and RangeCalendarRoot. */
export interface CalendarContextValue {
  readonly root: Readonly<ShallowRef<HTMLDivElement | null>>;
  readonly id: ComputedRef<string>;
  readonly headingId: ComputedRef<string>;
  readonly slotState: ComputedRef<CalendarSlotState>;
  readonly formatters: ComputedRef<CalendarFormatters>;
  readonly visibleStart: ComputedRef<PlainYearMonth | null>;
  readonly min: ComputedRef<PlainDate | null>;
  readonly max: ComputedRef<PlainDate | null>;
  readonly canNavigate: (unit: CalendarNavigationUnit, direction: -1 | 1) => boolean;
  readonly navigate: (unit: CalendarNavigationUnit, direction: -1 | 1) => boolean;
  readonly setVisibleMonth: (month: PlainYearMonth) => void;
  readonly onDayClick: (date: PlainDate, event: MouseEvent) => void;
  readonly onDayKeydown: (date: PlainDate, event: KeyboardEvent) => void;
  readonly onDayHover: (date: PlainDate | null) => void;
}

export const calendarContext = createContext<CalendarContextValue>("Calendar");
