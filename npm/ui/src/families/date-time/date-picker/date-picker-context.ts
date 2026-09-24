import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { DateTimeNow } from "../calendar/calendar-today.ts";
import type { DateMatcher, PlainDate } from "../calendar/plain-date.ts";

/** Root options forwarded by picker roots to their field and calendar parts. */
export interface PickerOptions {
  readonly min: PlainDate | null | undefined;
  readonly max: PlainDate | null | undefined;
  readonly isDateUnavailable: DateMatcher | undefined;
  readonly locale: string | undefined;
  readonly dir: "ltr" | "rtl" | undefined;
  readonly today: PlainDate | null | undefined;
  readonly now: DateTimeNow | undefined;
  readonly timeZone: string | undefined;
  readonly closeOnSelect: boolean;
  readonly allowNonContiguousRanges: boolean;
}

/** State shared by DatePicker and DateRangePicker parts. */
export interface PickerSharedContext {
  readonly open: ComputedRef<boolean>;
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;
  readonly options: ComputedRef<PickerOptions>;
  readonly disabled: ComputedRef<boolean>;
  readonly readOnly: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  /** Register the element that receives focus when the popover opens. */
  readonly registerFocusTarget: (target: () => HTMLElement | null) => () => void;
  /** Element that receives focus when the popover opens, usually the roving calendar day. */
  readonly focusTarget: () => HTMLElement | null;
}

/** Shared state published by DatePickerRoot. */
export interface DatePickerContextValue extends PickerSharedContext {
  readonly fieldId: ComputedRef<string>;
  readonly name: ComputedRef<string | undefined>;
  readonly value: ComputedRef<PlainDate | null>;
  readonly setValue: (value: PlainDate | null, event?: Event | null) => boolean;
}

export const datePickerContext = createContext<DatePickerContextValue>("DatePicker");
