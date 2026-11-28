/** Closed notation hook for {@link formatStatistic}. */
export type StatisticNotation = "standard" | "compact";

/** Inputs for the pure statistic formatter. */
export interface StatisticFormat {
  readonly value: number;
  readonly locale: string;
  readonly maximumFractionDigits: number;
  readonly notation: StatisticNotation;
  readonly prefix: string;
  readonly suffix: string;
}

/**
 * Format a statistic so the sign stays in front of the prefix.
 *
 * `-$1,280` reads as a negative amount. `$-1,280` does not.
 */
export function formatStatistic(options: StatisticFormat): string {
  const negative = options.value < 0;
  const formatted = new Intl.NumberFormat(options.locale, {
    maximumFractionDigits: options.maximumFractionDigits,
    notation: options.notation,
  }).format(Math.abs(options.value));
  const body = `${options.prefix}${formatted}${options.suffix}`;
  return negative ? `-${body}` : body;
}
