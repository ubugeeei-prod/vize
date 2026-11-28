import type { StatisticNotation } from "./statistic-format.ts";

/** Public props for the statistic summary. */
export interface StatisticProps {
  /**
   * Accessible name and visible title of the figure.
   */
  readonly title: string;

  /**
   * Numeric value. The sign is rendered before `prefix`.
   */
  readonly value: number;

  /**
   * Text placed immediately before the formatted number.
   *
   * @default ""
   */
  readonly prefix?: string;

  /**
   * Text placed immediately after the formatted number.
   *
   * @default ""
   */
  readonly suffix?: string;

  /**
   * BCP 47 locale passed to `Intl.NumberFormat`.
   *
   * @default "en-US"
   */
  readonly locale?: string;

  /**
   * Maximum fraction digits. Grouping follows `locale`.
   *
   * @default 0
   */
  readonly maximumFractionDigits?: number;

  /**
   * `compact` renders short forms such as `1.2K`.
   *
   * @default "standard"
   */
  readonly notation?: StatisticNotation;
}
