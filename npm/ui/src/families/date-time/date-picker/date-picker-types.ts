import type { PlainDate } from "../calendar/plain-date.ts";

/** Open-state token mirrored to picker `data-state`. */
export type PickerOpenState = "closed" | "open";

/** State exposed to DatePickerRoot slots and instances. */
export interface DatePickerSlotState {
  /** Committed date. */
  readonly value: PlainDate | null;

  /** Whether the calendar popover is open. */
  readonly open: boolean;

  /** Whether the picker is disabled. */
  readonly disabled: boolean;

  /** Whether the value is locked while the calendar can still be browsed. */
  readonly readOnly: boolean;

  /** Stable open-state token. */
  readonly state: PickerOpenState;
}

/** Public instance exposed by DatePickerRoot. */
export interface DatePickerRootExpose extends DatePickerSlotState {
  /** Request a value; returns whether it differs. */
  readonly setValue: (value: PlainDate | null) => boolean;

  /** Request an open state; returns whether it differs. */
  readonly setOpen: (value: boolean) => boolean;
}
