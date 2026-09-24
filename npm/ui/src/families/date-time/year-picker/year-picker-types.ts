import type { PeriodGridCell, PeriodPickerState } from "../month-picker/month-picker-types.ts";

/** State exposed for one year cell. */
export interface YearPickerCellState extends PeriodGridCell {
  /** ISO year of the cell. */
  readonly year: number;

  /** Localized year label, also used as the accessible name. */
  readonly label: string;
}

/** State exposed to YearPicker slots and its instance. */
export interface YearPickerSlotState {
  /** Selected year. */
  readonly value: number | null;

  /** First and last year of the visible page, or `null` while pending. */
  readonly firstYear: number | null;
  readonly lastYear: number | null;

  /** Localized page heading such as `2016 – 2027`. */
  readonly heading: string;

  /** Year cells in rows of `columns`. */
  readonly rows: readonly (readonly YearPickerCellState[])[];

  /** Whether previous/next page navigation is possible. */
  readonly canGoPrevious: boolean;
  readonly canGoNext: boolean;

  /** Stable state token. */
  readonly state: PeriodPickerState;
}

/** Public instance exposed by YearPicker. */
export interface YearPickerExpose extends YearPickerSlotState {
  /** Rendered root element. */
  readonly root: HTMLDivElement | null;

  /** Focus the roving year; returns whether focus moved. */
  readonly focus: (options?: FocusOptions) => boolean;

  /** Request a value; returns whether it differs. */
  readonly setValue: (value: number | null) => boolean;

  /** Show another page of years; returns whether the view moved. */
  readonly navigate: (direction: -1 | 1) => boolean;
}
