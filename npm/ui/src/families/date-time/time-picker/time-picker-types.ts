import type { HourCycle, PlainTime } from "../time-field/plain-time.ts";

/** One generated time slot. */
export interface TimePickerSlot {
  /** Slot time. */
  readonly time: PlainTime;

  /** `HH:MM` option value. */
  readonly value: string;

  /** Localized time label, also the option's typeahead text. */
  readonly label: string;

  /** Whether `isTimeUnavailable` rejected the slot. */
  readonly disabled: boolean;

  /** Whether the slot is the selected time. */
  readonly selected: boolean;
}

/** State exposed to TimePicker slots and its instance. */
export interface TimePickerSlotState {
  /** Selected time, when it matches a slot. */
  readonly value: PlainTime | null;

  /** Generated slots between `min` and `max`, `step` minutes apart. */
  readonly slots: readonly TimePickerSlot[];

  /** Resolved hour clock used for labels. */
  readonly hourCycle: HourCycle;

  /** Whether the listbox is disabled. */
  readonly disabled: boolean;

  /** Whether selection is locked. */
  readonly readOnly: boolean;
}

/** Public instance exposed by TimePicker. */
export interface TimePickerExpose extends TimePickerSlotState {
  /** Focus the listbox. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a value; returns whether it differs. */
  readonly setValue: (value: PlainTime | null) => boolean;
}
