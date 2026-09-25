import type { PlainYearMonth } from "../calendar/plain-date.ts";
import type { PeriodGridCell } from "./period-grid-runtime.ts";

export type { PeriodGridCell };

/** Width of month labels. */
export type MonthPickerFormat = "long" | "short" | "narrow" | "numeric" | "2-digit";

/** Root state token mirrored to `data-state`. */
export type PeriodPickerState = "disabled" | "empty" | "pending" | "readonly" | "selected";

/** State exposed for one month cell. */
export interface MonthPickerCellState extends PeriodGridCell {
  /** ISO year of the cell. */
  readonly year: number;

  /** ISO month (`1`–`12`) of the cell. */
  readonly month: number;

  /** Localized month label. */
  readonly label: string;

  /** Localized month and year used as the accessible name. */
  readonly fullLabel: string;
}

/** State exposed to MonthPicker slots and its instance. */
export interface MonthPickerSlotState {
  /** Selected month. */
  readonly value: PlainYearMonth | null;

  /** Year of the visible page, or `null` while pending. */
  readonly year: number | null;

  /** Localized year heading. */
  readonly heading: string;

  /** Month cells in rows of `columns`. */
  readonly rows: readonly (readonly MonthPickerCellState[])[];

  /** Whether previous/next year navigation is possible. */
  readonly canGoPrevious: boolean;
  readonly canGoNext: boolean;

  /** Stable state token. */
  readonly state: PeriodPickerState;
}

/** Public instance exposed by MonthPicker. */
export interface MonthPickerExpose extends MonthPickerSlotState {
  /** Rendered root element. */
  readonly root: HTMLDivElement | null;

  /** Focus the roving month; returns whether focus moved. */
  readonly focus: (options?: FocusOptions) => boolean;

  /** Request a value; returns whether it differs. */
  readonly setValue: (value: PlainYearMonth | null) => boolean;

  /** Show another year; returns whether the view moved. */
  readonly navigate: (direction: -1 | 1) => boolean;
}
