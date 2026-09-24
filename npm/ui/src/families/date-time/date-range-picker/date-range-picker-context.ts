import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { DateRange, PlainDate } from "../calendar/plain-date.ts";
import type { PickerSharedContext } from "../date-picker/date-picker-context.ts";

/** Range endpoint edited by a DateRangePickerField. */
export type DateRangeBoundary = "start" | "end";

/** Per-boundary record used by range picker parts. */
export type DateRangeBoundaryRecord<Value> = Readonly<Record<DateRangeBoundary, Value>>;

/** Shared state published by DateRangePickerRoot. */
export interface DateRangePickerContextValue extends PickerSharedContext {
  readonly value: ComputedRef<DateRange | null>;
  readonly setValue: (value: DateRange | null, event?: Event | null) => boolean;
  readonly drafts: ComputedRef<DateRangeBoundaryRecord<PlainDate | null>>;
  readonly setBoundary: (
    boundary: DateRangeBoundary,
    value: PlainDate | null,
    event?: Event | null,
  ) => void;
  readonly fieldIds: ComputedRef<DateRangeBoundaryRecord<string>>;
  readonly names: ComputedRef<DateRangeBoundaryRecord<string | undefined>>;
}

export const dateRangePickerContext = createContext<DateRangePickerContextValue>("DateRangePicker");
