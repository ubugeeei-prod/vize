import type { DateRange } from "../calendar/plain-date.ts";
import type { PickerOpenState } from "../date-picker/date-picker-types.ts";
import type { DateRangeBoundary } from "./date-range-picker-context.ts";

export type { DateRangeBoundary };

/** State exposed to DateRangePickerRoot slots and instances. */
export interface DateRangePickerSlotState {
  /** Committed range, or `null` until both endpoints are known. */
  readonly value: DateRange | null;

  /** Whether the calendar popover is open. */
  readonly open: boolean;

  /** Whether the picker is disabled. */
  readonly disabled: boolean;

  /** Whether the value is locked while the calendar can still be browsed. */
  readonly readOnly: boolean;

  /** Stable open-state token. */
  readonly state: PickerOpenState;
}

/** Public instance exposed by DateRangePickerRoot. */
export interface DateRangePickerRootExpose extends DateRangePickerSlotState {
  /** Request a range; returns whether it differs. */
  readonly setValue: (value: DateRange | null) => boolean;

  /** Request an open state; returns whether it differs. */
  readonly setOpen: (value: boolean) => boolean;
}
